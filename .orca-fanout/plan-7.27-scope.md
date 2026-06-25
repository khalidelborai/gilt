# Plan 7.27 — Scope/Utilities P2 batch (gilt parity 2.0)

**Repo:** gilt (single root crate; Rust port of Python `rich`). Base branch: parity-2.0.
**Reference (absolute):** `/Users/khaklidelborai/Data2/Velocity/rusty_rich/research_doc/` (scope/inspect docs).
File: `src/utils/scope.rs`. NO `unsafe`; no new deps; WASM-safe; do NOT push. Read the Scope render fully first.

## Task (TDD: failing test → RED → minimal impl → GREEN → conventional commit)

### Task 1 (P2): `Scope` per-cell key styling + ReprHighlighter on values — `src/utils/scope.rs` (~124-135)
rich's `render_scope` builds a grid where:
- keys use the `scope.key` style (and `scope.key.special` for dunder/special keys),
- the `=` separator uses `scope.equals`,
- values are highlighted with `ReprHighlighter`.
Currently gilt renders keys/values as plain text. Change the grid cells to:
- key cell: a `Text` styled via `console.get_style("scope.key")` (or `scope.key.special` for keys starting/ending with `_`), with fallbacks if the theme lacks them,
- equals cell: styled via `console.get_style("scope.equals")`,
- value cell: run `ReprHighlighter::default().highlight(&mut value_text)` (or the crate's ReprHighlighter API) so numbers/strings/etc. are colored.
Read the existing Scope grid construction + how other widgets call get_style + ReprHighlighter. Match rich's `render_scope`.
- Test: a scope with a key and a numeric value produces segments where the key carries a style and the value has a repr-highlight span (e.g. a number style). Assert the key cell style is non-null and the value has >1 span (highlighted).
- Commit: `fix(scope): per-cell key/equals styling + ReprHighlighter on values (Phase 7)`

## Final gates (ALL clean): `cargo nextest run --all-features` · `cargo test --doc` · `cargo clippy --all-features --all-targets -- -D warnings` · `cargo fmt --check` · `cargo build --all-targets --all-features`
Use your advisor tool before the commit.
