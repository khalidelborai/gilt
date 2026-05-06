# Changelog — gilt-derive

All notable changes to `gilt-derive` are documented here. The crate is
versioned in lockstep with `gilt`.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [1.4.1] - 2026-05-06

Lockstep with `gilt 1.4.1` patch — no source changes in this crate.
The main crate switches `cell_len` to per-codepoint summation to
match terminal reality on setups without ZWJ font support.

## [1.4.0] - 2026-05-06

Lockstep with `gilt 1.4.0` — no source changes in this crate. The
main crate adds `unicode-segmentation`-backed grapheme-aware width
math + truncation. See the main [CHANGELOG.md](../../CHANGELOG.md).

## [1.3.1] - 2026-05-06

Lockstep with `gilt 1.3.1` patch — no source changes in this crate.
The main crate fixes a `Console: !Sync` regression from v1.2.0.

## [1.3.0] - 2026-05-06

Lockstep version bump with `gilt 1.3.0` — no source changes in this
crate. The main crate's v1.3 release adds WebAssembly documentation
+ CI verification; see the main [CHANGELOG.md](../../CHANGELOG.md).

## [1.2.0] - 2026-05-06

Lockstep version bump with `gilt 1.2.0` — no source changes in this
crate. The main crate's v1.2 release adds `Console::with_writer`
(additive) and reorganises `console.rs` internally; see the main
[CHANGELOG.md](../../CHANGELOG.md) for details.

## [1.1.0] - 2026-05-05

Lockstep version bump with `gilt 1.1.0` — no source changes in this
crate. The main crate's v1.1 closes three v1.0 deferred items
(`Columns::from_renderables`, `Panel::from_renderable`, paired examples
for Traceback and Spinner-as-renderable). See the main repo's
[CHANGELOG.md](../../CHANGELOG.md) for details.

## [1.0.0] - 2026-04-29

Lockstep version bump with `gilt 1.0.0` — no source changes in this
crate. The derive macros (Table, Panel, Tree, Columns, Rule, Inspect,
Renderable) and their generated output are unchanged. The main crate's
v1.0 ergonomics overhaul (lossy `Style::parse`, `Text::styled`,
`Console::default`, `Status::set/run`, `Live::set/run`,
`Live::from_renderable`, `Table::with_columns`, `Padding::wrap`,
`Columns::from_items`) is described in the main repo's
[MIGRATION_v1.md](../../MIGRATION_v1.md).

## [0.13.0] - 2026-04-27

Lockstep version bump with `gilt 0.13.0` — no source changes in this
crate. The main crate's `DeriveColumns` / `DeriveInspect` / `DeriveRule`
top-level aliases (deprecated in v0.12.0) are now removed; the derive
macros themselves are unchanged and remain available via
`gilt::derives::*`.

## [0.12.0] - 2026-04-27

Lockstep version bump with `gilt 0.12.0` — no source changes in this
crate. The main crate's `DeriveColumns` / `DeriveInspect` / `DeriveRule`
top-level aliases are now `#[deprecated]` in favour of
`gilt::derives::*`. The derive macros themselves (Table, Panel, Tree,
etc.) and their generated output are unchanged.

## [0.11.4] - 2026-04-27

Phase 5 of the gilt-derive consolidation plan. **Internal refactor only**
— no API changes, no behaviour changes for valid input, all insta
snapshots byte-identical.

### Changed (internal)

- **Split the 4667-line `src/lib.rs`** into per-derive modules + shared
  helpers:
  - `src/lib.rs` (~2085 lines, was 4667): proc-macro entry-point shims
    + the `#[cfg(test)]` test module.
  - `src/shared.rs` (~105 lines): `named_field_ident`,
    `snake_to_title_case`, `box_style_tokens`, `justify_tokens` —
    cross-cutting helpers used by 4+ derives.
  - `src/table.rs` (~653 lines): Table derive impl + attrs.
  - `src/panel.rs` (~493 lines): Panel derive impl + attrs +
    `parse_field_attrs` (also used by Columns).
  - `src/tree.rs` (~374 lines): Tree derive impl + attrs.
  - `src/columns.rs` (~372 lines): Columns derive impl + attrs.
  - `src/rule.rs` (~287 lines): Rule derive impl + attrs.
  - `src/inspect.rs` (~203 lines): Inspect derive impl + attrs.
  - `src/renderable.rs` (~151 lines): Renderable derive impl + attrs.

### Why this matters

Each per-derive change can now be reviewed/diffed in isolation. The
proc-macro entry points stay at the crate root (where Rust requires
them), but each `derive_*_impl` function lives in its own focused
module with `pub(crate)` visibility for the test module to access.

