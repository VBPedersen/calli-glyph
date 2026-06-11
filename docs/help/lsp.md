---
id: lsp
title: Language Server Protocol
summary: Real-time diagnostics, completions, and hover docs powered by any LSP server
tags: lsp, language server, diagnostics, errors, warnings, hints, completion, hover, rust-analyzer, pylsp, typescript, tsserver, config, :lsp, lsp_status, enabled, command, args, extensions
---
# Language Server Protocol (LSP)

calli-glyph integrates with any LSP-compatible language server to give you real-time
diagnostics (errors, warnings, hints), code completions, and hover documentation —
all without leaving the terminal.

The LSP client follows the
[Language Server Protocol 3.17 specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/).
It communicates with the server in a background thread so editing is never blocked
while waiting for a response.

## How it Works

When you open a file, calli-glyph checks whether any configured LSP server handles
that file extension. If one is found it is spawned as a subprocess and the editor
sends the following lifecycle events automatically:

- **initialize** — sent on startup with the workspace root
- **textDocument/didOpen** — sent when the file is first opened
- **textDocument/didChange** — sent on every buffer edit
- **textDocument/didSave** — sent when you save the file

The server sends back `textDocument/publishDiagnostics` notifications at its own
pace. calli-glyph polls for these once per tick and updates the gutter markers and
diagnostics panel without any manual intervention.

## Diagnostic Display

Diagnostics are shown in two places:

**Gutter icons** — appear in the line number column next to affected lines:

| Icon | Colour | Meaning          |
|------|--------|------------------|
| `●`  | Red    | Error            |
| `◆`  | Yellow | Warning          |
| `◉`  | Cyan   | Information      |
| `·`  | Gray   | Hint             |

**Diagnostics panel** — toggle with `Ctrl+D`. Shows all diagnostics for the
workspace, grouped by file. The current file is listed first. Each entry shows the
line and column, the message, and the source (e.g. `rustc`, `clippy`). Use `j`/`k`
to scroll the panel and `f` to toggle between *current file only* and *all files*.

The **status bar** always shows a live count of errors and warnings for the
workspace so you can see the health of the project at a glance without opening
the panel.

## Configuration

LSP is configured under `[lsp]` in your `config.toml`. The top-level `enabled`
flag controls whether any server is ever started.

```toml
[lsp]
enabled = true
```

Each language server lives under `[lsp.servers.<name>]`:

```toml
[lsp.servers.rust-analyzer]
command  = "rust-analyzer"
args     = []
extensions = ["rs"]

[lsp.servers.pylsp]
command  = "pylsp"
args     = []
extensions = ["py"]

[lsp.servers.typescript-language-server]
command  = "typescript-language-server"
args     = ["--stdio"]
extensions = ["ts", "tsx", "js", "jsx"]
```

### Server Config Fields

| Field        | Type            | Required | Description                                                  |
|--------------|-----------------|----------|--------------------------------------------------------------|
| `command`    | string          | yes      | Executable to spawn — must be on `PATH` or an absolute path |
| `args`       | list of strings | no       | Command-line arguments passed to the server on startup       |
| `extensions` | list of strings | yes      | File extensions this server handles (without the leading dot)|

When a file is opened, calli-glyph picks the **first** configured server whose
`extensions` list contains the file's extension. If multiple servers claim the
same extension, the one defined earlier in the config wins.

## Built-in Server Defaults

The following servers are recognised out of the box. They still need to be
installed separately on your system — calli-glyph does not bundle them.

| Language             | Recommended server                    | Install                              |
|----------------------|---------------------------------------|--------------------------------------|
| Rust                 | `rust-analyzer`                       | `rustup component add rust-analyzer` |
| Python               | `pylsp`                               | `pip install python-lsp-server`      |
| JavaScript / TypeScript | `typescript-language-server`       | `npm i -g typescript-language-server typescript` |
| Go                   | `gopls`                               | `go install golang.org/x/tools/gopls@latest` |
| Lua                  | `lua-language-server`                 | see [LuaLS releases](https://github.com/LuaLS/lua-language-server/releases) |
| C / C++              | `clangd`                              | ships with LLVM / your package manager |

Any server not listed here can be added manually using the config format above —
as long as it speaks LSP 3.17 over stdio it will work.

## Workspace Root Detection

The workspace root is determined automatically when a file is opened. calli-glyph
walks up from the file's directory looking for:

- `.git`
- `.hg`
- `.svn`

The first directory containing one of these markers is used as the workspace root
and passed to the server during initialization. If none is found, the file's own
directory is used as the fallback.

This is important for servers like `rust-analyzer` that report diagnostics for the
entire workspace (all crates in a Cargo workspace), not just the file you have open.

## Diagnostics Scope

Some servers (notably `rust-analyzer`) publish diagnostics for every file in the
workspace, not only the currently open file. calli-glyph stores all of them and
makes them visible in the diagnostics panel. Use the `f` filter toggle in the panel
to switch between *current file* and *all files* views.

## Server Lifecycle

- The server process is started the first time a matching file is opened.
- It is stopped and a new one is started whenever you open a file of a different
  language (the old server's process is killed cleanly).
- If the server exits unexpectedly a warning is logged and LSP features are
  silently disabled for the session — the editor continues to work normally.
- The server process is always killed when calli-glyph exits, preventing orphaned
  background processes.

## Troubleshooting

**No diagnostics appearing**
- Confirm the server executable is on your `PATH`: run `rust-analyzer --version`
  (or equivalent) in a terminal.
- Check that `[lsp] enabled = true` is set in your config.
- Open the debug log (`:log`) and look for `[LSP]` lines. A `diag uri=…
  current=… match=false` entry means a URI mismatch — this usually means the
  server resolved the path to a different absolute form. File a bug with the two
  URI values shown.

**Server starts but shows "LSP: starting…" indefinitely**
- The server may be waiting for additional initialisation arguments. Some servers
  require `--stdio` in `args` to use standard I/O mode.

**Completions or hover not working**
- These features are requested on demand. Completions fire at the cursor position,
  hover fires on a dedicated keybind (see `:help keybindings`). If the server has
  not finished initialising yet the requests are silently dropped — wait for the
  status bar to show `LSP: ready`.

**High CPU usage**
- Some servers (e.g. `rust-analyzer`) do background indexing when first started.
  This is normal and subsides after the initial workspace load.