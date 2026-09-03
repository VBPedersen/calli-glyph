//! Tree-sitter grammar and LSP server installer.
//!
//! Backend for the "Install" tab in the Lang Panel: a small static
//! `registry` of known grammars/servers, a generic background `job`
//! runner, the `grammar_installer` / `lsp_installer` implementations, and
//! `InstallManager` which coordinates all of it and syncs results into
//! `Config`.
//!
//! TODO note for later: this is deliberately just data (`registry`) +
//!  actions (`start_grammar_install` / `start_lsp_install`) + status
//!  (`job_status`) on `InstallManager`, with no UI dependency baked in.
//!  A future "navigation plugin" (side modal with categories/subcategories
//!  that launches subsystems) can use this exact same API, it would just
//!  need to call `app.install_manager.start_*` and read `job_status` the
//!  same way the Lang Panel's Install tab does, so today's UI is not a
//!  dead end.

pub mod config_generator;
pub mod grammar_installer;
pub mod job;
pub mod lsp_installer;
pub mod manager;
pub mod registry;
pub mod credentials;

pub use job::{JobId, JobStatus};
pub use manager::InstallManager;
pub use registry::{GrammarSpec, LspSpec, PackageManager, GRAMMARS, LSP_SERVERS};
