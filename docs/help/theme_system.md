---
id: themes
title: Theme System & Theme Picker
summary: Customize syntax highlighting, UI colors, and layout structure with TOML themes and the theme picker plugin
tags: theme, themes, theme picker, ui, syntax, colors, layout, :themepicker, :thp, :theme, Ctrl+T, dracula
---

# Theme System & Theme Picker

The theme system consolidates syntax highlighting, UI component styling, and structural layout options into single `.toml` files. Themes are managed centrally by the editor, ensuring that all UI components and syntax highlightings draw from a unified palette without hardcoded fallbacks.

The editor comes with built-in embedded fallbacks (`dark` and `light`) and automatically writes default theme files to your configuration directory on first launch.

---

## Theme Picker Plugin

The Theme Picker allows you to interactively preview and select loaded themes directly inside the editor.

| Trigger          | Action                                           |
|------------------|--------------------------------------------------|
| `Ctrl+T`         | Open theme picker popup modal                    |
| `:themepicker`   | Open theme picker popup modal                    |
| `:thp`           | Alias for `:themepicker`                         |
| `:theme`         | Alias for `:themepicker`                         |

### Controls inside Theme Picker

* **`Up` / `Down`**: Navigate through available themes.
* **`Enter`**: Apply selected theme and close picker.
* **`Esc`**: Cancel and close picker without changing.

---

## Theme Configuration

The default active theme is configured in your editor's main configuration file under the `[ui]` section:

```toml
[ui]
theme = "dracula"
```

If the configured theme file cannot be found, the editor gracefully falls back to the embedded `"dark"` theme with a log warning.

---

## Theme Anatomy

Each theme file is written in TOML and divided into four functional sections:

### 1. `[meta]`
Contains basic metadata describing the theme name, author, and light/dark variant.

```toml
[meta]
name = "Dracula"
author = "Your Name"
dark = true
```

### 2. `[syntax]`
Defines tree-sitter syntax token styles (`[syntax.tokens]`) and general fallback colors (`[syntax.defaults]`).

Available style properties per token include `fg`, `bg`, `bold`, `italic`, and `underline`.

```toml
[syntax.tokens]
keyword  = { fg = "#ff79c6", bold = true }
string   = { fg = "#f1fa8c" }
comment  = { fg = "#6272a4", italic = true }
number   = { fg = "#bd93f9" }

[syntax.defaults]
fg = "#f8f8f2"
```

### 3. `[ui]`
Controls general user interface colors and per-component overrides.

```toml
[ui.colors]
background      = "#282a36"
foreground      = "#f8f8f2"
border          = "#44475a"
border_focused  = "#bd93f9"
selection_bg    = "#44475a"
cursor          = "#f8f8f2"
status_bar_bg   = "#191a21"
status_bar_fg   = "#f8f8f2"
error           = "#ff5555"

[ui.components]
popup_bg     = "#21222c"
popup_border = "#bd93f9"
```

### 4. `[layout]`
Defines structural formatting preferences bundled alongside the theme.

```toml
[layout]
border_style = "rounded"          # plain | rounded | double | thick
show_line_numbers = true
line_number_style = "absolute"    # absolute | relative | both
status_bar_position = "bottom"    # top | bottom
popup_padding = 1
scrollbar = true
```

---

## Adding Custom Themes

To add a new theme:

1. Create a `.toml` file inside your runtime `themes` directory (e.g., `~/.config/calliglyph/themes/custom.toml`).
2. Define the four sections (`[meta]`, `[syntax]`, `[ui]`, `[layout]`).
3. Open the editor and hit `Ctrl+T` to select your new theme.
