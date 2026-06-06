# gilt-cli

A CLI binary for [gilt](../../README.md) — rich terminal output from shell scripts, Makefiles, and CI pipelines without writing Rust.

## Installation

```sh
cargo install gilt-cli
```

Or from source:

```sh
cargo install --path crates/gilt-cli
```

## Usage

### `gilt print <MARKUP>`

Print text with rich markup tags.

```sh
gilt print '[bold red]Error:[/] something went wrong'
gilt print '[green]Success![/green]'
gilt print '[link=https://crates.io/crates/gilt][underline cyan]gilt on crates.io[/]'
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
printf 'Name,Age\nAlice,30\nBob,25\n' | gilt table
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

### `gilt tree`

Read an indented text outline from stdin and render it as a tree.

Each line is a node; indent by multiples of 2 spaces to set depth.
The first non-empty line is the root.

```sh
printf 'Project/\n  src/\n    main.rs\n  Cargo.toml\n' | gilt tree
find . -type f | sort | gilt tree
```

### `gilt syntax --lang <LANG>`

Read code from stdin and render with syntax highlighting.

```sh
gilt syntax --lang rust < src/main.rs
gilt syntax --lang python --line-numbers < script.py
git show HEAD:src/lib.rs | gilt syntax --lang rust
```

`--lang` accepts language names (`rust`, `python`, `javascript`) or file
extensions (`rs`, `py`, `js`, `toml`, `yaml`, …). `--theme` sets the colour
theme (default: `base16-ocean.dark`). `--line-numbers` adds line numbers.

### `gilt completions <SHELL>`

Emit a shell completion script for `bash`, `zsh`, or `fish`.

```sh
# bash
gilt completions bash >> ~/.bash_completion

# zsh (oh-my-zsh / fpath)
gilt completions zsh > "${fpath[1]}/_gilt"

# fish
gilt completions fish > ~/.config/fish/completions/gilt.fish
```

## Design notes

- Every subcommand is implemented as a testable function in `src/cmd.rs` that
  accepts an explicit `Write` sink, making unit tests straightforward.
- `clap` and `clap_complete` are dependencies of `gilt-cli` only — the gilt
  library itself is not affected.
- MSRV: 1.82 (matches the gilt library).
