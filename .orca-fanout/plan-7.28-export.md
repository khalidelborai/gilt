# Plan 7.28 — SVG clip-path wiring (gilt parity 2.0)

**Repo:** gilt (single root crate; Rust port of Python `rich`). Base branch: parity-2.0.
**Reference (absolute):** `/Users/khaklidelborai/Data2/Velocity/rusty_rich/research_doc/10-export.md`.
File: `src/console_export.rs`. NO `unsafe`; no new deps; WASM-safe; do NOT push. Read `build_svg_text` fully first.

## Background
In Phase 5, `build_svg_text` was changed to POPULATE `lines_defs` with one `<clipPath id="{unique_id}-line-{i}">` per rendered line (emitted into `<defs>`). BUT those clip-paths are currently INERT — no `<text>`/`<g>` element references them. rich's SVG wraps each line's text in a group that references its clip-path so the line is clipped to its row rectangle.

## Task (TDD: failing test → RED → minimal impl → GREEN → conventional commit)

### Task 1 (P2): Wire per-line clip-paths to their `<text>` elements — `src/console_export.rs` (`build_svg_text`)
For each rendered line `i`, wrap that line's `<text>` element(s) in a `<g clip-path="url(#{unique_id}-line-{i})">...</g>` (or add the `clip-path="url(#{unique_id}-line-{i})"` attribute directly to each line's `<text>` element). Match rich's SVG structure (read the reference). The clip-paths already exist in `<defs>`; this task makes them referenced/functional.
- Test (RED first): a multi-line recorded SVG export contains `clip-path="url(#` referencing `-line-0` AND `-line-1` on the text/group elements (not just in `<defs>`). Currently the export has the clipPath defs but NO `clip-path="url(...)"` reference — assert the reference is present after the fix. Verify the existing per-line clipPath defs are still emitted.
- Update any existing SVG golden/structure tests for the new `<g clip-path>`/attribute, justified against rich.
- Commit: `fix(export): wire SVG per-line clip-paths to their text elements (#c, Phase 7)`

## Final gates (ALL clean): `cargo nextest run --all-features` · `cargo test --doc` · `cargo clippy --all-features --all-targets -- -D warnings` · `cargo fmt --check` · `cargo build --all-targets --all-features`
Use your advisor tool before the commit. SVG export is pure string building — keep it WASM-safe.
