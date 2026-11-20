---
id: welcome
title: Welcome Screen
summary: The start screen shown when calli-glyph opens without a file, or with a folder
tags: welcome, home, start, startup, folder, project, new file, open, quit, logo
---

# Welcome Screen

The welcome screen is shown instead of the editor when calli-glyph starts
without a file to edit. It gives quick access to the most common actions and
goes away as soon as you open or create something.

## When it appears

| Launch                  | What you see                                              |
|-------------------------|-----------------------------------------------------------|
| `calliglyph`            | Welcome screen with the quick actions                     |
| `calliglyph <folder>`   | Welcome screen for that folder: quick actions plus its files |
| `calliglyph <file>`     | The editor directly, no welcome screen                    |

`calliglyph .` opens the welcome screen for the current directory. The folder
also becomes the project root, see `:help project`.

On small terminals the large logo is replaced by a compact one so the list
always fits.

## Quick actions

| Key | Action                                                              |
|-----|---------------------------------------------------------------------|
| `n` | New file: leave the welcome screen with an empty buffer             |
| `o` | Open file. In folder mode this is "Browse project files"            |
| `m` | Open the main menu, see `:help menu`                                |
| `c` | Go to the command line                                              |
| `s` | Open the settings, see `:help settings`                             |
| `h` | Open help                                                           |
| `q` | Quit                                                                |

## Navigation

| Key                 | Action                                                  |
|---------------------|---------------------------------------------------------|
| `↑` / `k`           | Move selection up                                       |
| `↓` / `j`           | Move selection down                                     |
| `Enter`             | Run the selected entry                                  |
| Toggle area key     | Switch to the command line, press again to come back (`Esc` by default) |

## Folder mode

When started with a folder, the screen shows the folder name and path, and a
**Files** section listing the top level of that folder (up to 12 entries).

| Entry              | `Enter` does                                                   |
|--------------------|----------------------------------------------------------------|
| File               | Opens the file                                                 |
| Folder (`▸`)       | Moves into that folder: it becomes the project root and the list updates |

If the folder has more entries, the last line tells you how many more there
are. Press `o` to browse all of them.

## When it goes away

The welcome screen is dismissed when you:

* open a file by any route: the list, the file picker, `:explore`, the menu or a command
* choose **New file**

Opening the menu, settings, help or the command line does **not** dismiss it,
and cancelling the file picker returns you to it.
