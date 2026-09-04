//! Static registry of installable tree-sitter grammars and LSP servers.
//!
//! This is the catalog shown in the Lang Panel's "Install" tab. Adding a
//! language just means adding an entry here — nothing else needs to change.

/// Describes where to fetch and how to build a tree-sitter grammar.
#[derive(Debug, Clone, Copy)]
pub struct GrammarSpec {
    /// Name used everywhere else in the app (matches `SyntaxLanguageConfig.grammar`)
    pub name: &'static str,
    /// git URL of the grammar repository
    pub repo_url: &'static str,
    /// Subdirectory inside the repo containing `src/parser.c`, for monorepos
    /// like tree-sitter-typescript (which has `typescript/` and `tsx/`)
    pub subdir: Option<&'static str>,
    /// File extensions this language should be associated with in config
    pub file_extensions: &'static [&'static str],
}

pub const GRAMMARS: &[GrammarSpec] = &[
    GrammarSpec {
        name: "rust",
        repo_url: "https://github.com/tree-sitter/tree-sitter-rust",
        subdir: None,
        file_extensions: &["rs"],
    },
    GrammarSpec {
        name: "python",
        repo_url: "https://github.com/tree-sitter/tree-sitter-python",
        subdir: None,
        file_extensions: &["py"],
    },
    GrammarSpec {
        name: "typescript",
        repo_url: "https://github.com/tree-sitter/tree-sitter-typescript",
        subdir: Some("typescript"),
        file_extensions: &["ts"],
    },
    GrammarSpec {
        name: "tsx",
        repo_url: "https://github.com/tree-sitter/tree-sitter-typescript",
        subdir: Some("tsx"),
        file_extensions: &["tsx"],
    },
    GrammarSpec {
        name: "javascript",
        repo_url: "https://github.com/tree-sitter/tree-sitter-javascript",
        subdir: None,
        file_extensions: &["js", "jsx", "mjs"],
    },
    GrammarSpec {
        name: "go",
        repo_url: "https://github.com/tree-sitter/tree-sitter-go",
        subdir: None,
        file_extensions: &["go"],
    },
    GrammarSpec {
        name: "c",
        repo_url: "https://github.com/tree-sitter/tree-sitter-c",
        subdir: None,
        file_extensions: &["c", "h"],
    },
    GrammarSpec {
        name: "cpp",
        repo_url: "https://github.com/tree-sitter/tree-sitter-cpp",
        subdir: None,
        file_extensions: &["cpp", "cc", "hpp"],
    },
    GrammarSpec {
        name: "json",
        repo_url: "https://github.com/tree-sitter/tree-sitter-json",
        subdir: None,
        file_extensions: &["json"],
    },
    GrammarSpec {
        name: "bash",
        repo_url: "https://github.com/tree-sitter/tree-sitter-bash",
        subdir: None,
        file_extensions: &["sh", "bash"],
    },
    GrammarSpec {
        name: "html",
        repo_url: "https://github.com/tree-sitter/tree-sitter-html",
        subdir: None,
        file_extensions: &["html"],
    },
    GrammarSpec {
        name: "css",
        repo_url: "https://github.com/tree-sitter/tree-sitter-css",
        subdir: None,
        file_extensions: &["css"],
    },
    GrammarSpec {
        name: "toml",
        repo_url: "https://github.com/tree-sitter-grammars/tree-sitter-toml",
        subdir: None,
        file_extensions: &["toml"],
    },
    GrammarSpec {
        name: "yaml",
        repo_url: "https://github.com/tree-sitter-grammars/tree-sitter-yaml",
        subdir: None,
        file_extensions: &["yaml", "yml"],
    },
];

/// Package manager used to auto-install an LSP server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Npm,
    Pip,
    Cargo,
    Go,
}

impl PackageManager {
    /// Binary name to check for on PATH / invoke.
    pub fn binary(self) -> &'static str {
        match self {
            PackageManager::Npm => "npm",
            PackageManager::Pip => "pip3",
            PackageManager::Cargo => "cargo",
            PackageManager::Go => "go",
        }
    }

    /// Args used to globally install `packages` with this package manager.
    /// Most specs have a single package name; a few (e.g. the TS language
    /// server, which needs both `typescript-language-server` and
    /// `typescript`) list more than one.
    pub fn install_args(self, packages: &[&str]) -> Vec<String> {
        match self {
            PackageManager::Npm => {
                let mut args = vec!["install".to_string(), "-g".to_string()];
                args.extend(packages.iter().map(|s| s.to_string()));
                args
            }
            PackageManager::Pip => {
                let mut args = vec!["install".to_string(), "--user".to_string()];
                args.extend(packages.iter().map(|s| s.to_string()));
                args
            }
            PackageManager::Cargo => {
                let mut args = vec!["install".to_string()];
                args.extend(packages.iter().map(|s| s.to_string()));
                args
            }
            PackageManager::Go => {
                let mut args = vec!["install".to_string()];
                args.extend(packages.iter().map(|p| format!("{}@latest", p)));
                args
            }
        }
    }
}

/// Describes an installable LSP server.
#[derive(Debug, Clone, Copy)]
pub struct LspSpec {
    /// Key used in config.toml under `[lsp.servers.<name>]`
    pub name: &'static str,
    /// How to auto-install it, if there's a reliable path. `None` means
    /// "always show manual instructions" (e.g. rust-analyzer, which isn't
    /// reliably installable via a package manager).
    pub package_manager: Option<PackageManager>,
    /// Package name(s) passed to the package manager, space separated
    pub package: &'static str,
    /// The command to run once installed (resolved via PATH)
    pub command: &'static str,
    pub args: &'static [&'static str],
    pub file_extensions: &'static [&'static str],
    pub root_markers: &'static [&'static str],
    /// Shown to the user when auto-install isn't available or fails
    pub manual_instructions: &'static str,
}

