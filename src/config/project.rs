//! Project / file-picker configuration. Lives under `[project]` in
//! config.toml.
//!
//! The goal here is specifically to make it easy to add a new external
//! picker. each one is a `[project.file_picker.external.<name>]` table:
//! a command, an arg list, and how to read back what was chosen, with no
//! picker-specific code anywhere else. `yazi`/`ranger`/`nnn` ship as
//! defaults. Adding `broot` or anything else a user already has installed
//! is just another TOML table, same as adding an LSP server was in
//! `src/language/install/registry.rs`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProjectConfig {
    pub file_picker: FilePickerConfig,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            file_picker: FilePickerConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct FilePickerConfig {
    pub mode: PickerMode,
    /// Which entry in `external` to use when `mode = "external"`.
    pub active_external: Option<String>,
    /// Registry of external pickers. Keyed by name (matches `active_external`).
    pub external: HashMap<String, ExternalPickerSpec>,
}

impl Default for FilePickerConfig {
    fn default() -> Self {
        let mut external = HashMap::new();

        external.insert(
            "yazi".to_string(),
            ExternalPickerSpec {
                command: "yazi".to_string(),
                args: vec![
                    "{dir}".to_string(),
                    "--chooser-file".to_string(),
                    "{output}".to_string(),
                ],
                output_mode: OutputMode::OutputFile,
            },
        );

        external.insert(
            "ranger".to_string(),
            ExternalPickerSpec {
                command: "ranger".to_string(),
                args: vec!["--choosefile={output}".to_string(), "{dir}".to_string()],
                output_mode: OutputMode::OutputFile,
            },
        );

        external.insert(
            "nnn".to_string(),
            ExternalPickerSpec {
                command: "nnn".to_string(),
                // nnn's -p prints the picked path(s) to FILE on quit
                args: vec![
                    "-p".to_string(),
                    "{output}".to_string(),
                    "{dir}".to_string(),
                ],
                output_mode: OutputMode::OutputFile,
            },
        );

        Self {
            mode: PickerMode::Builtin,
            active_external: Some("yazi".to_string()),
            external,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum PickerMode {
    #[default]
    Builtin,
    External,
}

/// How to retrieve the file the user picked once the external process exits.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    /// The command writes the chosen path to the file passed via
    /// `{output}` in its args (yazi's `--chooser-file`, ranger's
    /// `--choosefile`, nnn's `-p`).
    #[default]
    OutputFile,
    /// The command prints the chosen path to stdout on exit. Only works
    /// for tools that draw their interactive UI directly to the terminal
    /// (e.g. via `/dev/tty`) rather than through stdout, since this mode
    /// captures stdout instead of letting the child inherit it.
    Stdout,
}

/// A single external picker's command template. `{dir}` and `{output}`
/// are substituted into `args` at launch time: `{dir}` with the
/// directory to start browsing in, `{output}` with a fresh temp file path
/// (only meaningful for `OutputMode::OutputFile`).
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ExternalPickerSpec {
    pub command: String,
    pub args: Vec<String>,
    pub output_mode: OutputMode,
}

#[cfg(test)]
mod unit_project_config_tests {
    use super::*;

    #[test]
    fn default_mode_is_builtin() {
        assert_eq!(FilePickerConfig::default().mode, PickerMode::Builtin);
    }

    #[test]
    fn default_registry_includes_yazi_ranger_and_nnn() {
        let config = FilePickerConfig::default();
        assert!(config.external.contains_key("yazi"));
        assert!(config.external.contains_key("ranger"));
        assert!(config.external.contains_key("nnn"));
    }

    #[test]
    fn default_active_external_points_at_a_registered_picker() {
        let config = FilePickerConfig::default();
        let active = config.active_external.as_deref().unwrap();
        assert!(config.external.contains_key(active));
    }

    #[test]
    fn every_default_picker_args_reference_output_placeholder() {
        let config = FilePickerConfig::default();
        for (name, spec) in &config.external {
            if spec.output_mode == OutputMode::OutputFile {
                assert!(
                    spec.args.iter().any(|a| a.contains("{output}")),
                    "picker '{}' uses OutputFile mode but no arg references {{output}}",
                    name
                );
            }
        }
    }

    #[test]
    fn picker_mode_deserializes_from_lowercase_toml() {
        let fpc: FilePickerConfig = toml::from_str("mode = \"external\"").unwrap();
        assert_eq!(fpc.mode, PickerMode::External);

        let fpc: FilePickerConfig = toml::from_str("mode = \"builtin\"").unwrap();
        assert_eq!(fpc.mode, PickerMode::Builtin);
    }

    #[test]
    fn output_mode_deserializes_from_snake_case_toml() {
        let ext_p_s: ExternalPickerSpec = toml::from_str("output_mode = \"output_file\"").unwrap();
        assert_eq!(ext_p_s.output_mode, OutputMode::OutputFile);
        let ext_p_s: ExternalPickerSpec = toml::from_str("output_mode = \"stdout\"").unwrap();
        assert_eq!(ext_p_s.output_mode, OutputMode::Stdout);
    }

    #[test]
    fn user_can_add_a_custom_picker_via_toml_alone() {
        // The entire point of this registry: adding a new picker is pure
        // data, no code. This confirms a config file can define one that
        // isn't in the built-in defaults.
        let toml_str = r#"
            mode = "external"
            active_external = "broot"

            [external.broot]
            command = "broot"
            args = ["{dir}"]
            output_mode = "stdout"
        "#;
        let config: FilePickerConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.active_external.as_deref(), Some("broot"));
        let broot = config.external.get("broot").unwrap();
        assert_eq!(broot.command, "broot");
        assert_eq!(broot.output_mode, OutputMode::Stdout);
    }
}
