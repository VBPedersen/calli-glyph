---
id: lang_install
title: Grammar & LSP Server Installer
summary: Install tree-sitter grammars and LSP servers from inside the editor — no manual compiling or hunting for packages
tags: install, installer, tree-sitter, grammar, lsp, lsp server, download, compile, build, github, git, token, githubtoken, ghtoken, npm, pip, cargo, go, PATH, :lang, node-types.json
---
# Grammar & LSP Server Installer

Setting up syntax highlighting and language servers used to mean manually cloning
grammar repos, compiling them yourself, writing a `.toml` node-kind config by
hand, and separately tracking down and installing the right LSP server package.
The **Install** tab in the Language Panel (`:lang`) does all of that for you.

## How it Works

The installer draws from two static catalogs built into calli-glyph:

- **Tree-sitter grammars** — cloned from their upstream GitHub repo and compiled
  with your system's C/C++ compiler into a `.so`/`.dylib`/`.dll`, exactly as
  described in the manual process in [Syntax Highlighting](syntax.md).
- **LSP servers** — installed via whichever package manager is appropriate
  (`npm`, `pip`, `cargo`, or `go`) and, where there's no reliable one-liner
  install (notably `rust-analyzer`), shown as manual instructions instead.

Both kinds of install run in the background on a separate thread, the same way
the LSP client itself runs in the background — the editor is never blocked while
a grammar compiles or a package manager runs. Progress is polled once per tick
and streamed into the panel's log, and on success the result is written straight
into `config.toml` and saved, so it's usable immediately with no restart.

## Using the Install Tab

Open `:lang` and switch to the **Install** tab (`4`). The tab shows two columns:

```
[1] Diagnostics  |  [2] LSP  |  [3] Syntax  |  [4] Install

▸ Tree-sitter grammars              LSP servers
  rust        ✓ installed     rs      rust         ✓ configured  rust-analyzer
▶ python      · not installed py      python       · not installed pyright
  typescript  · not installed ts      typescript   · not installed typescript-lang...
```

| Key       | Action                                          |
|-----------|--------------------------------------------------|
| `g` / `l` | Focus the grammars list / the LSP servers list    |
| `↑` / `↓` | Move the selection within the focused list        |
| `Enter`   | Install the selected grammar or server            |
| `Esc`     | Close the panel                                   |

Each row shows a live status:

| Status            | Meaning                                                |
|-------------------|---------------------------------------------------------|
| `· not installed` | Not yet installed or configured                        |
| `… installing`    | A background job is currently running for this item     |
| `✓ installed`     | The grammar `.so` exists, or the server is configured    |
| `✗ failed`        | The last attempt failed — see the log panel for details |

The log panel at the bottom shows recent output for whichever row is currently
selected, including the full error message on failure.

## Grammar Installs

When you install a grammar, calli-glyph:

1. Clones the grammar's repo (shallow, `--depth 1`) into a temp directory.
2. Compiles `src/parser.c` (and `src/scanner.c`/`scanner.cc` if the grammar has
   one) into the platform's shared library format, and copies it into
   `grammar_dir`.
3. Generates a starter `<grammar>.toml` lang config by reading the grammar's own
   `src/node-types.json` (see *Auto-Generated Lang Configs* below) — but only if
   one doesn't already exist, so a hand-tuned config is never overwritten.
4. Registers the language under `[syntax.languages.<name>]` in `config.toml` and
   saves it.

If a grammar's compiled `.so` already exists at that path, installing it again
recompiles and replaces it — useful for picking up upstream fixes.

**Requirements:** `git` and a C compiler (`cc`, `gcc`, or `clang`; a C++ compiler
too for grammars with a `scanner.cc`) must be on `PATH`. If either is missing,
the install fails with a clear message telling you which one.

### Auto-Generated Lang Configs

Every official tree-sitter grammar ships a `src/node-types.json` listing every
node kind and whether it's a named node or an anonymous token. The installer
uses this to generate a best-effort starter config automatically:

- Anonymous word-like tokens (`if`, `class`, `return`) become `keyword` — this
  is how tree-sitter represents keywords internally, so the heuristic is
  reliable.
