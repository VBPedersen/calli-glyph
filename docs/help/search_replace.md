---
id: search_replace
title: Search & Replace
summary: Find text and replace occurrences across the file
tags: search, replace, find, plugin, search_replace, Ctrl+F, :search, :find, :s
---

# Search & Replace

The Search & Replace plugin lets you find any text in the current file and
optionally replace individual occurrences or all of them at once.

Open the dialog with `Ctrl+F` or the `:search` command. All matches are
highlighted in the editor as you type. The currently selected match is shown
in **yellow**, all others in gray.

## Opening

| Method        | Description                        |
|---------------|------------------------------------|
| `Ctrl+F`      | Open the search and replace dialog |
| `:search`     | Open via command line              |
| `:find`       | Alias for `:search`                |
| `:s`          | Alias for `:search`                |

## Fields

The dialog has two input fields: **Search** and **Replace**.
Use `Tab` to switch between them.

| Field     | Description                           |
|-----------|---------------------------------------|
| `Search`  | Text to search for (live, as you type)|
| `Replace` | Text to substitute in place of match  |

## Keybindings

| Key           | Action                                      |
|---------------|---------------------------------------------|
| `Tab`         | Switch between Search and Replace fields    |
| `↑`           | Jump to previous match                      |
| `↓`           | Jump to next match                          |
| `Enter`       | Replace the currently selected match        |
| `Shift+Enter` | Replace all matches at once                 |
| `Backspace`   | Delete last character in focused field      |
| `Esc`         | Close the dialog                            |

## Behaviour

Matches are found and highlighted live as you type into the Search field.
The editor scrolls automatically to keep the selected match visible.

Replacements are recorded in undo history. After replacing the current match,
the plugin moves to the next one automatically. Replace All processes all
matches in a single undoable action.

**Note:** Search is currently case-sensitive. Multi-line search is not yet
supported.
