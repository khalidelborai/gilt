# Plan 7.23 — Highlighter/ANSI P3 batch (gilt parity 2.0)

**Repo:** gilt (single root crate; Rust port of Python `rich`). Base branch: parity-2.0.
**Reference (absolute):** `/Users/khaklidelborai/Data2/Velocity/rusty_rich/research_doc/20-highlighter.md` (or the highlighter doc).
File: `src/utils/highlighter.rs`. NO `unsafe`; no new deps; WASM-safe; do NOT push. Read the ReprHighlighter regex patterns fully first.

## Tasks (TDD each: failing test → RED → minimal impl → GREEN → conventional commit per task)

### Task 1 (P3): `attrib_name` pattern capped at 50 chars — `src/utils/highlighter.rs` (~149)
The `attrib_name` regex uses `{1,50}` (bounded). rich uses an unbounded `+`. Change `{1,50}` to `+` (or the rich-equivalent) so long attribute names highlight. Verify against rich's `ReprHighlighter.highlights` regex.
- Test: a `repr` string with a >50-char attribute name still highlights the full name.
- Commit: `fix(highlighter): attrib_name pattern unbounded length (Phase 7)`

### Task 2 (P3): `attrib_value` group optional should be required — `src/utils/highlighter.rs` (~149)
The `attrib_value` capture group has a trailing `?` making it optional; rich requires it. Remove the `?` so `name=value` only matches when a value is present. Verify against rich's regex.
- Test: `name=` (no value) does NOT match attrib_value; `name=42` does.
- Commit: `fix(highlighter): attrib_value group required not optional (Phase 7)`

### Task 3 (P3): JSON key whitespace detection ASCII-only — `src/utils/highlighter.rs` (~251)
The JSON highlighter detects key whitespace with an ASCII-only space list. Replace with `char::is_whitespace()` so Unicode whitespace is handled. Read the actual code near line 251.
- Test: a JSON-ish string with a Unicode-whitespace-separated key still highlights the key correctly.
- Commit: `fix(highlighter): JSON whitespace detection uses char::is_whitespace (Phase 7)`

## Final gates (ALL clean): `cargo nextest run --all-features` · `cargo test --doc` · `cargo clippy --all-features --all-targets -- -D warnings` · `cargo fmt --check` · `cargo build --all-targets --all-features`
Use your advisor tool before each commit. These are regex tweaks — be careful not to break existing highlighter tests; update any that encoded the old (capped/optional/ascii) behavior, justified against rich.
