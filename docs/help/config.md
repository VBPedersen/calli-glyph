---
id: config
title: Configuration
summary: Customize editor behaviour, appearance, performance, and keybindings
tags: config, configuration, settings, keybindings, theme, :config, reload, tab_width, line_numbers, scrolloff, auto_save
---

# Configuration

calli-glyph is configured via a `config.toml` file stored in your system
config directory (typically `~/.config/calliglyph/config.toml`). Settings
are applied on startup. If no file is found, a default one is created.

## Commands

| Command          | Description                                      |
|------------------|--------------------------------------------------|
| `:config`        | Open the configuration overview                  |
| `:config reload` | Reload config from disk without restarting       |

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
| `theme`           | `"default"`                 | `default` | Editor color theme              |
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
See `:help keybindings` for all available action names per context.

## Validation

The config is validated on load. Invalid values are silently ignored and
fall back to defaults. To check your config for errors and warnings,
use `:config` to see the validation report.

**Note:** Reload your config with `:config reload` after making changes —
no restart required.
