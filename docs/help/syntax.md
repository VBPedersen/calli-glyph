---
id: syntax
title: Syntax Highlighting
summary: Tree-sitter powered highlighting configured entirely via TOML — no recompilation needed
tags: syntax, highlighting, tree-sitter, grammar, token, theme, colour, color, language, rust, python, typescript, javascript, grammar_dir, node_kind, parent_rule, stop_at, extensions, .so, .dll, .dylib
---
# Syntax Highlighting

calli-glyph uses [tree-sitter](https://tree-sitter.github.io/tree-sitter/) to parse
source files and apply colours based on the structure of your code. The system is
split into three independent layers so any part can be changed without recompiling
the editor:

```
source code  →  grammar (.so)  →  lang config (.toml)  →  theme (.toml)  →  colours
```

- **Grammar** — a compiled shared library that parses the language into a syntax tree.
- **Lang config** — a TOML file that maps tree-sitter node kinds to semantic token types.
- **Theme** — a TOML file that maps token types to colours and styles.

## Configuration

Syntax highlighting is configured under `[syntax]` in your `config.toml`.

```toml
[syntax]
enabled     = true
grammar_dir = "~/.config/calliglyph/grammars"  # default if omitted
```

Each language is declared as a subsection under `[syntax.languages]`:

```toml
[syntax.languages.rust]
extensions = ["rs"]
grammar    = "rust"        # loads tree_sitter_rust.so from grammar_dir

[syntax.languages.typescript]
extensions = ["ts", "tsx"]
grammar    = "typescript"

[syntax.languages.python]
extensions = ["py"]
grammar    = "python"
```

| Key          | Type            | Description                                            |
|--------------|-----------------|--------------------------------------------------------|
| `extensions` | list of strings | File extensions that activate this language (no dot)   |
| `grammar`    | string          | Stem name of the grammar library and config file       |

When a file is opened calli-glyph picks the first language whose `extensions` list
contains the file's extension. The `grammar` value is used to find both the `.so`
and the `.toml` inside `grammar_dir`.

## Grammar Libraries

A grammar library is a compiled shared library produced from a tree-sitter grammar.
The file must be placed in `grammar_dir` and named after the platform convention:

| Platform | Filename pattern           | Example                   |
|----------|----------------------------|---------------------------|
| Linux    | `tree_sitter_<name>.so`    | `tree_sitter_rust.so`     |
| macOS    | `tree_sitter_<name>.dylib` | `tree_sitter_rust.dylib`  |
| Windows  | `tree_sitter_<name>.dll`   | `tree_sitter_rust.dll`    |

### Compiling a grammar

You need a C compiler and the grammar source. Most grammars are on GitHub under
`https://github.com/tree-sitter/tree-sitter-<language>`.

```bash
# Clone the grammar
git clone https://github.com/tree-sitter/tree-sitter-go
cd tree-sitter-go

# Linux / macOS
gcc -shared -fPIC -o ~/.config/calliglyph/grammars/tree_sitter_go.so \
    -I./src src/parser.c

# Some grammars also have a scanner.c (external scanner for complex tokens)
gcc -shared -fPIC -o ~/.config/calliglyph/grammars/tree_sitter_go.so \
    -I./src src/parser.c src/scanner.c

# Windows (MSVC)
cl /LD /I src src\parser.c /Fe:tree_sitter_go.dll
```

Rust, Python, and TypeScript grammars are bundled as fallbacks and do not need to
be compiled manually unless you want to override them.

## Lang Config Files

A lang config TOML lives in `grammar_dir` alongside the `.so`, named
`<grammar>.toml` (e.g. `rust.toml`). It tells the highlighter which tree-sitter
node kinds map to which token types, and which nodes should stop the tree walk.

### Full format

```toml
stop_at = ["string_literal", "line_comment"]

[node_kinds]
"fn"             = "keyword"
"string_literal" = "string"
"line_comment"   = "comment"
"type_identifier"= "type"

[[parent_rules]]
node_kind   = "identifier"
parent_kind = "function_item"
token_type  = "function"
```

### `[node_kinds]`

A table mapping a tree-sitter node kind string to a token type. The node kind is
the exact string returned by `node.kind()` in the tree-sitter API — these vary per
language. The token type must match a key in your theme file.

### `stop_at`

A list of node kinds whose children should **not** be walked. Use this for nodes
like strings and comments where descending into inner punctuation or escape
sequences would produce duplicate or unwanted spans. **IMPORTANT** to keep stop_at at the top of the .toml befor node_kinds

### `[[parent_rules]]`

Sometimes the same node kind needs a different token type depending on context.
For example an `identifier` should be `"function"` when it is the name of a
function definition, but `"variable"` everywhere else.

Each rule has three fields:

| Field        | Description                                          |
|--------------|------------------------------------------------------|
| `node_kind`  | The node kind to match                               |
| `parent_kind`| The required parent node kind                        |
| `token_type` | Token type to emit when both match                   |

Rules are checked before the `[node_kinds]` table, so they take priority.

## Token Types

These are the token type names recognised by the default theme. You can define
your own names and add matching entries to your theme file.

| Token type  | Typical use                                          |
|-------------|------------------------------------------------------|
| `keyword`   | Language keywords: `fn`, `let`, `class`, `return`    |
| `string`    | String and character literals                        |
| `number`    | Integer and float literals                           |
| `comment`   | Line and block comments                              |
| `type`      | Type names, interfaces, structs                      |
| `function`  | Function names at their definition site              |
| `boolean`   | `true`, `false`                                      |
| `operator`  | Operators and punctuation (language-dependent)       |
| `variable`  | Variable names                                       |
| `property`  | Struct fields, object keys                           |
| `parameter` | Function parameters                                  |
| `constant`  | Compile-time constants                               |

Any token type not present in the theme falls back to the `[defaults]` style.

## Adding a New Language

1. **Compile the grammar** — see the *Grammar Libraries* section above.

2. **Create the lang config** — copy the closest existing `.toml` from
   `grammar_dir` as a starting point and adjust the node kind names. To discover
   the exact node kind strings for a grammar, open a file of that language and
   check the tree-sitter playground at
   `https://tree-sitter.github.io/tree-sitter/playground` or run the tree-sitter
   CLI with `tree-sitter parse <file>`.

3. **Register the extension** in `config.toml`:

   ```toml
   [syntax.languages.go]
   extensions = ["go"]
   grammar    = "go"
   ```

4. Open a file with the new extension — highlighting activates immediately with no
   restart required.

## Bundled Languages

The following grammars are built into the editor as fallbacks. They activate
automatically even without a compiled `.so` in `grammar_dir`, but their lang
configs are still read from `grammar_dir` so you can customise the token mappings.
Default config files are written to `grammar_dir` on first launch if they do not
already exist.

| Language   | Extensions       |
|------------|------------------|
| Rust       | `.rs`            |
| Python     | `.py`            |
| TypeScript | `.ts`, `.tsx`    |
| JavaScript | `.js`, `.jsx`    |

## Troubleshooting

**No highlighting for a file type**
- Check that `[syntax] enabled = true` is set.
- Confirm the extension is listed under `extensions` for a language in your config.
- Check the editor log (`:log`) for `[Syntax]` lines — a missing `.so` or `.toml`
  will produce a warning with the exact path it looked for.

**Wrong colours**
- The lang config maps node kinds to token types; the theme maps token types to
  colours. If a token type exists in your lang config but not your theme, it
  renders in the default style. Add a matching entry to your theme file.
- Node kind names are exact — a typo produces no match and no highlight. Use the
  tree-sitter playground to verify the exact kind string.

**Highlighting stops mid-file**
- The node is likely in `stop_at` when it should not be, or a very large node is
  being walked without a stop. Check that `stop_at` only contains leaf-level nodes
  like strings and comments, not container nodes.

**Performance**
- Highlighting is cached per content version — the tree is only walked when the
  buffer changes. Very large files with thousands of tokens may cause a brief pause
  on the first render after a big paste; this is normal.