pub const LSP_SERVERS: &[LspSpec] = &[
    LspSpec {
        name: "rust",
        package_manager: None,
        package: "rust-analyzer",
        command: "rust-analyzer",
        args: &[],
        file_extensions: &["rs"],
        root_markers: &[".git", "Cargo.toml"],
        manual_instructions: "Install via `rustup component add rust-analyzer`, or download a release binary from https://github.com/rust-lang/rust-analyzer/releases and put it on your PATH.",
    },
    LspSpec {
        name: "python",
        package_manager: Some(PackageManager::Pip),
        package: "pyright",
        command: "pyright-langserver",
        args: &["--stdio"],
        file_extensions: &["py"],
        root_markers: &[".git"],
        manual_instructions: "Install via `pip install --user pyright`, or `npm install -g pyright`.",
    },
    LspSpec {
        name: "typescript",
        package_manager: Some(PackageManager::Npm),
        package: "typescript-language-server typescript",
        command: "typescript-language-server",
        args: &["--stdio"],
        file_extensions: &["ts", "tsx", "js", "jsx"],
        root_markers: &[".git", "package.json"],
        manual_instructions: "Install via `npm install -g typescript-language-server typescript`.",
    },
    LspSpec {
        name: "go",
        package_manager: Some(PackageManager::Go),
        package: "golang.org/x/tools/gopls",
        command: "gopls",
        args: &[],
        file_extensions: &["go"],
        root_markers: &[".git", "go.mod"],
        manual_instructions: "Install via `go install golang.org/x/tools/gopls@latest`.",
    },
    LspSpec {
        name: "bash",
        package_manager: Some(PackageManager::Npm),
        package: "bash-language-server",
        command: "bash-language-server",
        args: &["start"],
        file_extensions: &["sh", "bash"],
        root_markers: &[".git"],
        manual_instructions: "Install via `npm install -g bash-language-server`.",
    },
    LspSpec {
        name: "json",
        package_manager: Some(PackageManager::Npm),
        package: "vscode-langservers-extracted",
        command: "vscode-json-language-server",
        args: &["--stdio"],
        file_extensions: &["json"],
        root_markers: &[".git"],
        manual_instructions: "Install via `npm install -g vscode-langservers-extracted`.",
    },
];

//████████╗███████╗███████╗████████╗███████╗
//╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//   ██║   █████╗  ███████╗   ██║   ███████╗
//   ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//   ██║   ███████╗███████║   ██║   ███████║
//   ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝

#[cfg(test)]
mod unit_registry_tests {
    use super::*;

    #[test]
    fn grammar_catalog_is_non_empty_and_has_unique_names() {
        assert!(!GRAMMARS.is_empty());
        let mut names: Vec<&str> = GRAMMARS.iter().map(|g| g.name).collect();
        let total = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), total, "duplicate grammar names in registry");
    }

    #[test]
    fn lsp_catalog_is_non_empty_and_has_unique_names() {
        assert!(!LSP_SERVERS.is_empty());
        let mut names: Vec<&str> = LSP_SERVERS.iter().map(|s| s.name).collect();
        let total = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), total, "duplicate lsp server names in registry");
    }

    #[test]
    fn every_grammar_has_at_least_one_file_extension() {
        for spec in GRAMMARS {
            assert!(
                !spec.file_extensions.is_empty(),
                "grammar '{}' has no file extensions",
                spec.name
            );
        }
    }

    #[test]
    fn every_lsp_server_has_at_least_one_file_extension() {
        for spec in LSP_SERVERS {
            assert!(
                !spec.file_extensions.is_empty(),
                "lsp server '{}' has no file extensions",
                spec.name
            );
        }
    }

    #[test]
    fn npm_install_args_include_global_flag_and_all_packages() {
        let args = PackageManager::Npm.install_args(&["typescript-language-server", "typescript"]);
        assert_eq!(
            args,
            vec!["install", "-g", "typescript-language-server", "typescript"]
        );
    }

    #[test]
    fn pip_install_args_use_user_flag() {
        let args = PackageManager::Pip.install_args(&["pyright"]);
        assert_eq!(args, vec!["install", "--user", "pyright"]);
    }

    #[test]
    fn cargo_install_args_are_plain() {
        let args = PackageManager::Cargo.install_args(&["ripgrep"]);
        assert_eq!(args, vec!["install", "ripgrep"]);
    }

    #[test]
    fn go_install_args_append_latest_tag() {
        let args = PackageManager::Go.install_args(&["golang.org/x/tools/gopls"]);
        assert_eq!(args, vec!["install", "golang.org/x/tools/gopls@latest"]);
    }

    #[test]
    fn go_install_args_handle_multiple_packages() {
        let args = PackageManager::Go.install_args(&["foo", "bar"]);
        assert_eq!(args, vec!["install", "foo@latest", "bar@latest"]);
    }

    #[test]
    fn package_manager_binaries_are_distinct() {
        let mut bins = vec![
            PackageManager::Npm.binary(),
            PackageManager::Pip.binary(),
            PackageManager::Cargo.binary(),
            PackageManager::Go.binary(),
        ];
        let total = bins.len();
        bins.sort_unstable();
        bins.dedup();
        assert_eq!(bins.len(), total);
    }
}
