---
id: command_line
title: Command Line
summary: Execute editor commands using the : prompt
tags: command, command_line, :w, :q, :wq, :help, :debug, :config, :plugin, save, quit, commands
---

# Command Line

The command line is how you execute named commands in calli-glyph. It mirrors
the style of modal editors like Vim — press `Esc` from the editor to open
it, type a command starting with `:`, and press `Enter` to run it.

## Opening and Closing

| Key    | Action                                 |
|--------|----------------------------------------|
| `Esc`  | Toggle between editor and command line |
| `Enter`| Execute the current command            |

## Built-in Commands

**File**

| Command  | Aliases              | Description                      |
|----------|----------------------|----------------------------------|
| `:w`     | `:write`, `:save`    | Save the current file            |
| `:w!`    | `:write!`, `:save!`  | Force save (overwrite without prompt) |
| `:q`     | `:quit`              | Quit without saving              |
| `:q!`    | `:quit!`             | Force quit, no confirmation      |
| `:wq`    | `:writequit`         | Save and quit                    |

**Editor**

| Command           | Aliases   | Description                          |
|-------------------|-----------|--------------------------------------|
| `:help`           | `:h`      | Open the help popup                  |
| `:help <topic>`   | `:h <topic>` | Jump to a specific help topic     |
| `:debug`          | `:dbg`    | Toggle the developer console         |
| `:config`         | `:cfg`    | Open the config overview             |
| `:config reload`  |           | Reload config from disk              |

**Plugins**

| Command                    | Description                               |
|----------------------------|-------------------------------------------|
| `:plugin list`             | List all plugins and their status         |
| `:plugin enable <n>`    | Enable a plugin by name                   |
| `:plugin disable <n>`   | Disable a plugin by name                  |
| `:plugin <n> help`      | Show help for a specific plugin           |
| `:search`                  | Open the search and replace dialog        |
| `:find`, `:s`              | Aliases for `:search`                     |

## Flags

Some commands accept flags to modify their behaviour:

| Flag        | Description                              |
|-------------|------------------------------------------|
| `--force`   | Same as `!` suffix — skip confirmations |
| `--dry-run` | Preview the action without applying it   |
| `--backup`  | Create a backup before the operation     |

## Typing in the Command Line

| Key         | Action                              |
|-------------|-------------------------------------|
| `Left`      | Move cursor left                    |
| `Right`     | Move cursor right                   |
| `Backspace` | Delete character before cursor      |
| `Delete`    | Delete character after cursor       |

Commands must begin with `:` — text without a leading colon is not
recognized as a command. Arguments are separated by spaces, for example
`:help debug` or `:plugin enable search_replace_plugin`.

Unknown commands are passed to the plugin system first before failing,
so plugin commands like `:search` work the same way as built-in ones.
