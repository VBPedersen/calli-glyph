// ------------------------------
// LSP message types, implemented based on 3.17 specification: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/
// ------------------------------

use crate::config::lsp::LspServerConfig;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

/// A diagnostic reported by the server for a specific position in a file.
/// Based on https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#diagnostic
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub line: u32,      // 0-indexed
    pub col_start: u32, // 0-indexed byte column
    pub col_end: u32,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source: Option<String>, // e.g. "rustc", "clippy"
}

///https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#diagnosticSeverity
/// Severity of diagnostic message
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

impl DiagnosticSeverity {
    fn from_lsp_int(n: u8) -> Self {
        match n {
            1 => Self::Error,
            2 => Self::Warning,
            3 => Self::Information,
            4 => Self::Hint,
            _ => Self::Hint,
        }
    }
}

/// A single completion item offered by the server.
/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#completionItem
#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub detail: Option<String>, // e.g. type signature
    pub kind: CompletionKind,
    pub documentation: Option<String>,
    pub insert_text: Option<String>, // what to actually insert; falls back to label
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#completionItemKind
/// The kind of a completion entry
#[derive(Debug, Clone)]
pub enum CompletionKind {
    Text,
    Method,
    Function,
    Constructor,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Unit,
    Value,
    Enum,
    Keyword,
    Snippet,
    Color,
    File,
    Reference,
    Folder,
    EnumMember,
    Constant,
    Struct,
    Event,
    Operator,
    TypeParameter,
    Other,
}

impl CompletionKind {
    fn from_lsp_int(n: u8) -> Self {
        match n {
            1 => Self::Text,
            2 => Self::Method,
            3 => Self::Function,
            4 => Self::Constructor,
            5 => Self::Field,
            6 => Self::Variable,
            7 => Self::Class,
            8 => Self::Interface,
            9 => Self::Module,
            10 => Self::Property,
            11 => Self::Unit,
            12 => Self::Value,
            13 => Self::Enum,
            14 => Self::Keyword,
            15 => Self::Snippet,
            16 => Self::Color,
            17 => Self::File,
            18 => Self::Reference,
            19 => Self::Folder,
            20 => Self::EnumMember,
            21 => Self::Constant,
            22 => Self::Struct,
            23 => Self::Event,
            24 => Self::Operator,
            25 => Self::TypeParameter,
            _ => Self::Other,
        }
    }
}

/// Hover documentation returned by the server.
#[derive(Debug, Clone)]
pub struct HoverResult {
    pub contents: String,
    pub range: Option<(u32, u32, u32, u32)>, // start_line, start_col, end_line, end_col
}

/// Messages the background reader thread sends to the main thread.
#[derive(Debug)]
pub enum LspMessage {
    /// textDocument/publishDiagnostics notification
    Diagnostics {
        uri: String,
        diagnostics: Vec<Diagnostic>,
    },
    /// Response to a completion request
    CompletionResponse {
        request_id: i64,
        items: Vec<CompletionItem>,
    },
    /// Response to a hover request
    HoverResponse {
        request_id: i64,
        result: Option<HoverResult>,
    },
    /// Server initialized successfully
    Initialized,
    /// Any error the server reported
    Error { code: i64, message: String },
    /// Server process died or the pipe closed
    ServerExited,
}

// ------------------------------
// LSP Client
// ------------------------------

/// Connection state of the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not yet started
    Disconnected,
    /// Server process running, waiting for initialize response
    Initializing,
    /// Ready to receive requests
    Ready,
    /// Server died or errored
    Failed(String),
}

/// Tracks which request id relates to which kind of pending request,
/// so when a response arrives it knows what to do with it.
#[derive(Debug)]
enum PendingRequest {
    Initialize,
    Completion,
    Hover,
}

/// The LSP client that handles all state of LSP, communication with,
/// and in general the entry to the LSP system
pub struct LspClient {
    pub server_name: String,
    pub state: ConnectionState,

    stdin: ChildStdin,
    pub incoming: Receiver<Value>, // raw JSON from the background reader
    _child: Child,

    next_id: i64,
    pending: HashMap<i64, PendingRequest>,

    /// Current diagnostics per file URI
    pub diagnostics: HashMap<String, Vec<Diagnostic>>,
    /// Most recent completion items
    pub completions: Vec<CompletionItem>,
    /// Most recent hover result
    pub hover: Option<HoverResult>,

