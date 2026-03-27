---
id: undo_redo
title: Undo & Redo
summary: Step backwards and forwards through your edit history
tags: undo, redo, history, Ctrl+z, Ctrl+y, undo_history_limit
---

# Undo & Redo

Every edit you make in calli-glyph is recorded in a history stack. Undo
steps backwards through that history one action at a time, and Redo steps
forwards again if you have undone something.

## Keybindings

| Key      | Action |
|----------|--------|
| `Ctrl+z` | Undo last action  |
| `Ctrl+y` | Redo last undone action |

## What gets recorded

Every edit is tracked, including single character insertions and deletions,
multi-character selections being deleted or replaced, paste operations,
cut operations, newlines (Enter), and search-and-replace — both single
replacements and replace-all. Replace all is recorded as a single bulk action,
so it takes only one `Ctrl+z` to undo all changes from that operation.

## Dirty State

The editor tracks whether your file has unsaved changes. The undo/redo
history is aware of your last save point — stepping back past it means
you have unsaved changes, and stepping forward to it means the file
matches what is on disk. This is reflected in the status bar.

## History Limit

The history is capped to avoid unlimited memory use. The default limit
is **1000 actions**. When the limit is reached, the oldest entry is dropped
to make room for the new one. The limit can be changed in your config:

```
[editor]
undo_history_limit = 1000
```

Setting it to `0` disables undo entirely. Very high values (above 10,000)
may use significant memory on large or long editing sessions.

## Redo is cleared on new edits

If you undo several steps and then type anything, the redo stack is cleared.
There is no branching history — making a new edit after undoing commits you
to the new path.

**Note:** Undo and redo keybindings can be remapped in your `.config` file
under `[keymaps.editor]`. See `:help config` for details.
