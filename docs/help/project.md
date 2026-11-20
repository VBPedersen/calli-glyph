---
id: project
title: Project & File Picker
summary: Project root, the built-in file browser, and using an external picker such as yazi
tags: project, root, :explore, :files, :cd, :project, file picker, file browser, yazi, external, builtin, git, folder
---

# Project & File Picker

calli-glyph keeps track of a **project root**: the folder your work lives in.
The file picker starts there, and you can change the root at any time.

## Commands

| Command             | Description                                                      |
|---------------------|------------------------------------------------------------------|
| `:explore`          | Open the file picker                                             |
| `:files`            | Alias for `:explore`                                             |
| `:cd <path>`        | Set the project root                                             |
| `:cd`               | Reset the root to the automatically detected one                 |
| `:project`          | Alias for `:cd`                                                  |

The file picker is also available from the menu (Project, Browse files, key
`f`) and from the welcome screen (key `o`).

## How the project root is chosen

| Situation                       | Root                                                        |
|---------------------------------|-------------------------------------------------------------|
| Started with a folder           | That folder, exactly as given                               |
| Started with a file             | Found by walking up from the file to the first folder containing `.git`, `.hg` or `.svn` |
| Started with nothing            | None until you set one. The picker starts in the current directory |
| `:cd <path>`                    | That folder. Relative paths are resolved from the current root |
| `:cd` with no path              | Detected again from the open file, or the current directory |

`:cd` only accepts folders. Pointing it at a file or a path that does not
exist shows an error and leaves the root unchanged.

On the welcome screen, entering a folder from the list also sets it as the
project root, see `:help welcome`.

## Built-in browser

The built-in browser is the default. It needs nothing installed and works the
same everywhere. Move with the arrow keys or `j` and `k`, press `Enter` to open
a file or enter a folder, and `Esc` to close. The footer of the browser lists
all its keys.

Opening a file leaves the welcome screen if it was showing.

## External picker

You can use a file manager you already know, such as `yazi`, instead of the
built-in browser. calli-glyph hands the whole terminal to the tool and waits.
When the tool exits with a file selected, that file opens in the editor. If it
exits without a selection, nothing happens and you are back where you were.

Configure it in `config.toml`:

```toml
[project.file_picker]
mode = "external"
active_external = "yazi"
```

| Key                | Values                | Description                                    |
|--------------------|-----------------------|------------------------------------------------|
| `mode`             | `"builtin"`, `"external"` | Which picker `:explore` opens              |
| `active_external`  | a name                | Which external picker to use in external mode  |

Each external picker is defined in its own table named after it,
`[project.file_picker.external.<name>]`, for example `yazi`. The name there
must match `active_external`.

You can switch `mode` from the settings screen (Project, File picker), see
`:help settings`.

### If the external picker does not open

Problems are written to the log instead of shown as popups:

* `mode` is `"external"` but `active_external` is not set
* `active_external` names a picker that has no matching table
* The tool failed to start

Open the debug console and check the logs, see `:help debug`.
