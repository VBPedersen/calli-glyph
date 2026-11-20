---
id: debug
title: Developer Console
summary: Real-time internal state inspector, logger, and performance monitor
tags: debug, debugger, console, logs, snapshots, metrics, developer, :debug, :dbg
---

# Developer Console

The Developer Console is a read-only overlay that exposes the editor's
internal state in real time. It is intended for diagnosing complex behaviour
such as cursor positioning, selection state, undo history depth, clipboard
contents, and rendering performance.

The console has **zero overhead** when deactivated — logging and metrics
collection are fully disabled until you turn it on.

## Commands

| Command          | Description                                     |
|------------------|-------------------------------------------------|
| `:debug`         | Toggle the console (enable + show overlay)      |
| `:debug enable`  | Start logging without opening the overlay       |
| `:debug disable` | Stop logging and clear all data from memory     |
| `:dbg`           | Alias for `:debug`                              |

## Tabs

The console has several tabs you can cycle through with `Tab` and `Shift+BackTab`.

| Tab          | Contents                                                  |
|--------------|-----------------------------------------------------------|
| `Overview`   | Cursor position, scroll, active area, file path           |
| `Editor`     | Buffer line count, selection start/end, clipboard size    |
| `History`    | Undo stack depth, redo stack depth, last actions          |
| `Logs`       | Timestamped log entries (Info, Warn, Error, Debug, Trace) |
| `Snapshots`  | Saved point-in-time captures of the full app state        |
| `Metrics`    | Frame time, CPU usage, memory usage, render count         |

## Keybindings

| Key              | Action                          |
|------------------|---------------------------------|
| `Tab`            | Next tab                        |
| `Shift+BackTab`  | Previous tab                    |
| `h`              | Previous tab                    |
| `l`              | Next tab                        |
| `Up` / `k`       | Scroll up                       |
| `Down` / `j`     | Scroll down                     |
| `s`              | Take a manual snapshot          |
| `c`              | Clear logs                      |
| `C`              | Clear snapshots                 |
| `r`              | Reset performance metrics       |
| `m`              | Cycle snapshot capture mode     |
| `Enter`          | Open selected log / snapshot    |
| `Esc` / `q`      | Close the console               |

## Snapshots

Snapshots are point-in-time captures of the full application state including
cursor position, buffer content, scroll offset, clipboard, undo/redo stacks,
active area, file path, and current frame time. Up to **50 snapshots** are
kept in memory at once — older ones are dropped automatically.

### Capture Modes

Cycle through modes with `m`:

| Mode          | When a snapshot is taken                          |
|---------------|---------------------------------------------------|
| `OnEvent`     | Automatically on commands, key presses, and errors (default) |
| `Manual`      | Only when you press `s`                           |
| `EveryFrame`  | Every render frame (high volume — use sparingly)  |
| `None`        | Never                                             |

## Logs

The log viewer shows entries from the global logger. Each entry has a
timestamp, level, and message. Levels in ascending severity: `Trace`,
`Debug`, `Info`, `Warn`, `Error`. The log buffer holds up to **1000 entries**
before the oldest are dropped.

## Performance Metrics

The Metrics tab shows live stats refreshed approximately every 500ms:

| Metric         | Description                                     |
|----------------|-------------------------------------------------|
| Frame time     | Average time per render over the last 120 frames |
| Min / Max      | Fastest and slowest frame in the recent window  |
| CPU usage      | Normalized across all cores                     |
| Memory         | Resident memory usage in MB                     |
| Render count   | Total frames rendered this session              |
| Event count    | Total input events processed this session       |

Press `r` to reset the frame time window and counters.

**Note:** All keybindings listed are defaults and can be remapped via your
`.config` file. See `:help config` for details.

## State Tab

The **State** tab is the second tab of the debug console, right after Overview.
It shows what the app is doing right now, in sections:

| Section            | Shows                                                                  |
|--------------------|------------------------------------------------------------------------|
| Focus              | Active area, whether the welcome screen is up, open popup, stack of open screens, active plugin |
| Pending states     | The queue of steps waiting on the user, in order, with `▶` on the next one |
| Buffer             | File, modified flags, line count, content version, cursor, selection, auto-save |
| Project & language | Project root, language id, document URI, whether the language server is running |
| Plugins            | Loaded plugins, number of commands, number of menu entries             |

### Pending states

Some actions need several confirmations in a row, for example "save, then open
another file" or "save, then quit". The steps are queued and resolved one at a
time as you answer each prompt. The State tab lists the queue, for example:

    ▶ 1. Save /home/me/project/main.rs
      2. Open /home/me/project/lib.rs at line 12

If the queue has entries but no popup or screen is waiting on you, a warning is
shown: the queue is stale, which usually means a step was dropped. Each step
being queued and resolved is also written to the **Logs** tab with a `[Pending]`
prefix.

Use `Tab` / `Shift+Tab` to switch tabs and the scroll keys to move through a
long State page, same as the other tabs.
