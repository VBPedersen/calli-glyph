---
id: debug
title: Developer Console
summary: Real-time internal state inspector and debug overlay
tags: debug, debugger, console, logs, snapshots, developer, :debug
---

# Developer Console

The Developer Console is a read-only overlay providing a real-time snapshot
of the editor's internal state, action history, and system logs.

Primarily intended for development and diagnosing complex editor behaviour
such as cursor positioning, multi-line edits, and history management —
without requiring external print statements or interrupting the editor loop.

The console has **zero performance overhead** when deactivated.

## Commands

| Command              | Description                                              |
|----------------------|----------------------------------------------------------|
| `:debug`             | Toggle the console (defaults to `:debug toggle`)         |
| `:debug enable`      | Activate logging. Does not show the overlay.             |
| `:debug disable`     | Deactivate logging and clear all logs from memory.       |

## Keybindings

These bindings are active while the console overlay is open.

| Key        | Action                          |
|------------|---------------------------------|
| `Tab`      | Cycle to next tab               |
| `Shift+Tab`| Cycle to previous tab           |
| `Ctrl+u`   | Scroll up                       |
| `Ctrl+d`   | Scroll down                     |
| `Ctrl+l`   | Clear logs                      |
| `Ctrl+s`   | Clear snapshots                 |
| `Ctrl+m`   | Take manual snapshot            |
| `Ctrl+c`   | Cycle display mode              |
| `Ctrl+r`   | Reset metrics                   |
| `Esc`      | Close the console               |

**Note:** All keybindings listed are system defaults and can be customized
via your `.config` file.
