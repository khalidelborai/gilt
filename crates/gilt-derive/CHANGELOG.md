# Changelog — gilt-derive

All notable changes to `gilt-derive` are documented here. The crate is
versioned in lockstep with `gilt`.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

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
