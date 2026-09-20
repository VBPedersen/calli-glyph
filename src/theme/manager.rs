//! `ThemeManager` is the single consolidation point the rest of the app
//! pulls theme data from — the "individual files can pull info from a
//! unified place" piece. It owns every loaded theme, which one is active,
//! and where to (re)load from; nothing else should be parsing theme TOML
//! or hardcoding a color.

use super::{Theme, EMBEDDED_DARK, EMBEDDED_DRACULA, EMBEDDED_LIGHT};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct ThemeManager {
    /// Keyed by theme name — the file stem for disk-loaded themes (e.g.
    /// `dracula.toml` -> `"dracula"`), or `"dark"`/`"light"` for the
    /// embedded fallbacks before any file overrides them.
    themes: HashMap<String, Theme>,
    active_name: String,
    themes_dir: PathBuf,
}

impl ThemeManager {
    /// Builds the manager: seeds the embedded fallbacks, writes them to
    /// `themes_dir` on first launch if nothing's there yet (so a user has
    /// something to copy and edit, same convention as grammar lang
    /// configs), scans `themes_dir` for `.toml` files, and resolves
    /// `active_name` — falling back to `"dark"` with a warning if the
    /// configured name isn't found among what was loaded.
    pub fn new(themes_dir: PathBuf, active_name: &str) -> Self {
        let mut themes = HashMap::new();
        themes.insert("dark".to_string(), Theme::embedded_dark());
        themes.insert("light".to_string(), Theme::embedded_light());
        themes.insert("dracula".to_string(), Theme::embedded_dracula());

        ensure_default_themes_written(&themes_dir);
        scan_dir(&themes_dir, &mut themes);

        let resolved_active = if themes.contains_key(active_name) {
            active_name.to_string()
        } else {
            log_warn!(
                "[Theme] Configured theme '{}' not found among loaded themes, falling back to 'dark'",
                active_name
            );
            "dark".to_string()
        };

        Self {
            themes,
            active_name: resolved_active,
            themes_dir,
        }
    }

    /// The currently active theme. Always returns something — never a
    /// render-blocking `Option` — since the embedded "dark" theme is
    /// guaranteed present.
    pub fn active(&self) -> &Theme {
        self.themes
            .get(&self.active_name)
            .or_else(|| self.themes.get("dark"))
            .expect("embedded 'dark' theme is always present")
    }

    pub fn active_name(&self) -> &str {
        &self.active_name
    }

    /// Switches the active theme by name. Fails without side effects if
    /// `name` isn't loaded — callers (e.g. the theme picker) should use
    /// `names()` to only ever offer names that will succeed here.
    pub fn set_active(&mut self, name: &str) -> Result<(), String> {
        if self.themes.contains_key(name) {
            self.active_name = name.to_string();
            Ok(())
        } else {
            Err(format!("No theme named '{}' is loaded", name))
        }
    }

    /// All loaded theme names, sorted, for populating a picker.
    pub fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.themes.keys().map(|s| s.as_str()).collect();
        names.sort_unstable();
        names
    }

    pub fn get(&self, name: &str) -> Option<&Theme> {
        self.themes.get(name)
    }

    /// Re-scans `themes_dir`, picking up new/edited `.toml` files without
    /// restarting the app. Embedded fallbacks are never removed by this —
    /// only added to or overridden.
    pub fn rescan(&mut self) {
        scan_dir(&self.themes_dir, &mut self.themes);
    }
}

fn ensure_default_themes_written(dir: &Path) {
    if std::fs::create_dir_all(dir).is_err() {
        return;
    }
    write_if_missing(&dir.join("dark.toml"), EMBEDDED_DARK);
    write_if_missing(&dir.join("light.toml"), EMBEDDED_LIGHT);
    write_if_missing(&dir.join("dracula.toml"), EMBEDDED_DRACULA);
}

fn write_if_missing(path: &Path, content: &str) {
    if !path.exists() {
        if let Err(e) = std::fs::write(path, content) {
            log_warn!("[Theme] Failed to write default theme {}: {}", path.display(), e);
        }
    }
}

