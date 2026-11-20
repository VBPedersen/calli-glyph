---
id: themeeditor
title: Theme Editor
summary: Edit every color and layout option of a theme live, with swatches and instant preview
tags: theme editor, edit theme, colors, swatch, hex, syntax colors, layout, border style, revert, reset, new theme, preview
---

# Theme Editor

The theme editor changes the colors and layout options of a theme while you
watch the result. Each change is written to the theme's `.toml` file and
applied immediately.

## Opening the editor

| Trigger                              | Action                                   |
|--------------------------------------|------------------------------------------|
| Theme browser, then `e`              | Edit the selected theme                  |
| Menu: Appearance, Edit current theme | Edit the active theme (key `e`)          |
| Theme browser, then `n`              | Create a theme and edit the new copy     |

See `:help themes` for the browser and for how themes are stored.

## Preview and applying

The theme you edit becomes the visible theme while the editor is open, so you
see every change on the whole interface, including the editor behind it.

This preview is not saved as your active theme:

* Press `a` to keep the edited theme as your active theme
* Close the editor with `Esc` without `a` and the theme you had before comes back

The color changes themselves are always saved in the theme's file.

## Controls

| Key                    | Action                                                         |
|------------------------|----------------------------------------------------------------|
| `Tab` / `Shift+Tab`    | Next / previous group                                          |
| `↑` / `k`, `↓` / `j`   | Move between rows                                              |
| `Enter`                | Edit a color, or change a layout option                        |
| `l` / `→`, `h` / `←`   | Next / previous value for layout options                       |
| `u`                    | Revert the row to what it was when the editor opened           |
| `x`                    | Clear the row from the file (falls back to the default)        |
| `a`                    | Use this theme as the active theme                             |
| `R`                    | Restore a built-in theme to its shipped version (press twice)  |
| `Esc`                  | Close the editor                                               |

## Editing a color

Press `Enter` on a color row. The current value is filled in and a swatch
beside the input shows the color as you type:

| Input              | Result                                        |
|--------------------|-----------------------------------------------|
| `#1e1e1e`          | Stored as `#1e1e1e`                           |
| `#abc` or `abc`    | Expanded and stored as `#aabbcc`              |
| `red`, `lightblue` | Stored as the color name                      |
| anything else      | A red `✗` is shown and nothing is saved       |

`Enter` saves, `Esc` cancels the edit and keeps the old color.

## Groups

| Group    | Contents                                                                |
|----------|-------------------------------------------------------------------------|
| Base     | Background, foreground, borders, selection, cursor                      |
| Bars     | Status bar, command line, list highlight, line numbers, current line    |
| Status   | Error, warning, info, hint, success                                     |
| Accents  | Red, blue, cyan, green, yellow, magenta                                 |
| Logs     | Colors for the debug console log levels                                 |
| Popups   | Popup background, text and border, debug console background             |
| Syntax   | Default text and the colors of keywords, strings, comments and so on    |
| Layout   | Border style, line numbers, status bar position, popup padding, scrollbar, dark hint |

Each row shows its color as a swatch. `(unset)` means the theme does not set it
and it is derived from other colors. This is normal for selection text and the
popup colors.

Syntax color changes also refresh highlighting in the file you have open.

## Built-in themes

`dark`, `light` and `dracula` can be edited like any other theme, the changes
go into their files. Press `R` twice to restore one to the version that ships
with calli-glyph. Themes you created cannot be reset.

## Not covered here

The editor changes colors and layout options. For syntax token modifiers
(`bold`, `italic`, `underline`), background colors on tokens, or tokens that
are not in the list, edit the theme file by hand, see `:help themes`.

Comments and formatting in the theme file are kept when the editor saves.
