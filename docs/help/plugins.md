---
id: plugins
title: Plugin System
summary: Extend calli-glyph with optional, configurable plugins
tags: plugins, plugin, enable, disable, list, :plugin, extensions, Ctrl+F, Ctrl+T
---

# Plugin System

Plugins extend calli-glyph with optional functionality that can be toggled
and configured independently of the core editor. The editor runs fine without
any plugins active.

Each plugin can register its own commands, command aliases, and default
keybindings. When a plugin is active it receives keyboard input before the
editor does, and can render its own overlay on top of the editor.

## Commands

| Command                    | Description                               |
|----------------------------|-------------------------------------------|
| `:plugin list`             | List all available plugins and their status |
| `:plugin enable <name>`    | Enable a plugin by name                   |
| `:plugin disable <name>`   | Disable a plugin by name                  |
| `:plugin <name> help`      | Show help for a specific plugin           |

## Built-in Plugins

| Plugin                 | Default Key | Command       | Description                |
|------------------------|-------------|---------------|----------------------------|
| `search_replace_plugin`| `Ctrl+F`    | `:search`     | Find and replace text      |
| `test_plugin`          | `Ctrl+T`    | `:test`       | Developer test plugin      |

## Config

Plugins are enabled in the `[plugins]` section of your `.config` file.
List each plugin you want loaded under `enabled`:

```
[plugins]
enabled = ["search_replace_plugin"]
```

Default keybindings for a plugin can be overridden per command using the
format `plugin_name.command_name`:

```
[plugins]
keybindings = { "search_replace_plugin.search" = "Ctrl+H" }
```

When a keybinding is set in config for a plugin, the plugin's own default
for that command is ignored. Plugins without any config overrides keep all
their built-in defaults.

## Writing Plugins

Plugins implement the `Plugin` trait. A plugin must provide:

- A unique `name` used to identify it across the system
- A `metadata` block declaring its commands and default keybindings
- An `init` handler called when the editor starts
- A `handle_key_event` handler — return `true` to consume the key, `false` to pass it through
- A `render` function for drawing any overlay UI

Refer to `src/plugins/test_plugin.rs` for a minimal working example.

**Note:** Only enable plugins you trust. Plugins have full access to editor state.
