//! # LSP Module
//! The LSP subsystem. Holds the lsp client that implements communication and functionality for
//! lsp's. A few LSP's come pre-installed, while allowing for user configured additions via config.

mod client;

pub use client::{
    CompletionItem, CompletionKind, ConnectionState, Diagnostic, DiagnosticSeverity, HoverResult,
    LspClient, LspMessage,
};

/// Convert a file path to a file:// URI the LSP server understands.
pub fn path_to_uri(path: &std::path::Path) -> String {
    // On Windows paths need forward slashes and a leading /
    let s = path.to_string_lossy();

    // Strip Windows UNC extended path prefix \\?\ that canonicalize() adds
    let s = s.trim_start_matches(r"\\?\");

    // Normalize backslashes to forward slashes
    let s = s.replace('\\', "/");

    // Percent-encode spaces (and other chars LSP servers encode)
    let encoded = percent_encode_path(&s);

    if encoded.starts_with('/') {
        // Unix absolute path
        format!("file://{}", encoded)
   } else {
        // Windows: lowercase the drive letter (C:/ -> c:/)
        let lower = {
            let mut chars = encoded.chars();
            match chars.next() {
                Some(first) => first.to_ascii_lowercase().to_string() + chars.as_str(),
                None => encoded,
            }
        };
        format!("file:///{}", lower)
    }
}

/// Percent-encode characters that LSP servers encode in URIs.
/// Only encodes what's necessary: letters, digits, and safe URI chars are left as-is.
fn percent_encode_path(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            // Unreserved + path chars: pass through
            'A'..='Z' | 'a'..='z' | '0'..='9'
            | '-' | '_' | '.' | '~' | '/' | ':' => c.to_string(),
            // Everything else (spaces, parens, brackets, etc.): encode
            _ => {
                let mut buf = [0u8; 4];
                let bytes = c.encode_utf8(&mut buf);
                bytes.bytes().map(|b| format!("%{:02X}", b)).collect()
            }
        })
        .collect()
}

/// Language id string for the LSP initialize/didOpen calls.
pub fn extension_to_language_id(ext: &str) -> &'static str {
    match ext {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "jsx" => "javascriptreact",
        "tsx" => "typescriptreact",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" => "cpp",
        "go" => "go",
        "lua" => "lua",
        "sh" => "shellscript",
        "json" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "md" => "markdown",
        _ => "plaintext",
    }
}
