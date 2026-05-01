use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level LSP config. Lives under [lsp] in config.toml.
///
/// Example config.toml:
/// ```toml
/// [lsp]
/// enabled = true
/// log_level = "error"
///
/// [lsp.servers.rust]
/// command = "rust-analyzer"
/// args = []
/// enabled = true
/// file_extensions = ["rs"]
/// initialization_options = {}
///
/// [lsp.servers.python]
/// command = "pyright-langserver"
/// args = ["--stdio"]
/// enabled = true
/// file_extensions = ["py"]
///
/// [lsp.servers.typescript]
/// command = "typescript-language-server"
/// args = ["--stdio"]
/// enabled = true
/// file_extensions = ["ts", "js", "tsx", "jsx"]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LspConfig {
    /// To enable lsp function or not: set false to disable all LSP globally
    pub enabled: bool,

    /// How long to wait for the server to respond before aborting (ms)
    pub request_timeout_ms: u64,

    /// Per-language server configurations, key is just a name chosen by user
    /// e.g. "rust", "python".
    pub servers: HashMap<String, LspServerConfig>,
}

/// Default impl for LSP config, comes with rust, python and TypeScript setup be default,
/// however does require the defined lsp's to  be installed on system
impl Default for LspConfig {
    fn default() -> Self {
        let mut servers = HashMap::new();

        servers.insert(
            "rust".to_string(),
            LspServerConfig {
                command: "rust-analyzer".to_string(),
                args: vec![],
                enabled: true,
                file_extensions: vec!["rs".to_string()],
                initialization_options: HashMap::new(),
            },
        );

        servers.insert(
            "python".to_string(),
            LspServerConfig {
                command: "pyright-langserver".to_string(),
                args: vec!["--stdio".to_string()],
                enabled: false, // off by default — user opts in
                file_extensions: vec!["py".to_string()],
                initialization_options: HashMap::new(),
            },
        );

        servers.insert(
            "typescript".to_string(),
            LspServerConfig {
                command: "typescript-language-server".to_string(),
                args: vec!["--stdio".to_string()],
                enabled: false,
                file_extensions: vec![
                    "ts".to_string(),
                    "js".to_string(),
                    "tsx".to_string(),
                    "jsx".to_string(),
                ],
                initialization_options: HashMap::new(),
            },
        );

        Self {
            enabled: true,
            request_timeout_ms: 5000,
            servers,
        }
    }
}

impl LspConfig {
    /// Find the server config for a given file extension.
    /// Returns the server name and its config.
    pub fn server_for_extension(&self, ext: &str) -> Option<(&str, &LspServerConfig)> {
        if !self.enabled {
            return None;
        }
        self.servers
            .iter()
            .find(|(_, cfg)| cfg.enabled && cfg.file_extensions.iter().any(|e| e == ext))
            .map(|(name, cfg)| (name.as_str(), cfg))
    }
}

/// Configuration for a single LSP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LspServerConfig {
    /// The executable to run, e.g. "rust-analyzer"
    pub command: String,

    /// Arguments passed to the server process
    pub args: Vec<String>,

    /// Is server active?
    pub enabled: bool,

    /// Which file extensions this server handles
    pub file_extensions: Vec<String>,

    /// Passed as initializationOptions in the LSP initialize request.
    /// Allows per server settings without knowing them at compile time.
    pub initialization_options: HashMap<String, serde_json::Value>,
}

impl Default for LspServerConfig {
    fn default() -> Self {
        Self {
            command: String::new(),
            args: vec![],
            enabled: false,
            file_extensions: vec![],
            initialization_options: HashMap::new(),
        }
    }
}