- Named nodes are pattern-matched by name: anything containing `comment` →
  `comment`, `string`/`regex` → `string`, `number`/`integer`/`float` → `number`,
  `true`/`false`/`boolean` → `boolean`, `type_identifier`/`predefined_type` →
  `type`.
- Matched `string`/`comment` nodes are added to `stop_at`, except types
  containing `template` (so `${...}` interpolation inside template strings is
  still walked and highlighted).

This can't produce `[[parent_rules]]` — telling a function's name identifier
apart from a variable's requires knowing the grammar's specific node structure,
which isn't in `node-types.json`. A generated config gets you comments, strings,
numbers, booleans, keywords, and basic types immediately; refine it by hand from
there exactly as described in [Syntax Highlighting](syntax.md#lang-config-files).

If a grammar doesn't ship `src/node-types.json` (rare), no config is generated
and you'll need to write one by hand, same as before the installer existed.

### Authenticating Grammar Clones (`:githubtoken`)

Anonymous `git clone` over HTTPS against GitHub is subject to secondary rate
limiting, especially if you install several grammars back-to-back — you may
occasionally see a clone fail and automatically retry. Authenticating raises
that limit substantially and avoids the retry entirely.

Set a GitHub [Personal Access Token](https://github.com/settings/tokens) with
the command line:

```
:githubtoken ghp_your_token_here
```

`:ghtoken` works as a shorthand alias. To go back to anonymous cloning:

```
:githubtoken clear
```

The token is stored in its own file under your config directory (not
`config.toml`), with owner-only permissions on Unix. It's used purely to
authenticate `git clone` — no scopes are required beyond read access to public
repositories. A fine-grained, short-lived token is recommended over a classic
PAT with broad scopes.

Failed clones retry automatically up to 3 times with a short backoff before
giving up, since rate-limit failures are usually transient.

## LSP Server Installs

When you install a server, calli-glyph:

1. Checks whether the server's command is already resolvable — if so, nothing
   is installed, it's just registered in config as-is.
2. Otherwise, runs the appropriate package manager (`npm install -g`,
   `pip install --user`, `cargo install`, or `go install ...@latest`) for that
   server's package.
3. Locates the installed binary's **absolute path** — not just the bare
   command name. Package managers often install to locations that aren't on
   this process's `PATH` (npm's global prefix, `~/.local/bin` for pip installs,
   `$GOPATH/bin` for Go) even though they will be on `PATH` in a fresh shell, so
   the installer asks each package manager directly where it puts things.
4. Writes that resolved path into `[lsp.servers.<name>]` in `config.toml`.

If a server has no reliable one-command install (`rust-analyzer` is the
canonical example — it needs `rustup component add` or a manually downloaded
release binary), the panel shows the manual instructions instead of attempting
one.

**Requirements:** the relevant package manager (`npm`, `pip3`/`python`, `cargo`,
or `go`) must be on `PATH`.

## Troubleshooting

**Grammar clone fails with an auth/username error**
- This means git couldn't complete an anonymous clone (usually rate limiting)
  and the fallback credential prompt was disabled rather than left to hang your
  terminal. It retries automatically; if it still fails after 3 attempts, set a
  GitHub token with `:githubtoken` (see above) and try again.

**LSP install says "installed" but nothing changes**
- Check the log panel for the resolved path — if it found the binary somewhere
  unexpected, `[lsp.servers.<name>].command` in `config.toml` will show exactly
  where. If the log says the binary couldn't be located at all, the install may
  have placed it somewhere non-standard; use the manual instructions shown in
  the error instead.

**No C compiler found**
- Install `gcc`/`clang` via your system package manager (Linux/macOS), or a
  MinGW-based toolchain (Windows). MSVC's `cl.exe` isn't currently supported by
  the automatic compile step — compile manually per
  [Syntax Highlighting](syntax.md#compiling-a-grammar) instead.

**Grammar has no `src/parser.c`**
- Some grammar repos don't commit the generated parser and expect you to run
  `tree-sitter generate` first. The installer doesn't do this for you currently
  — you'll need to generate and compile that grammar manually.