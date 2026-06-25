# Plan 7.22 — Color/Palette batch (gilt parity 2.0)

**Repo:** gilt (single root crate; Rust port of Python `rich`). Base branch: parity-2.0.
**Reference (absolute):** `/Users/khaklidelborai/Data2/Velocity/rusty_rich/research_doc/03-color.md` + palette docs.
Files: `src/color/mod.rs`, `src/color/palette.rs`. NO `unsafe`; no new deps; WASM-safe; do NOT push. Read cited code fully first.

## Tasks (TDD each: failing test → RED → minimal impl → GREEN → conventional commit per task)

### Task 1 (P1 residual): `EightBit(0-15) → Windows` downgrade passthrough — `src/color/mod.rs` (~483)
When downgrading a `Color::EightBit(n)` with `n < 16` to the Windows color system, it should pass through directly to `Color::Windows(n)` (the first 16 EightBit colors ARE the standard/Windows palette) rather than going through an RGB nearest-match. Add a shortcut: in the `ColorSystem::Windows` downgrade arm, `Color::EightBit(n) if n < 16 => Color::Windows(n)` BEFORE the RGB match. Verify against rich's `Color.downgrade` (the standard-color passthrough). This complements the earlier #39 downgrade fix.
- Test: `Color::EightBit(5).downgrade(ColorSystem::Windows)` == `Color::Windows(5)` (exact, not RGB-approximated).
- Commit: `fix(color): EightBit(0-15) passes through to Windows on downgrade (#39 residual, Phase 7)`

### Task 2 (P3): `ANSI_COLOR_NAMES` accessor public — `src/color/mod.rs` (~609, ~632)
rich exposes name↔number lookups. Make the existing `ansi_color_name(n)` and `get_ansi_color_number(name)` (read the real fn names near those lines) `pub` so external callers can map ANSI color names. If they don't exist, add `pub fn ansi_color_name(n: u8) -> Option<&'static str>` and `pub fn get_ansi_color_number(name: &str) -> Option<u8>` backed by the existing name table.
- Test: `ansi_color_name(1)` == Some("red") (or rich's name); `get_ansi_color_number("red")` round-trips.
- Commit: `feat(color): public ANSI color name<->number accessors (Phase 7)`

### Task 3 (P3): `Palette::match_color` no LRU cache — `src/color/palette.rs` (~34)
`Palette::match_color(triplet)` does a linear nearest-color search every call. Add a small thread-local LRU/bounded cache keyed by `ColorTriplet` → index, mirroring the crate's existing cache patterns (search for `thread_local!`/`LruCache`). WASM-safe. If `lru` is not already a dep, use a simple bounded HashMap.
- Test: `match_color` returns the same index on repeat calls (cache correctness) for a few triplets.
- Commit: `perf(palette): cache match_color nearest-color lookups (Phase 7)`

## Final gates (ALL clean): `cargo nextest run --all-features` · `cargo test --doc` · `cargo clippy --all-features --all-targets -- -D warnings` · `cargo fmt --check` · `cargo build --all-targets --all-features`
Use your advisor tool before each commit. Keep any `src/lib.rs` re-export edits to a single minimal line.