    /// The request id of the in-flight completion request, to be able to
    /// ignore stale responses if the cursor moved.
    completion_request_id: Option<i64>,
    hover_request_id: Option<i64>,

    /// Document version counter, incremented on every didChange
    doc_version: i32,
}

impl LspClient {
    /// Spawn the server process and send the initialize request.
    /// Returns immediately, the initialize response arrives later with poll().
    pub fn start(
        server_name: String,
        config: &LspServerConfig,
        workspace_root: Option<&str>,
    ) -> Result<Self, String> {
        let mut child = Command::new(&config.command)
            .args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to spawn process {}:{}", config.command, e))?;

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        let (tx, rx): (Sender<Value>, Receiver<Value>) = channel();

        // Background reader thread, reads Content-Length framed JSON from stdout
        // and forwards raw Value objects to the main thread via the channel.
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                // Read headers until blank line
                let mut content_length: Option<usize> = None;
                loop {
                    let mut header_line = String::new();
                    match reader.read_line(&mut header_line) {
                        Ok(0) => {
                            let _ = tx.send(json!({"__exit": true}));
                            return;
                        }
                        Err(_) => {
                            let _ = tx.send(json!({"__exit": true}));
                            return;
                        }
                        Ok(_) => {}
                    }
                    let trimmed = header_line.trim();
                    if trimmed.is_empty() {
                        break; // end of headers
                    }
                    if let Some(rest) = trimmed.strip_prefix("Content-Length: ") {
                        content_length = rest.trim().parse().ok();
                    }
                }

                let len = match content_length {
                    Some(l) if l > 0 => l,
                    _ => continue,
                };

                let mut body = vec![0u8; len];
                use std::io::Read;
                if reader.read_exact(&mut body).is_err() {
                    let _ = tx.send(json!({"__exit": true}));
                    return;
                }

                match serde_json::from_slice::<Value>(&body) {
                    Ok(msg) => {
                        if tx.send(msg).is_err() {
                            return; // main thread dropped the receiver
                        }
                    }
                    Err(e) => {
                        log_warn!("[LSP] Failed to parse message: {}", e);
                    }
                }
            }
        });
        let init_options = if config.initialization_options.is_empty() {
            Value::Null
        } else {
            serde_json::to_value(&config.initialization_options).unwrap_or(Value::Null)
        };

        let root_uri = workspace_root
            .map(|r| format!("file://{}", r))
            .unwrap_or_else(|| "file:///".to_string());

        let mut client = Self {
            server_name,
            state: ConnectionState::Initializing,
            stdin,
            incoming: rx,
            _child: child,
            next_id: 1,
            pending: HashMap::new(),
            diagnostics: HashMap::new(),
            completions: Vec::new(),
            hover: None,
            completion_request_id: None,
            hover_request_id: None,
            doc_version: 0,
        };

        // Send initialize
        let id = client.alloc_id();
        client.pending.insert(id, PendingRequest::Initialize);
        client.send_request(
            id,
            "initialize",
            json!({
                "processId": std::process::id(),
                "clientInfo": { "name": "calli-glyph", "version": "0.1.0" },
                "rootUri": root_uri,
                "initializationOptions": init_options,
                "capabilities": {
                    "textDocument": {
                        "synchronization": {
                            "didSave": true,
                            "dynamicRegistration": false
                        },
                        "publishDiagnostics": {
                            "relatedInformation": true,
                            "versionSupport": false
                        },
                        "completion": {
                            "completionItem": {
                                "snippetSupport": false,
                                "documentationFormat": ["plaintext"]
                            }
                        },
                        "hover": {
                            "contentFormat": ["plaintext"]
                        }
                    },
                    "workspace": {
                        "workspaceFolders": false
                    }
                }
            }),
        )?;

        Ok(client)
    }

    // ------------------------------
    // Notifications to the server
    // ------------------------------

    /// Tell the server a file was opened. Call once when a file is first loaded.
    pub fn notify_did_open(
        &mut self,
        uri: &str,
        language_id: &str,
        text: &str,
    ) -> Result<(), String> {
        self.doc_version = 1;
        self.send_notification(
            "textDocument/didOpen",
            json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": language_id,
                    "version": self.doc_version,
                    "text": text
                }
            }),
        )
    }

    /// Tell the server the file changed. Call after every edit.
    /// Uses full-document sync (simplest, fast enough for most files).
    pub fn notify_did_change(&mut self, uri: &str, full_text: &str) -> Result<(), String> {
        self.doc_version += 1;
        self.send_notification(
            "textDocument/didChange",
            json!({
                "textDocument": {
                    "uri": uri,
                    "version": self.doc_version
                },
                "contentChanges": [{ "text": full_text }]
            }),
        )
    }

    /// Tell the server the file was saved.
    pub fn notify_did_save(&mut self, uri: &str) -> Result<(), String> {
        self.send_notification(
            "textDocument/didSave",
            json!({ "textDocument": { "uri": uri } }),
        )
    }

    /// Tell the server the file was closed.
    pub fn notify_did_close(&mut self, uri: &str) -> Result<(), String> {
        self.send_notification(
            "textDocument/didClose",
            json!({ "textDocument": { "uri": uri } }),
        )
    }

    // ------------------------------
    // Requests to the server
    // ------------------------------

    /// Request completion items at the given cursor position.
    /// Results arrive asynchronously via poll() → LspMessage::CompletionResponse.
    pub fn request_completion(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<(), String> {
        if self.state != ConnectionState::Ready {
            return Ok(());
        }
        let id = self.alloc_id();
        self.completion_request_id = Some(id);
        self.pending.insert(id, PendingRequest::Completion);
        self.send_request(
            id,
            "textDocument/completion",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": character },
                "context": { "triggerKind": 1 }
            }),
        )
    }

    /// Request hover documentation at the given cursor position.
    pub fn request_hover(&mut self, uri: &str, line: u32, character: u32) -> Result<(), String> {
        if self.state != ConnectionState::Ready {
            return Ok(());
        }
        let id = self.alloc_id();
        self.hover_request_id = Some(id);
        self.pending.insert(id, PendingRequest::Hover);
        self.send_request(
            id,
            "textDocument/hover",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": character }
            }),
        )
    }

    // ------------------------------
    // Polling
    // ------------------------------

    /// Drain all pending messages from the background reader thread.
    /// Call this once per tick in your main loop.
    /// Returns a list of events the rest of the app should act on.
    pub fn poll(&mut self) -> Vec<LspMessage> {
        let mut events = Vec::new();

        // Collect all available messages without blocking
        let mut raw_messages = Vec::new();
        while let Ok(msg) = self.incoming.try_recv() {
            raw_messages.push(msg);
        }

        for msg in raw_messages {
            // Server exit signal from background thread
            if msg.get("__exit").is_some() {
                self.state = ConnectionState::Failed("Server process exited".to_string());
                events.push(LspMessage::ServerExited);
                continue;
            }

            // Notification (no "id" field)
            if msg.get("id").is_none() {
                if let Some(method) = msg["method"].as_str() {
                    match method {
                        "textDocument/publishDiagnostics" => {
                            if let Some(evt) = self.handle_diagnostics(&msg) {
                                events.push(evt);
                            }
                        }
                        "window/logMessage" | "window/showMessage" => {
                            // Log server messages to our debug logger
                            if let Some(text) = msg["params"]["message"].as_str() {
                                log_info!("[LSP {}] {}", self.server_name, text);
                            }
                        }
                        _ => {} // Ignore other notifications
                    }
                }
                continue;
            }

            // Response to one of our requests (has "id" field)
            let id = match msg["id"].as_i64() {
                Some(id) => id,
                None => continue,
            };

            // Error response
            if msg.get("error").is_some() {
                let code = msg["error"]["code"].as_i64().unwrap_or(0);
                let message = msg["error"]["message"]
                    .as_str()
                    .unwrap_or("unknown error")
                    .to_string();
                log_warn!(
                    "[LSP {}] Error response id={}: {}",
                    self.server_name,
                    id,
                    message
                );
                events.push(LspMessage::Error { code, message });
                self.pending.remove(&id);
                continue;
            }

            match self.pending.remove(&id) {
                Some(PendingRequest::Initialize) => {
                    // Send "initialized" notification to complete the handshake
                    let _ = self.send_notification("initialized", json!({}));
                    self.state = ConnectionState::Ready;
                    log_info!("[LSP {}] Ready", self.server_name);
                    events.push(LspMessage::Initialized);
                }
                Some(PendingRequest::Completion) => {
                    // Only use this response if it's from the most recent request
                    if self.completion_request_id == Some(id) {
                        let items = self.parse_completion_response(&msg["result"]);
                        self.completions = items.clone();
                        events.push(LspMessage::CompletionResponse {
                            request_id: id,
                            items,
                        });
                    }
                }
                Some(PendingRequest::Hover) => {
                    if self.hover_request_id == Some(id) {
                        let result = self.parse_hover_response(&msg["result"]);
                        self.hover = result.clone();
                        events.push(LspMessage::HoverResponse {
                            request_id: id,
                            result,
                        });
                    }
                }
                None => {
                    log_warn!("[LSP {}] Response for unknown id={}", self.server_name, id);
                }
            }
        }

        events
    }

    // ------------------------------
    // Private helpers
    // ------------------------------

    /// Allocate the next id for use and increments next id
    fn alloc_id(&mut self) -> i64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Formats method and parameters to json message and sends with write_message
    fn send_notification(&mut self, method: &str, params: Value) -> Result<(), String> {
        let msg = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        self.write_message(&msg)
    }

    /// Formats id, method and parameters to json request and sends with write_message
    fn send_request(&mut self, id: i64, method: &str, params: Value) -> Result<(), String> {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });
        self.write_message(&msg)
    }

    /// Writes and flushes message to stdin of child process
    fn write_message(&mut self, msg: &Value) -> Result<(), String> {
        let body =
            serde_json::to_string(msg).map_err(|e| format!("JSON serialization failed: {}", e))?;
        write!(self.stdin, "Content-Length: {}\r\n\r\n{}", body.len(), body)
            .map_err(|e| format!("Write to LSP stdin failed: {}", e))?;
        self.stdin
            .flush()
            .map_err(|e| format!("Flush to LSP stdin failed: {}", e))?;
        Ok(())
    }

    /// Handles diagnostics by inserting block of data related to state of language system to diagnostic registry
    fn handle_diagnostics(&mut self, msg: &Value) -> Option<LspMessage> {
        let params = &msg["params"];
        let uri = params["uri"].as_str()?.to_string();
        let raw_diags = params["diagnostics"].as_array()?;

        let diagnostics: Vec<Diagnostic> = raw_diags
            .iter()
            .filter_map(|d| {
                let range = &d["range"];
                let start = &range["start"];
                let end = &range["end"];
                Some(Diagnostic {
                    line: start["line"].as_u64()? as u32,
                    col_start: start["character"].as_u64()? as u32,
                    col_end: end["character"].as_u64()? as u32,
                    severity: DiagnosticSeverity::from_lsp_int(
                        d["severity"].as_u64().unwrap_or(1) as u8
                    ),
                    message: d["message"].as_str()?.to_string(),
                    source: d["source"].as_str().map(String::from),
                })
            })
            .collect();

        self.diagnostics.insert(uri.clone(), diagnostics.clone());

        Some(LspMessage::Diagnostics { uri, diagnostics })
    }

    /// Parses completion response to item list of [CompletionItem] type
    fn parse_completion_response(&self, result: &Value) -> Vec<CompletionItem> {
        // result can be CompletionList { items: [...] } or just [...]
        let items_val = if result.is_array() {
            result
        } else {
            &result["items"]
        };

        let Some(items) = items_val.as_array() else {
            return vec![];
        };

        items
            .iter()
            .filter_map(|item| {
                let label = item["label"].as_str()?.to_string();
                Some(CompletionItem {
                    label,
                    detail: item["detail"].as_str().map(String::from),
                    documentation: item["documentation"]
                        .as_str()
                        .or_else(|| item["documentation"]["value"].as_str())
                        .map(String::from),
                    insert_text: item["insertText"].as_str().map(String::from),
                    kind: CompletionKind::from_lsp_int(item["kind"].as_u64().unwrap_or(0) as u8),
                })
            })
            .take(50) // cap at 50 items for the popup
            .collect()
    }

    /// Parses hover response to single optional [HoverResult] type
    fn parse_hover_response(&self, result: &Value) -> Option<HoverResult> {
        if result.is_null() {
            return None;
        }

        // contents can be a string, MarkedString, or MarkupContent
        let contents = if let Some(s) = result["contents"].as_str() {
            s.to_string()
        } else if let Some(s) = result["contents"]["value"].as_str() {
            s.to_string()
        } else if let Some(arr) = result["contents"].as_array() {
            arr.iter()
                .filter_map(|c| c.as_str().or_else(|| c["value"].as_str()))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            return None;
        };

        let range = result.get("range").and_then(|r| {
            Some((
                r["start"]["line"].as_u64()? as u32,
                r["start"]["character"].as_u64()? as u32,
                r["end"]["line"].as_u64()? as u32,
                r["end"]["character"].as_u64()? as u32,
            ))
        });

        Some(HoverResult { contents, range })
    }
}
