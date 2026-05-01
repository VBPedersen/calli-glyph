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
    let s = path.to_string_lossy().replace('\\', "/");
    if s.starts_with('/') {
        format!("file://{}", s)
    } else {
        format!("file:///{}", s)
    }
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
