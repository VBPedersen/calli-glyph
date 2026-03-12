---
id: config
title: Configuration
summary: Customize keybindings, editor behaviour, and appearance
tags: config, configuration, settings, keybindings, theme, :config
---

# Configuration

calli-glyph is configured via a `.config` file located in your home directory
or alongside the binary. Settings are applied on startup.

## Commands

| Command        | Description                          |
|----------------|--------------------------------------|
| `:config`      | Open the configuration overview      |
| `:config reload` | Reload config from disk without restarting |

## Config File

The config file uses a simple key-value format. Example:

```
theme = dark
line_numbers = true
tab_width = 4
```

## Options

| Key             | Values              | Default | Description                       |
|-----------------|---------------------|---------|-----------------------------------|
| `theme`         | `dark`, `light`     | `dark`  | Editor color theme                |
| `line_numbers`  | `true`, `false`     | `true`  | Show line numbers in the gutter   |
| `tab_width`     | number              | `4`     | Width of a tab character          |
| `scroll_offset` | number              | `3`     | Lines of context kept above/below cursor |

## Keybindings

Custom keybindings can be defined in the `[keybindings]` section of your
config file. Refer to the **Keybindings** help page for available actions.

**Note:** Invalid config values are silently ignored and fall back to defaults.
