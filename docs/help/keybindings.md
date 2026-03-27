---
id: keybindings
title: Keybindings
summary: All default keybindings and how to customize them
tags: keybindings, keymaps, keys, shortcuts, bindings, config, remap, editor, command_line, debug
---

# Keybindings

calli-glyph uses context-sensitive keybindings. The active context determines
which bindings are in effect. There are three contexts: **Editor**,
**Command Line**, and **Debug Console**.

All bindings listed here are defaults and can be remapped in your
`.config` file. See `:help config` for the keymap format.

## Editor Keybindings

Active while editing text in the main editor area.

**Movement**

| Key          | Action                  |
|--------------|-------------------------|
| `Up`         | Move cursor up          |
| `Down`       | Move cursor down        |
| `Left`       | Move cursor left        |
| `Right`      | Move cursor right       |

**Selection**

| Key            | Action                  |
|----------------|-------------------------|
| `Shift+Up`     | Extend selection up     |
| `Shift+Down`   | Extend selection down   |
| `Shift+Left`   | Extend selection left   |
| `Shift+Right`  | Extend selection right  |

**Editing**

| Key        | Action                        |
|------------|-------------------------------|
| `Ctrl+s`   | Save file                     |
| `Ctrl+z`   | Undo                          |
| `Ctrl+y`   | Redo                          |
| `Ctrl+c`   | Copy selection                |
| `Ctrl+v`   | Paste                         |
| `Ctrl+x`   | Cut selection                 |
| `Backspace`| Delete character before cursor|
| `Delete`   | Delete character after cursor |
| `Enter`    | Insert newline                |
| `Tab`      | Insert tab or spaces          |

**Other**

| Key    | Action                                  |
|--------|-----------------------------------------|
| `Esc`  | Toggle between editor and command line  |

## Command Line Keybindings

Active while typing in the command line (`:` mode).

| Key          | Action                     |
|--------------|----------------------------|
| `Enter`      | Execute command            |
| `Esc`        | Return to editor           |
| `Backspace`  | Delete character           |
| `Delete`     | Delete character forward   |
| `Left`       | Move cursor left           |
| `Right`      | Move cursor right          |

## Debug Console Keybindings

Active while the debug console overlay is open. See `:help debug` for
full console documentation.

| Key              | Action                    |
|------------------|---------------------------|
| `Esc` / `q`      | Close the console         |
| `Tab`            | Next tab                  |
| `Shift+BackTab`  | Previous tab              |
| `Up` / `k`       | Scroll up                 |
| `Down` / `j`     | Scroll down               |
| `h`              | Previous tab              |
| `l`              | Next tab                  |
| `s`              | Take manual snapshot      |
| `r`              | Reset metrics             |
| `c`              | Clear logs                |
| `C`              | Clear snapshots           |
| `m`              | Cycle display mode        |
| `Enter`          | Interact with selection   |

## Customizing Keybindings

Add a `[keymaps]` section to your config file with the context as a subsection.
Map key strings to action names:

```
[keymaps.editor]
"Ctrl+s" = "save"
"Ctrl+z" = "undo"
"Ctrl+y" = "redo"
```

Key format: use `+` to combine modifiers with a key — `Ctrl+s`, `Shift+Up`,
`Alt+Enter`. Modifiers supported: `Ctrl`, `Alt`, `Shift`.

## Available Editor Actions

`save`, `copy`, `paste`, `cut`, `undo`, `redo`, `backspace`, `delete`,
`enter`, `tab`, `toggle_area`, `move_up`, `move_down`, `move_left`,
`move_right`, `select_up`, `select_down`, `select_left`, `select_right`

## Available Command Line Actions

`enter`, `tab`, `toggle_area`, `backspace`, `delete`,
`move_up`, `move_down`, `move_left`, `move_right`

## Available Debug Actions

`exit_debug`, `next_tab`, `prev_tab`, `scroll_up`, `scroll_down`,
`clear_logs`, `clear_snapshots`, `manual_snapshot`, `cycle_mode`,
`reset_metrics`, `debug_interact`, `enter`, `tab`, `toggle_area`
