---
id: themes
title: Theme System & Theme Picker
summary: Customize syntax highlighting, UI colors, and layout with TOML themes, the theme browser, and the theme editor
tags: theme, themes, theme picker, theme browser, ui, syntax, colors, layout, :themepicker, :thp, :theme, Ctrl+T, dark, light, dracula, new theme, preview
---

# Theme System & Theme Picker

The theme system consolidates syntax highlighting, UI component styling, and
structural layout options into single `.toml` files. Themes are managed
centrally by the editor, so every UI component and syntax highlight draws from
one palette with no hardcoded colors.

Three themes are built in: `dark`, `light` and `dracula`. Their files are
written to your themes directory on first launch so you can edit or copy them.
If you delete one of them, it is written back from the built-in copy.

---

## Theme Browser

The theme browser lets you preview and apply themes from inside the editor.

| Trigger          | Action                                           |
|------------------|--------------------------------------------------|
| `Ctrl+T`         | Open the theme browser                           |
| `:themepicker`   | Open the theme browser                           |
| `:thp`, `:theme` | Aliases for `:themepicker`                       |
| Menu             | Appearance, then Themes (key `t`)                |

### Controls inside the browser

| Key              | Action                                                        |
|------------------|---------------------------------------------------------------|
| `↑` / `↓`        | Move through themes. The editor previews each one live       |
| `Enter`          | Apply the selected theme and save it as your theme           |
| `e`              | Edit the selected theme, see `:help themeeditor`             |
| `n`              | Create a new theme as a copy of the selected one             |
| `Esc`            | Cancel and go back to the theme you started with             |

A `●` marks the theme that was active when you opened the browser. Built-in
themes are tagged, and the theme's display name is shown next to its file name.

### Creating a theme

Press `n`, type a name and press `Enter`. The selected theme is copied to a new
file in your themes directory and opened in the theme editor.

* The file name is made from the name: `My Theme!` becomes `my-theme.toml`
* The name inside the file (`[meta] name`) keeps what you typed
* A theme with the same file name is never overwritten

---

## Theme Configuration

The active theme is stored in your main configuration file under `[ui]`.
Applying a theme in the browser saves this for you.

```toml
[ui]
theme = "dracula"
```

If the configured theme cannot be found, the editor falls back to the built-in
`"dark"` theme and writes a warning to the log.

---

## Theme Anatomy

Each theme file is TOML with four sections. Every section is optional: anything
missing falls back to a default.

### 1. `[meta]`
Basic information about the theme.

```toml
[meta]
name = "Dracula"
author = "Your Name"
dark = true
```

`dark` is a hint for whether the theme has a dark background.

### 2. `[syntax]`
Tree-sitter token styles (`[syntax.tokens]`) and a fallback (`[syntax.defaults]`).
Each style can set `fg`, `bg`, `bold`, `italic` and `underline`.

```toml
[syntax.tokens]
keyword  = { fg = "#ff79c6", bold = true }
string   = { fg = "#f1fa8c" }
comment  = { fg = "#6272a4", italic = true }
number   = { fg = "#bd93f9" }

[syntax.defaults]
fg = "#f8f8f2"
```

Token names used by the built-in themes: `keyword`, `string`, `comment`,
`number`, `type`, `function`, `boolean`, `variable`, `property`, `parameter`,
`constant`, `operator`.

### 3. `[ui]`
General interface colors in `[ui.colors]` and per-component overrides in
`[ui.components]`.

| Group      | Keys                                                                      |
|------------|---------------------------------------------------------------------------|
| Base       | `background`, `foreground`, `border`, `border_focused`, `selection_bg`, `selection_fg`, `cursor` |
| Bars       | `status_bar_bg`, `status_bar_fg`, `command_line_bg`, `command_line_fg`, `list_highlight_bg`, `list_highlight_fg`, `line_number`, `line_number_current`, `current_line_bg` |
| Status     | `error`, `warning`, `info`, `hint`, `success`                             |
| Accents    | `accent_red`, `accent_blue`, `accent_cyan`, `accent_green`, `accent_yellow`, `accent_magenta` |
| Logs       | `log_error`, `log_warning`, `log_info`, `log_debug`, `log_trace`          |
| Components | `popup_bg`, `popup_fg`, `popup_border`, `debug_console_bg`                |

`selection_fg` and the component colors are optional. When they are not set,
the selection keeps the syntax color of the text and components follow the
general colors (for example popups use `background`).

```toml
[ui.colors]
background      = "#282a36"
foreground      = "#f8f8f2"
selection_bg    = "#44475a"

[ui.components]
popup_bg     = "#21222c"
popup_border = "#bd93f9"
```

### 4. `[layout]`
Structural preferences bundled with the theme.

| Key                   | Values                              |
|-----------------------|-------------------------------------|
| `border_style`        | `plain`, `rounded`, `double`, `thick` |
| `line_number_style`   | `absolute`, `relative`, `both`      |
| `status_bar_position` | `top`, `bottom`                     |
| `popup_padding`       | a number                            |
| `scrollbar`           | `true`, `false`                     |

```toml
[layout]
border_style = "rounded"
line_number_style = "absolute"
status_bar_position = "bottom"
popup_padding = 1
scrollbar = true
```

Whether line numbers are shown at all is an editor option, not a theme option:
see `line_numbers` in `:help config`.

---

## Color Formats

| Format          | Example                    |
|-----------------|----------------------------|
| Hex             | `"#ff8800"`                |
| Named color     | `"red"`, `"darkgray"`, `"lightblue"` |

Named colors: `red`, `green`, `blue`, `yellow`, `cyan`, `magenta`, `black`,
`white`, `gray`, `darkgray`, and the `light` variants of the colors (for example
`lightred`).

Only 6-digit hex is read from theme files. A value that cannot be understood is
shown in the terminal's default color instead of breaking the theme. The theme
editor accepts 3-digit hex as well and stores it as 6 digits.

---

## Adding Themes by Hand

1. Create a `.toml` file in your themes directory, for example
   `~/.config/calliglyph/themes/custom.toml`.
2. Fill in the sections you want to change. Missing ones use defaults.
3. Restart the editor, then pick the theme with `Ctrl+T`.

The easier route is to press `n` in the theme browser, which starts you from a
complete copy of an existing theme.
