//! Central registry of user-facing "things you can do". The main menu
//! TODO (and possibly later a command palette / Lua)
//! are views over this, so adding a feature
//! to the menu never means touching the menu UI.

use crate::core::app::App;

#[derive(Clone)]
pub enum MenuAction {
    /// Dispatched through the plugin command registry (`app.execute_plugin_command`).
    Command(String),
    /// For built-ins that aren't plugin commands.
    Callback(fn(&mut App)),
}

#[derive(Clone)]
pub struct MenuEntry {
    pub category: String,
    pub title: String,
    pub description: String,
    /// Optional hotkey inside the menu.
    pub key: Option<char>,
    pub action: MenuAction,
}

impl MenuEntry {
    pub fn command(category: &str, title: &str, description: &str, command: &str) -> Self {
        Self {
            category: category.into(),
            title: title.into(),
            description: description.into(),
            key: None,
            action: MenuAction::Command(command.into()),
        }
    }

    pub fn callback(category: &str, title: &str, description: &str, f: fn(&mut App)) -> Self {
        Self {
            category: category.into(),
            title: title.into(),
            description: description.into(),
            key: None,
            action: MenuAction::Callback(f),
        }
    }

    pub fn with_key(mut self, key: char) -> Self {
        debug_assert!(!matches!(key, 'j' | 'k' | '\0'), "reserved menu key");
        self.key = Some(key);
        self
    }
}

#[derive(Default)]
pub struct MenuRegistry {
    entries: Vec<MenuEntry>,
}

impl MenuRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an entry. An existing one with the same category + title is replaced
    pub fn register(&mut self, entry: MenuEntry) {
        match self
            .entries
            .iter_mut()
            .find(|e| e.category == entry.category && e.title == entry.title)
        {
            Some(existing) => *existing = entry,
            None => self.entries.push(entry),
        }
    }

    pub fn entries(&self) -> &[MenuEntry] {
        &self.entries
    }

    /// Categories in order of first appearance.
    pub fn categories(&self) -> Vec<String> {
        categories_of(&self.entries)
    }
}

/// Returns vec of strings containing, categories from array of menu entries
/// without duplicates.
fn categories_of(entries: &[MenuEntry]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for e in entries {
        if !out.contains(&e.category) {
            out.push(e.category.clone());
        }
    }
    out
}

/// Built-in entries. Fill the TODOs with your real command names/callbacks.
pub fn register_builtin_entries(menu: &mut MenuRegistry) {
    menu.register(
        MenuEntry::command("Project", "Browse files", "Open the file picker", "explore")
            .with_key('f'),
    );
    menu.register(MenuEntry::command("Appearance", "Theme picker", "Switch the color theme", "theme").with_key('t'));
    menu.register(MenuEntry::command("Language", "Language installer", "Install LSP servers", "lang").with_key('i'));
    // TODO: register debugger, via a Callback or its command
    // TODO: register settings
}

#[cfg(test)]
mod unit_menu_registry_tests {
    use super::*;

    fn cmd(cat: &str, title: &str) -> MenuEntry {
        MenuEntry::command(cat, title, "", "x")
    }

    #[test]
    fn register_replaces_same_category_and_title() {
        let mut m = MenuRegistry::new();
        m.register(cmd("A", "One"));
        m.register(MenuEntry::command("A", "One", "new desc", "y"));
        assert_eq!(m.entries().len(), 1);
        assert_eq!(m.entries()[0].description, "new desc");
    }

    #[test]
    fn same_title_in_different_category_is_kept() {
        let mut m = MenuRegistry::new();
        m.register(cmd("A", "One"));
        m.register(cmd("B", "One"));
        assert_eq!(m.entries().len(), 2);
    }

    #[test]
    fn categories_keep_first_appearance_order_without_duplicates() {
        let mut m = MenuRegistry::new();
        m.register(cmd("Tools", "a"));
        m.register(cmd("Project", "b"));
        m.register(cmd("Tools", "c"));
        assert_eq!(m.categories(), vec!["Tools", "Project"]);
    }
}
