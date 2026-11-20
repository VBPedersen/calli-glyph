---
id: menu
title: Main Menu
summary: One place to reach the app's features: settings, themes, controls, file picker and more
tags: menu, :menu, Ctrl+P, main menu, categories, shortcuts, quit, plugins, entries
---

# Main Menu

The main menu collects the app's features in one place. Entries are grouped
into categories shown as tabs, and every entry can have a one-key shortcut.

## Opening the menu

| Trigger          | Action                                                    |
|------------------|-----------------------------------------------------------|
| `Ctrl+P`         | Open the menu (default shortcut, rebind in `:controls`)   |
| `:menu`          | Open the menu from the command line                       |
| `m`              | Open the menu from the welcome screen                     |

## Navigation

| Key                 | Action                                                  |
|---------------------|---------------------------------------------------------|
| `Tab` / `Shift+Tab` | Next / previous category                                |
| `↑` / `k`           | Move up                                                 |
| `↓` / `j`           | Move down                                               |
| `Enter`             | Run the selected entry                                  |
| Shortcut letter     | Run an entry directly, shown as `[x]` on its row        |
| `Esc`               | Close the menu                                          |

The description of the selected entry is shown below the list. Shortcut
letters work from any category, not only the one you are looking at.

The menu closes after an entry runs. If the entry opens another screen, such
as the settings, that screen takes its place.

## Built-in entries

| Category   | Entry              | Key | What it does                                  |
|------------|--------------------|-----|-----------------------------------------------|
| Project    | Browse files       | `f` | Opens the file picker, see `:help project`    |
| Appearance | Themes             | `t` | Browse, preview and apply themes              |
| Appearance | Edit current theme | `e` | Change colors and layout of the active theme  |
| Appearance | New theme          | `n` | Create a theme as a copy of the active one    |
| Language   | GitHub token       | `g` | Set or remove the token for grammar installs  |
| Settings   | Settings           | `s` | Change common options, see `:help settings`   |
| Settings   | Controls           | `c` | Rebind keyboard shortcuts, see `:help controls` |
| App        | Quit               | `q` | Quit, asks to confirm if there are unsaved changes |

Other plugins, such as the language installer or the debugger, add their own
entries. Category tabs appear in the order they were registered.

## Entries depend on plugins

An entry that runs a plugin command is only shown while that plugin is
enabled. If a plugin is turned off in your config, its entries disappear from
the menu instead of failing when selected.

## Adding entries from a plugin

A plugin lists its menu entries by implementing `menu_entries`:

```rust
fn menu_entries(&self) -> Vec<MenuEntry> {
    vec![MenuEntry::command(
        "Project", "Browse files", "Open the file picker", "explore",
    )
    .with_key('f')]
}
```

* `MenuEntry::command(category, title, description, command)` runs a plugin command
* `MenuEntry::callback(category, title, description, function)` runs a function directly
* `.with_key('x')` sets the shortcut letter. Avoid `j` and `k`, they scroll the list
* An entry with the same category and title as an existing one replaces it
