---
id: config
title: Configuration
summary: Customize editor behaviour, appearance, performance, and keybindings
tags: config, configuration, settings, keybindings, theme, :config, reload, tab_width, line_numbers, scrolloff, auto_save, file_picker, project
---

# Configuration

calli-glyph is configured via a `config.toml` file stored in your system
config directory (typically `~/.config/calliglyph/config.toml`). If no file is
found, a default one is created.

You do not have to edit the file by hand: common options can be changed in
the settings screen (`:help settings`), keyboard shortcuts in the controls
screen (`:help controls`), and colors in the theme editor
(`:help themeeditor`). All three save to your config and keep your comments.

Changes made in those screens apply immediately (a few options say they need a
restart). Changes you make to the file yourself are applied on startup, or
with `:config reload`.

## Commands

| Command          | Description                                      |
|------------------|--------------------------------------------------|
| `:config`        | Open the configuration overview                  |
| `:config reload` | Reload config from disk without restarting       |
| `:settings`      | Open the settings screen                         |
| `:controls`      | Open the keybinding editor                       |

## Editor Options

Configured under `[editor]` in your config file.

| Key                    | Type    | Default | Description                              |
|------------------------|---------|---------|------------------------------------------|
| `tab_width`            | number  | `4`     | Width of a tab character in columns      |
| `use_spaces`           | bool    | `false` | Insert spaces instead of tab characters  |
| `line_numbers`         | bool    | `true`  | Show line numbers in the gutter          |
| `relative_line_numbers`| bool    | `true`  | Show line numbers relative to cursor     |
| `wrap_lines`           | bool    | `false` | Wrap long lines instead of scrolling     |
| `auto_save`            | bool    | `false` | Automatically save on edit               |
| `auto_save_delay_ms`   | number  | `1000`  | Delay before auto-saving (milliseconds)  |
| `show_whitespace`      | bool    | `false` | Render whitespace characters visibly     |
| `highlight_current_line`| bool   | `false` | Highlight the line the cursor is on      |
| `scrolloff`            | number  | `3`     | Lines kept visible above and below cursor|
| `scroll_lines`         | number  | `1`     | Lines scrolled per mouse wheel tick      |
| `scroll_margin_bottom` | number  | `5`     | Empty lines kept at bottom when scrolling|
| `undo_history_limit`   | number  | `1000`  | Maximum number of undo steps stored      |

## UI Options

Configured under `[ui]`.

| Key               | Values                      | Default   | Description                     |
|-------------------|-----------------------------|-----------|---------------------------------|
| `theme`           | a theme name, e.g. `"dark"` | `dark`    | Active theme, see `:help themes`|
| `show_status_bar` | `true`, `false`             | `true`    | Show the status bar             |
| `cursor_style`    | `block`, `line`, `underline`| `block`   | Visual style of the cursor      |
| `cursor_blink`    | `true`, `false`             | `true`    | Whether the cursor blinks       |

## Performance Options

Configured under `[performance]`.

| Key                    | Type   | Default | Description                               |
|------------------------|--------|---------|-------------------------------------------|
| `tick_rate_ms`         | number | `50`    | Editor event loop tick rate (milliseconds)|
| `cursor_blink_rate_ms` | number | `500`   | Cursor blink interval (milliseconds)      |
| `lazy_redraw`          | bool   | `false` | Only redraw when input is received        |

## Project Options

Configured under `[project.file_picker]`. See `:help project`.

| Key               | Values                    | Default     | Description                           |
|-------------------|---------------------------|-------------|---------------------------------------|
| `mode`            | `builtin`, `external`     | `builtin`   | Which file picker `:explore` opens    |
| `active_external` | a picker name             | not set     | External picker used in external mode |

## Keybindings

Custom keybindings go in `[keymaps]` with subsections for each context.
Each entry maps a key string to an action name.

```
[keymaps.editor]
"Ctrl+s" = "save"
"Ctrl+z" = "undo"

[keymaps.command_line]
"Enter" = "enter"

[keymaps.debug]
"Esc" = "exit_debug"
```

Key strings use `+` as a separator: `Ctrl+s`, `Shift+Up`, `Alt+Enter`.
`Ctrl+S` and `Ctrl+s` are the same key.

Each `[keymaps.<context>]` table is the **complete** list for that context: if
you write one, keys you leave out are not bound. The controls screen writes
the whole table for you, which is the safest way to change a single key.

Shortcuts of plugin commands live in `[plugins.keybindings]`:

```
[plugins.keybindings]
"menu_plugin.menu" = "Ctrl+P"
```

See `:help controls` for all available action names per context and the rules
for which keys can be bound.

## Validation

The config is validated on load. Invalid values are silently ignored and
fall back to defaults. To check your config for errors and warnings,
use `:config` to see the validation report.

**Note:** Reload your config with `:config reload` after editing the file
by hand. No restart required.
