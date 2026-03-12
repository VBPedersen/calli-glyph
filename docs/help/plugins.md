---
id: plugins
title: Plugin System
summary: Extend calli-glyph with optional, configurable plugins
tags: plugins, extensions, plugin, enable, disable, :plugin
---

# Plugin System

Plugins extend calli-glyph with optional functionality that can be enabled,
disabled, and configured independently of the core editor. Unlike built-in
commands, plugins are non-core — the editor runs without them.

## Commands

| Command                  | Description                              |
|--------------------------|------------------------------------------|
| `:plugin list`           | List all available plugins and their status |
| `:plugin enable <name>`  | Enable a plugin                          |
| `:plugin disable <name>` | Disable a plugin                         |
| `:plugin <name> help`    | Show help for a specific plugin          |

## Config

Plugins are configured in the `[plugins]` section of your `.config` file:

```
[plugins]
my_plugin = true
other_plugin = false
```

Individual plugin settings live under their own section:

```
[my_plugin]
some_option = value
```

## Writing Plugins

Plugins implement the `Plugin` trait. At minimum a plugin provides:

- A unique `name`
- An `on_enable` and `on_disable` handler
- Optional command handlers and keybindings

Refer to the source in `src/plugins/` for examples.

**Note:** Only enable plugins you trust. Plugins have access to editor state.