### Verification

- `cargo build -p gilt-derive`: clean
- `cargo test -p gilt-derive --lib`: 113 passed (unchanged)
- `cargo test -p gilt-derive --test trybuild`: 2 groups (3 pass + 3
  fail) (unchanged)
- All 7 `insta` snapshots at `src/snapshots/` byte-identical to
  v0.11.3 — codegen verified stable across the file split.

### Notes

- Test module stays inline in `lib.rs` for now. Splitting tests
  per-derive could land in v0.12.0 alongside the planned
  `DeriveColumns`/`DeriveInspect`/`DeriveRule` deprecations.

## [0.11.3] - 2026-04-27

Test infrastructure release. Adds `trybuild` compile-pass / compile-fail
tests + `insta` snapshot tests for generated code stability. No changes
to derive output for valid input.

### Added

- **`trybuild` test infrastructure** (`tests/trybuild.rs`) running:
  - `tests/compile_pass/*.rs` — smoke tests for each derive on
    minimally-correct input. New: `table_minimal.rs`,
    `panel_minimal.rs`, `derives_namespace.rs` (the last verifies the
    new `gilt::derives::*` namespace from main-crate v0.11.3).
  - `tests/compile_fail/*.rs` + `.stderr` goldens — error-message
    regression guards. Initial cases: `table_on_enum.rs` (graceful
    error vs panic), `inspect_on_union.rs` (same for unions), and
    `renderable_unknown_via.rs` (verifies the v0.11.2 `via`
    restructuring still produces a spanned error pointing at the
    offending literal).
- **`insta` snapshot tests for generated code** (7 tests, one per
  derive) inside `lib.rs`'s `#[cfg(test)] mod tests`. Snapshots live
  at `crates/gilt-derive/src/snapshots/`. Catches accidental codegen
  drift during refactors. Regenerate with
  `INSTA_UPDATE=always cargo test -p gilt-derive --lib expand_`.

### Notes

- Test count: 113 unit (was 106) + 7 snapshot + 6 trybuild (3 pass + 3
  fail). All green.
- `cargo publish -p gilt-derive --dry-run`: clean — `[dev-dependencies]`
  cycle (gilt-derive's tests use the main `gilt` crate) is fine because
  dev-deps are excluded from the published crate.
- For rustc upgrades that change `.stderr` formatting:
  `TRYBUILD=overwrite cargo test -p gilt-derive --test trybuild`.

## [0.11.2] - 2026-04-27

Robustness release. No public API changes; no behavior change for valid
input.

### Changed

- **All `.expect("named field must have ident")` panics replaced with
  proper `syn::Error` returns.** Previously, deriving on a tuple-struct
  field (e.g. via macro composition) would panic the proc-macro with
  `internal compiler error`. Now it produces a clean `compile_error!`
  pointing at the offending field with the message *"expected named
  field — gilt-derive only supports structs with named fields"*.
  Touched 5 sites across the Table / Panel / Tree / Columns / Rule
  derives.
- **`Renderable` derive's `via` attribute restructured** to eliminate a
  `.unwrap()` whose safety relied on control flow. The check now
  matches on `Option<LitStr>` directly, keeping the literal in scope
  for spanned errors. Behavior identical for all valid input.

### Added

- **`crates/gilt-derive/README.md`** — landing page for crates.io and
  docs.rs (previously docs.rs showed only the `lib.rs` top-level doc).
- **`crates/gilt-derive/CHANGELOG.md`** — independent release history,
  this file.

### Internal

- Extracted shared `named_field_ident()` helper at the top of `lib.rs`
  for the panic-free named-field guard.

## [0.11.1] - 2026-04-27

Lockstep cleanup release with `gilt 0.11.1` (no source changes in this
crate). Bumped to publish alongside the main crate's deprecated-item
removals.

## [0.11.0] - 2026-04-27

Lockstep version bump for crates.io publish (the workspace had been at
0.10.0 but `gilt-derive` was never published past 0.9.0). Required so
`gilt 0.11.0` could resolve its `gilt-derive = "^0.11.0"` dependency.

No source changes in this crate. The generated code uses
`Table.style: String` and `Column.style: String` fields (unaffected by
gilt's L1 Color enum collapse and Segment field-to-method conversion in
0.11.0).

## [0.9.0] and earlier

See git history at <https://github.com/khalidelborai/gilt> — pre-0.11
releases were published as gilt-derive 0.9 with the workspace gilt
already at 0.9–0.10.
