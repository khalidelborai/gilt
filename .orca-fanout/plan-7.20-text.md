# Plan 7.20 — Text P2 batch (gilt parity 2.0)

**Repo:** gilt (single root crate; Rust port of Python `rich`). Base branch: parity-2.0.
**Reference (absolute):** `/Users/khaklidelborai/Data2/Velocity/rusty_rich/research_doc/04-text.md`.
File: `src/text/core.rs`. NO `unsafe`; no new deps; WASM-safe; do NOT push. Read the cited fns fully first.
NOTE: the named-theme-style-at-parse-time item is intentionally DEFERRED — do NOT attempt it.

## Tasks (TDD each: failing test → RED → minimal impl → GREEN → conventional commit per task)

### Task 1 (P2): `highlight_regex` callable-style overload — `src/text/core.rs` (~890)
rich's `Text.highlight_regex(re, style)` accepts a callable that maps the matched str → Style. Add `fn highlight_regex_callable<F: Fn(&str) -> Style>(&mut self, re: &Regex, style_fn: F) -> usize` applying a per-match computed style. Match rich's behavior (style per match). Keep the existing `highlight_regex` intact (additive).
- Test: highlighting numbers with a fn that styles odd/even differently produces distinct spans.
- Commit: `feat(text): highlight_regex callable-style overload (Phase 7)`

### Task 2 (P2): `highlight_regex` + `style_prefix` combinable — `src/text/core.rs` (~890, ~927)
Currently a regex highlight and a style-prefix (named-group prefix) can't be combined in one call. Add a unified `fn highlight_regex_unified(&mut self, re: &Regex, style: Option<Style>, style_prefix: Option<&str>) -> usize` that applies BOTH a base style and per-named-group prefix styles (the named-group → `{prefix}.{name}` style lookup, like the existing `highlight_regex_with_groups`). Read the existing group-prefix logic and combine.
- Test: a regex with a named group + a base style applies both.
- Commit: `feat(text): unified highlight_regex with style + style_prefix (Phase 7)`

### Task 3 (P2): `with_indent_guides` uses BYTE length for indent detection — `src/text/core.rs` (~1170)
The indent-guide detection computes leading whitespace via `line.len() - trimmed.len()` (BYTES). For non-ASCII lines this is wrong. Change to char/cell count: `line.chars().count() - line.trim_start().chars().count()` (count of leading whitespace CHARS). Match rich's indent-guide spacing.
- Test: a line with non-ASCII content + leading spaces gets the correct indent-guide column.
- Commit: `fix(text): indent-guide detection uses char count not byte length (Phase 7)`

## Final gates (ALL clean): `cargo nextest run --all-features` · `cargo test --doc` · `cargo clippy --all-features --all-targets -- -D warnings` · `cargo fmt --check` · `cargo build --all-targets --all-features`
Use your advisor tool before each commit. If a public signature changes, fix ALL call sites (use the all-targets build to find them).
