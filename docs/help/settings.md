---
id: settings
title: Settings
summary: Change common options from inside the editor, saved straight to config.toml
tags: settings, :settings, options, preferences, config, tab_width, use_spaces, scrolloff, auto_save, lazy_redraw, restart, reset
---

# Settings

The settings screen lets you change the most common options without editing
`config.toml` by hand. Every change is saved immediately.

## Opening settings

| Trigger          | Action                                      |
|------------------|---------------------------------------------|
| `:settings`      | Open the settings screen                    |
| `s`              | Open it from the welcome screen             |
| Menu             | Settings, then Settings (key `s`)           |

## Navigation

| Key                    | Action                                                    |
|------------------------|-----------------------------------------------------------|
| `Tab` / `Shift+Tab`    | Next / previous category                                  |
| `↑` / `k`, `↓` / `j`   | Move between settings                                     |
| `l` / `→`              | Next value: turn on, increase, or next choice             |
| `h` / `←`              | Previous value: turn off, decrease, or previous choice    |
| `Enter`                | Same as `l`                                               |
| `r`                    | Reset the selected setting to its default                 |
| `o`                    | Open the raw `config.toml` in the editor                  |
| `Esc`                  | Close                                                     |

The current value is shown on the right as `‹ value ›`. The description of the
selected setting is shown below the list, and a message line confirms each save.

## Available settings

| Category    | Setting              | Values                | Applies        |
|-------------|----------------------|-----------------------|----------------|
| Editor      | Tab width            | 1 to 16               | immediately    |
| Editor      | Use spaces           | on / off              | immediately    |
| Editor      | Scrolloff            | 0 to 20               | immediately    |
| Editor      | Scroll lines         | 1 to 20               | immediately    |
| Editor      | Bottom margin        | 0 to 20               | immediately    |
| Editor      | Undo history         | 10 to 10000           | restart        |
| Saving      | Auto-save            | on / off              | immediately    |
| Saving      | Auto-save delay (ms) | 500 to 60000          | immediately    |
| Performance | Lazy redraw          | on / off              | immediately    |
| Performance | Tick rate (ms)       | 10 to 500             | restart        |
| Performance | Cursor blink (ms)    | 100 to 2000           | restart        |
| Project     | File picker          | builtin, external     | immediately    |

Settings marked `↻` in the list are saved at once but only take effect after
you restart calli-glyph. Numbers stop at their minimum and maximum.

Options that are not in this list can still be set in `config.toml`, see
`:help config`.

## How saving works

* Changes are written to `config.toml` and the config is reloaded right away.
* Comments and formatting in your file are kept. Only the changed line is touched.
* Resetting a setting removes its line, so the default applies again.
* If `config.toml` has a syntax error the screen refuses to write and tells you
  to fix the file first, so a hand edit is never overwritten.

## Related

| Topic          | Where                                              |
|----------------|----------------------------------------------------|
| Themes         | `:help themes`                                     |
| Key bindings   | `:help controls`                                   |
| Config file    | `:help config`                                     |
