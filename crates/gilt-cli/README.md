# gilt-cli

A CLI binary for [gilt](../../README.md) — rich terminal output from shell scripts, Makefiles, and CI pipelines without writing Rust.

## Installation

```sh
cargo install --path crates/gilt-cli
```

## Usage

### `gilt print <MARKUP>`

Print text with rich markup tags.

```sh
gilt print '[bold red]Error:[/] something went wrong'
gilt print '[green]Success![/green]'
```

### `gilt style [FLAGS] <TEXT>`

Print text with explicit style flags.

```sh
gilt style --fg red --bold 'Warning'
gilt style --fg blue --italic --underline 'Note'
gilt style --bg white --fg black 'Inverted'
```

Flags: `--fg <color>`, `--bg <color>`, `--bold`, `--italic`, `--underline`, `--dim`.
Colors accept names (`red`, `green`, `blue`, …), hex (`#ff0000`), or 256-color indices.

### `gilt table`

Read CSV from stdin and render as a Unicode box-drawing table.

```sh
gilt table < data.csv
echo 'Name,Age\nAlice,30\nBob,25' | gilt table
```

### `gilt rule [TITLE]`

Draw a horizontal rule, optionally with a centered title.

```sh
gilt rule
gilt rule 'Section Header'
```

### `gilt panel <TEXT> [--title TITLE]`

Render text inside a bordered panel.

```sh
gilt panel 'content here'
gilt panel 'content here' --title 'My Panel'
```

### `gilt markdown`

Read Markdown from stdin and render it to the terminal.

```sh
gilt markdown < README.md
curl -s https://example.com/doc.md | gilt markdown
```

### `gilt json`

Read JSON from stdin and pretty-print it with syntax highlighting.

```sh
gilt json < data.json
curl -s https://api.example.com/data | gilt json
```

## Design notes

- Every subcommand is implemented as a testable function in `src/cmd.rs` that
  accepts an explicit `Write` sink, making unit tests straightforward.
- `clap` is a dependency of `gilt-cli` only — the gilt library itself is not
  affected.
- MSRV: 1.82 (matches the gilt library).
