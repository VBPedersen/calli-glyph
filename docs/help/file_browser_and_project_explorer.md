---
id: file_browser
title: File Browser & Project Explorer
summary: Explore project directories and open files using the built-in browser or configured external tools
tags: file_browser, project_explorer, explore, files, cd, project, yazi, ranger, nnn, tree, picker
---

# File Browser & Project Explorer

`calli-glyph` features a powerful project exploration system that lets you navigate directories, manage project roots, and open files. You can use either the built-in tree-view modal browser or integrate your favorite external terminal file picker (such as `yazi`, `ranger`, or `nnn`).

## Opening and Exploring

| Command / Alias | Description |
|------------------|------------------------------------------------------------------|
| `:explore` | Open the file picker (builtin browser or configured external tool) |
| `:files` | Alias for `:explore` |

## Managing Project Roots (`:cd`)

The project manager automatically detects your project root by walking up directories looking for version control markers (`.git`, `.hg`, `.svn`). You can also explicitly change or reset the project root using the `:cd` command:

| Command | Description |
|-------------------|------------------------------------------------------------------|
| `:cd <path>` | Set the project root explicitly to an absolute or relative path |
| `:project <path>` | Alias for `:cd <path>` |
| `:cd` | Reset the root back to the auto-detected project root (based on the current file or working directory) |

---

## Built-in File Browser Controls

When `config.project.file_picker.mode = "builtin"` (the default), launching `:explore` opens an interactive modal tree view. Directories are lazily expanded as you browse.

### Navigation & Actions

| Key | Action |
|-------------------|------------------------------------------------------------------|
| `↑` / `↓` or `j` / `k` | Move cursor up and down through visible entries |
| `Enter` | Toggle a directory open/closed, or open a selected file into the editor |
| `l` / `→` | Expand a collapsed directory |
| `h` / `←` | Collapse an expanded directory, or jump to its parent row if already collapsed or on a file |
| `r` | Enter the selected directory as the new root (updating `app.project_manager.root` for future sessions) |
| `u` | Go up a step to the parent of the current browser root |
| `g` | Jump to the top of the list |
| `b` | Jump to the bottom of the list |
| `Esc` | Close the file browser modal |

---

## External File Pickers

If you prefer external terminal file managers, you can configure `calli-glyph` to spawn tools like `yazi`, `ranger`, or `nnn` by updating your project configuration file:

```toml
[project.file_picker]
mode = "external"
active_external = "yazi"
```

When an external picker is active, running `:explore` temporarily suspends the editor's TUI, launches your chosen tool interactively in the terminal, and automatically opens any file you select upon exit.