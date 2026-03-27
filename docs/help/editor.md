---
id: editor
title: Editor
summary: Core text editing, selection, clipboard, scrolling, and file operations
tags: editor, cursor, movement, write, save, copy, paste, cut, undo, redo, delete, backspace, selection, scroll, tab, indent
---

# Editor

The main editing area of calli-glyph. Open a file by launching the editor
with a filename: `calli-glyph <filename>`. If the file does not exist it
is created automatically.

## Modes

calli-glyph uses a modal editing model. Press `Esc` to switch between the
editor and the command line.

| Area         | Description                                      |
|--------------|--------------------------------------------------|
| Editor       | Text input and cursor movement                   |
| Command Line | Execute `:commands`                              |

## Movement

| Key        | Action                                               |
|------------|------------------------------------------------------|
| `↑`        | Move cursor up                                       |
| `↓`        | Move cursor down                                     |
| `←`        | Move cursor left (wraps to end of previous line)     |
| `→`        | Move cursor right (wraps to start of next line)      |

## Text Selection

Hold `Shift` while using arrow keys to select text. The selection grows
from where you started moving. Selected text is highlighted and can be
copied, cut, deleted, or replaced by typing.

| Key            | Action                    |
|----------------|---------------------------|
| `Shift+↑`      | Extend selection up        |
| `Shift+↓`      | Extend selection down      |
| `Shift+←`      | Extend selection left      |
| `Shift+→`      | Extend selection right     |

Moving the cursor without `Shift` clears the selection.

## Editing

| Key         | Action                                                   |
|-------------|----------------------------------------------------------|
| `i` / any char | Type to insert at the cursor position                 |
| `Enter`     | Insert a new line. Splits the current line at the cursor |
| `Tab`       | Insert a tab or spaces. At the start of a line, matches the indentation of the line above |
| `Backspace` | Delete the character before the cursor. If text is selected, deletes the selection |
| `Delete`    | Delete the character after the cursor. If text is selected, replaces selection with whitespace |

## Clipboard

Copy, cut, and paste operate on the current text selection. There is no
system clipboard integration — the clipboard is internal to the session.

| Key      | Action                                              |
|----------|-----------------------------------------------------|
| `Ctrl+c` | Copy the selected text                              |
| `Ctrl+x` | Cut the selected text (removes it from the buffer)  |
| `Ctrl+v` | Paste at the cursor position                        |

Multi-line text can be copied and pasted. Pasting inserts text at the
cursor, splitting the current line if necessary.

## Undo & Redo

| Key      | Action                |
|----------|-----------------------|
| `Ctrl+z` | Undo last action      |
| `Ctrl+y` | Redo last undone action |

Every edit is recorded in the undo history. See `:help undo_redo` for
details on the history limit and how bulk actions like replace-all work.

## Scrolling

The editor scrolls automatically to keep the cursor in view. The
`scrolloff` setting keeps a margin of context lines above and below the
cursor as you move. Scroll offset and behaviour can be tuned in your
config — see `:help config`.

## Tabs and Indentation

`Tab` is context-aware. At the start of a line, it copies the indentation
from the line above so new lines stay consistently indented. Elsewhere it
inserts a tab character or a fixed number of spaces depending on your config:

```
[editor]
tab_width = 4
use_spaces = false
```

Set `use_spaces = true` to insert spaces instead of a tab character.

## File Commands

| Command | Description                          |
|---------|--------------------------------------|
| `:w`    | Save file                            |
| `:w!`   | Force save (no confirmation prompt)  |
| `:q`    | Quit without saving                  |
| `:q!`   | Force quit                           |
| `:wq`   | Save and quit                        |

**Note:** All keybindings are defaults and can be remapped in your `.config`
file. See `:help config` and `:help keybindings` for details.
