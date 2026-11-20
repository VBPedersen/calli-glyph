---
id: controls
title: Controls
summary: Rebind keyboard shortcuts for the editor, command line, debug console and plugins
tags: controls, :controls, :keys, keybindings, keymaps, shortcuts, rebind, keys, hotkeys, bindings, plugins
---

# Controls

The controls screen lets you change keyboard shortcuts without editing
`config.toml`. Changes are checked, saved and applied immediately.

## Opening controls

| Trigger                      | Action                              |
|------------------------------|-------------------------------------|
| `:controls`, `:keys`         | Open the controls screen            |
| Menu: Settings, Controls     | Open it from the menu (key `c`)     |

## Tabs

| Tab          | What you can rebind                                              |
|--------------|------------------------------------------------------------------|
| Editor       | Editing, movement, selection and file actions                    |
| Command line | Keys while typing a command                                      |
| Debug        | Keys inside the debug console                                    |
| Plugins      | Shortcuts that run plugin commands                               |

Each row shows the action and the keys bound to it. `—` means unbound. An
action can have several keys.

## Navigation

| Key                    | Action                                                     |
|------------------------|------------------------------------------------------------|
| `Tab` / `Shift+Tab`    | Next / previous tab                                        |
| `↑` / `k`, `↓` / `j`   | Move between actions                                       |
| `Enter`                | Rebind: press the new key combination. It replaces all keys of the action |
| `a`                    | Add: press a combination to add another key to the action  |
| `t`                    | Type a key by name, for example `Esc` or `Ctrl+Shift+k`    |
| `x`                    | Unbind the action                                          |
| `u`                    | Reset the action to its default keys                       |
| `R`                    | Reset the whole tab to defaults (press twice)              |
| `Esc`                  | Close, or cancel a rebind in progress                      |

While waiting for a key, `Esc` cancels. To bind `Esc` itself, use `t` and type
`Esc`.

## Rules

Not every key can work everywhere. The controls screen refuses the ones that
cannot, and explains why.

| Rule                                                                           |
|--------------------------------------------------------------------------------|
| Editor: character keys need `Ctrl` (without `Alt`), otherwise they type the character |
| Command line: character keys need `Ctrl` or `Alt`, otherwise they type the character |
| Debug: any key works                                                           |
| Plugins: character keys need `Ctrl` or `Alt`                                   |
| Keys like arrows, `F1` to `F12`, `Home`, `End` work with any modifiers         |

### Keys terminals cannot tell apart

Most terminals send the same signal for these pairs, so the first key of each
pair is rejected:

| Rejected     | Terminal sends |
|--------------|----------------|
| `Ctrl+M`, `Ctrl+J` | `Enter`  |
| `Ctrl+I`     | `Tab`          |
| `Ctrl+H`     | `Backspace`    |
| `Ctrl+[`     | `Esc`          |

### Conflicts

If the key is already bound to another action in the same tab, you are asked:
`Enter` replaces it (the other action loses that key), `Esc` cancels.

Plugin shortcuts are checked before everything else. A key used by a plugin
cannot be bound in the other tabs, and a key used by an editor action cannot
be given to a plugin, because one of the two would never fire.

### Essential actions

These cannot be unbound, so you cannot lock yourself out. Rebind them instead:

| Tab          | Action                      |
|--------------|-----------------------------|
| Editor       | Switch to command line      |
| Command line | Back to editor              |
| Debug        | Close debug console         |

## Plugins tab

Each row is a plugin command, shown as `plugin › command`, with the shortcut
currently in use.

* A command has one shortcut: `Enter` or `t` replaces it, and `a` is not available on this tab
* `u` returns a command to the plugin's default shortcut
* Plugin shortcuts cannot be removed, only changed or reset

**Note:** once you set a shortcut for any command of a plugin, that plugin's
other default shortcuts stop applying until you set them too.

## Where changes are saved

Keymap changes rewrite the `[keymaps.<tab>]` table in `config.toml`
(`editor`, `command_line` or `debug`). The rest of your file, including its
comments, is not touched. Plugin shortcuts go in `[plugins.keybindings]` as
`"plugin.command" = "Ctrl+P"`.

Before saving, the complete keymap is checked. If it could not be loaded,
nothing is written.

## Key names

Keys are written with `+` between the modifiers and the key.

| Part            | Names                                                              |
|-----------------|--------------------------------------------------------------------|
| Modifiers       | `Ctrl`, `Alt`, `Shift`                                             |
| Named keys      | `Enter`, `Tab`, `BackTab`, `Backspace`, `Delete`, `Esc`, `Up`, `Down`, `Left`, `Right`, `PageUp`, `PageDown`, `Home`, `End`, `Space`, `Plus` |
| Function keys   | `F1` to `F12`                                                      |
| Characters      | Any single ASCII character, for example `s` or `/`                 |

`Ctrl+S` and `Ctrl+s` are the same key. A bare uppercase letter such as `C`
means `Shift+c`.

## Action names

For writing `config.toml` by hand.

**Editor:** `save`, `copy`, `cut`, `paste`, `undo`, `redo`, `backspace`,
`delete`, `enter`, `tab`, `toggle_area`, `move_up`, `move_down`, `move_left`,
`move_right`, `select_up`, `select_down`, `select_left`, `select_right`

**Command line:** `enter`, `toggle_area`, `backspace`, `delete`, `tab`,
`move_up`, `move_down`, `move_left`, `move_right`

**Debug:** `exit_debug`, `next_tab`, `prev_tab`, `scroll_up`, `scroll_down`,
`manual_snapshot`, `clear_logs`, `clear_snapshots`, `reset_metrics`,
`cycle_mode`, `debug_interact`, `enter`, `tab`, `toggle_area`
