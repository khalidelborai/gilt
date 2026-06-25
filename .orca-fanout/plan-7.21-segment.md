# Plan 7.21 — Segment P3 batch (gilt parity 2.0)

**Repo:** gilt (single root crate `gilt`; Rust port of Python `rich`). **Branch base:** parity-2.0.
**Reference (absolute):** `/Users/khaklidelborai/Data2/Velocity/rusty_rich/research_doc/01-segment.md`
All changes in `src/segment.rs` (+ maybe a re-export in `src/lib.rs` / `src/prelude.rs`). Additive — no breaking changes.

## Global constraints
- MSRV 1.82; NO `unsafe`; no new deps; WASM-safe. Do NOT set RUSTC_WRAPPER. Do NOT push.
- Match the existing code style in `src/segment.rs`. Read it fully first.

## Tasks (TDD each: failing test → confirm RED → minimal impl → confirm GREEN → commit)

### Task 1: `Segments` and `SegmentLines` renderable wrapper types
rich exposes `Segments` (a renderable wrapping a list of segments) and `SegmentLines`. Add:
```rust
pub struct Segments(pub Vec<Segment>);
pub struct SegmentLines(pub Vec<Vec<Segment>>);   // optional new_lines handling — see rich's SegmentLines
```
Implement `Renderable` for both (`Segments::gilt_console` returns the segments as-is; `SegmentLines::gilt_console` flattens lines, inserting a newline `Segment::line()` between rows). Check rich's `segment.py` `Segments`/`SegmentLines` for exact behavior (the reference doc). Add a manual `Debug` if needed (Segment is Debug, so derive should work).
- Test: a `Segments(vec![Segment::new("hi", None, None)])` renders to a segment list containing "hi"; `SegmentLines(vec![vec![seg "a"], vec![seg "b"]])` renders "a", newline, "b".
- Commit: `feat(segment): add Segments and SegmentLines renderable wrappers (Phase 7)`

### Task 2: LRU cache for `split_cells` (or the cell-splitting fn ~segment.rs:276)
Read the actual function name near line 276 (it splits a segment by cell width). Add a small thread-local LRU cache keyed by `(text, width)` → result, mirroring the existing style-cache pattern used elsewhere in the crate (search for `LruCache` / `thread_local!` usage). Keep it WASM-safe (thread_local is fine). If the function is `&self`-free and cheap, and caching adds complexity without clear benefit, you MAY skip with a noted reason — but prefer adding it for parity. Use the `lru` crate ONLY if already a dependency; otherwise a simple `HashMap`-backed bounded cache.
- Test: calling the split fn twice with the same input returns equal results (cache correctness — same output), and a third distinct input also correct.
- Commit: `perf(segment): LRU-cache cell splitting (Phase 7)`

### Task 3: `impl Display for Segment`
Add `impl std::fmt::Display for Segment` (~segment.rs:112) rendering like `Segment(<text>, <style>)` or just the text — check rich's `Segment.__repr__`/`__str__`. rich's Segment is a NamedTuple; its str is the repr. Implement a reasonable Display (e.g. `write!(f, "Segment({:?}, {:?}, {:?})", self.text, self.style, self.control)` or simpler). Don't conflict with the derived Debug.
- Test: `format!("{}", Segment::new("x", None, None))` contains "x".
- Commit: `feat(segment): impl Display for Segment (Phase 7)`

## Final gates (run ALL, must be clean)
- `cargo nextest run --all-features` (whole crate + integration tests)
- `cargo test --doc`
- `cargo clippy --all-features --all-targets -- -D warnings`
- `cargo fmt --check`
- `cargo build --all-targets --all-features`  (confirm tests/ + benches/ compile)

If any new public type is added, you MAY re-export it at `src/lib.rs` / `src/prelude.rs` following the existing pattern — but keep such edits minimal (a single re-export line) to avoid merge churn.