fn scan_dir(dir: &Path, themes: &mut HashMap<String, Theme>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };

        match Theme::from_file(&path) {
            Ok(theme) => {
                themes.insert(stem.to_string(), theme);
            }
            Err(e) => {
                log_warn!("[Theme] Failed to load {}: {}", path.display(), e);
            }
        }
    }
}

#[cfg(test)]
mod unit_manager_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn new_always_has_embedded_fallbacks_even_with_empty_dir() {
        let dir = tempdir().unwrap();
        let manager = ThemeManager::new(dir.path().join("themes"), "dark");
        assert!(manager.names().contains(&"dark"));
        assert!(manager.names().contains(&"light"));
    }

    #[test]
    fn new_writes_default_theme_files_on_first_launch() {
        let dir = tempdir().unwrap();
        let themes_dir = dir.path().join("themes");
        let _manager = ThemeManager::new(themes_dir.clone(), "dark");

        assert!(themes_dir.join("dark.toml").exists());
        assert!(themes_dir.join("light.toml").exists());
    }

    #[test]
    fn new_does_not_overwrite_an_existing_dark_toml() {
        let dir = tempdir().unwrap();
        let themes_dir = dir.path().join("themes");
        std::fs::create_dir_all(&themes_dir).unwrap();
        std::fs::write(themes_dir.join("dark.toml"), "[meta]\nname = \"Hand Edited\"\n").unwrap();

        let manager = ThemeManager::new(themes_dir, "dark");
        assert_eq!(manager.active().meta.name, "Hand Edited");
    }

    #[test]
    fn new_falls_back_to_dark_when_configured_theme_missing() {
        let dir = tempdir().unwrap();
        let manager = ThemeManager::new(dir.path().join("themes"), "does-not-exist");
        assert_eq!(manager.active_name(), "dark");
    }

    #[test]
    fn scan_dir_picks_up_user_added_theme_file() {
        let dir = tempdir().unwrap();
        let themes_dir = dir.path().join("themes");
        std::fs::create_dir_all(&themes_dir).unwrap();
        std::fs::write(
            themes_dir.join("custom.toml"),
            "[meta]\nname = \"Custom\"\n",
        )
            .unwrap();

        let manager = ThemeManager::new(themes_dir, "dark");
        assert!(manager.names().contains(&"custom"));
        assert_eq!(manager.get("custom").unwrap().meta.name, "Custom");
    }

    #[test]
    fn set_active_switches_theme() {
        let dir = tempdir().unwrap();
        let mut manager = ThemeManager::new(dir.path().join("themes"), "dark");
        assert_eq!(manager.active_name(), "dark");

        manager.set_active("light").unwrap();
        assert_eq!(manager.active_name(), "light");
        assert_eq!(manager.active().meta.name, "Light");
    }

    #[test]
    fn set_active_fails_for_unknown_theme_without_changing_state() {
        let dir = tempdir().unwrap();
        let mut manager = ThemeManager::new(dir.path().join("themes"), "dark");

        let result = manager.set_active("nonexistent");
        assert!(result.is_err());
        assert_eq!(manager.active_name(), "dark"); // unchanged
    }

    #[test]
    fn rescan_picks_up_a_file_added_after_construction() {
        let dir = tempdir().unwrap();
        let themes_dir = dir.path().join("themes");
        let mut manager = ThemeManager::new(themes_dir.clone(), "dark");
        assert!(!manager.names().contains(&"latecomer"));

        std::fs::write(
            themes_dir.join("latecomer.toml"),
            "[meta]\nname = \"Latecomer\"\n",
        )
            .unwrap();
        manager.rescan();

        assert!(manager.names().contains(&"latecomer"));
    }

    #[test]
    fn names_are_sorted() {
        let dir = tempdir().unwrap();
        let manager = ThemeManager::new(dir.path().join("themes"), "dark");
        let names = manager.names();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted);
    }
}