//! # Language Module
//!
//! The `language` module is the logic of the editor's language understanding. It defines
//! how source code is parsed, highlighted, and analyzed.
//!
//! ## Architecture Overview
//!
//! 1. **Syntax (`syntax`)**: The **Tree-sitter** wrapper. Uses **Tree-sitter** for incremental parsing and high-performance
//!    syntax highlighting.
//! 2. **LSP (`lsp`)**: Manages Language Server Protocol clients, providing features
//!    like go-to-definition, rename, and diagnostics.
//! 3. **Manager (`manager`)**: The orchestrator. It ties the buffer to the correct
//!    language server and syntax parser based on file types.
//! 4. **Theme (`theme`)**: Maps Tree-sitter scopes and LSP semantic tokens to
//!    actual TUI colors/styles.
//!
//! ## Common Workflows
//!
//! ### Registering a New Language
//! TODO EXPLAIN
//!
//! ```rust
//! ```

pub mod syntax;
pub mod theme;
// pub mod lsp;
pub mod manager;
