---
id: editor
title: Editor
summary: Core text editing, cursor movement, and file operations
tags: editor, cursor, movement, write, save, copy, paste, undo, redo, delete, backspace
---

# Editor

The core editing area of calli-glyph. Opens a file for reading and writing.
Launch with `calli-glyph <filename>` to open a file directly.

## Modes

calli-glyph uses a modal editing model. The active mode determines how
keyboard input is interpreted.

| Mode         | Description                                      |
|--------------|--------------------------------------------------|
| Normal       | Navigation and command input                     |
| Insert       | Direct text input                                |
| Command      | Execute `:commands` via the command line         |
| Visual       | Text selection                                   |

## Keybindings

**Movement**

| Key        | Action                        |
|------------|-------------------------------|
| `h`        | Move cursor left              |
| `j`        | Move cursor down              |
| `k`        | Move cursor up                |
| `l`        | Move cursor right             |
| Arrow keys | Move cursor in any direction  |

**Editing**

| Key        | Action                        |
|------------|-------------------------------|
| `i`        | Enter insert mode             |
| `Esc`      | Return to normal mode         |
| `Backspace`| Delete character before cursor|
| `Delete`   | Delete character after cursor |
| `Ctrl+c`   | Copy selection                |
| `Ctrl+x`   | Cut selection                 |
| `Ctrl+v`   | Paste                         |
| `Ctrl+z`   | Undo                          |
| `Ctrl+r`   | Redo                          |

## Commands

| Command | Description                          |
|---------|--------------------------------------|
| `:w`    | Save file                            |
| `:w!`   | Force save                           |
| `:q`    | Quit without saving                  |
| `:q!`   | Force quit                           |
| `:wq`   | Save and quit                        |

**Note:** All keybindings listed are system defaults and can be customized
via your `.config` file.
