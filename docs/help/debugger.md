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
