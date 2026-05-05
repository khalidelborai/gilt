# Rich-vs-Gilt Audit Re-verification (2026-05-05)

Primary-source verification of the prior audit's claims. All counts via `wc -l`, `find`, `grep`.

## LOC ratios

1. **rich Python source ~38,515 lines** — VERIFIED. `find rich/rich -name '*.py' | xargs wc -l` → `38515 total` (no test dir present).
2. **gilt Rust source ~67,718 lines** — PARTIAL. `gilt/src` alone is 67,857 lines; including `crates/gilt-derive` total is **72,713**. Audit number is close but slightly stale and excludes derive crate.
3. **gilt:rich library ratio 1.76×** — VERIFIED for `src/` only (67,857/38,515 = 1.76). Including derive crate it's 1.89×.
4. **rich examples 1529 / 36 files / 42.5 avg** — VERIFIED exactly.
5. **gilt examples 16,189 / 105 files / 154 avg** — REFUTED. Current: 106 files, 15,754 lines, avg 148.6. Audit overstated by ~435 LOC and one file.
6. **examples ratio 3.63×** — REFUTED. Actual ratio is **15754/1529 = 10.30×**, nearly triple the audit's number. The audit appears to have computed avg-line-ratio (148.6/42.5≈3.5×), not total-LOC ratio.

## Feature surface diff

7. **traceback** — VERIFIED REFUTED (audit claim was wrong). `src/error/traceback.rs` 1398 LOC; `pub struct Traceback` at line 92.
8. **pretty stub** — VERIFIED REFUTED (audit claim was wrong). `src/utils/pretty.rs` 1525 LOC.
9. **prompt** — VERIFIED. `src/prompt.rs` 2004 LOC (vs rich `prompt.py` 549 LOC).
10. **logging** — VERIFIED. `src/error/logging_handler.rs` 1012 LOC + `tracing_layer.rs` 767 LOC. Note: rich's `_log_render.py` (94 LOC) has no standalone gilt file; logic is folded inline into `logging_handler.rs`.
11. **jupyter** — VERIFIED (no Rust analog needed). `grep -ri jupyter gilt/src Cargo.toml` returns zero hits. `rich/jupyter.py` is 101 LOC of IPython-display glue, not applicable to a Rust TTY library.
12. **diagnose** — VERIFIED. `src/utils/diagnose.rs` 694 LOC (rich `diagnose.py` 37 LOC; gilt is a 19× expansion).
13. **inspect** — VERIFIED. `src/utils/inspect.rs` 552 LOC + `crates/gilt-derive/src/inspect.rs` derive macro.
14. **ansi** — VERIFIED. `src/utils/ansi.rs` 810 LOC.
15. **pager** — VERIFIED. `src/pager.rs` 287 LOC.
16. **scope** — VERIFIED. `src/utils/scope.rs` 655 LOC.

## Examples

17. **"Only 1 of 36 rich examples missing in gilt: exception.py"** — REFUTED (out of date). Cross-reference of all 36 rich example basenames against `gilt/examples/` shows **EXACT match for all 36** (attrs, bars, columns, cp_progress, downloader, dynamic_progress, exception, export, file_progress, fullscreen, group, group2, highlighter, jobs, justify, justify2, layout, link, listdir, live_progress, log, overflow, padding, print_calendar, rainbow, recursive_error, repr, save_table_svg, screen, spinners, status, suppress, table, table_movie, top_lite_simulator, tree). Gap is 0/36, not 1/36.
18. **Hard examples** — VERIFIED PRESENT. `examples/exception.rs` (43 LOC), `examples/dynamic_progress.rs` (110 LOC), `examples/downloader.rs` (100 LOC) all exist.

## Hidden gaps (rich modules vs gilt)

19. Cross-reference of `rich/rich/*.py` against `gilt/src/**/*.rs`:

   **Public modules with gilt counterpart**: align, bar, box, cells, color, color_triplet, columns, console, constrain, containers, control, default_styles, diagnose, emoji, file_proxy, filesize, highlighter, inspect, json, layout, live, live_render, logging, markdown, markup, measure, padding, pager, palette, panel, pretty, progress, progress_bar, prompt, protocol, region, rule, scope, screen, segment, spinner (in `status/spinner.rs`, 509 LOC), status (in `status/mod.rs`, 22kB), styled, style, syntax, table, terminal_theme, text, theme, traceback, tree, ansi.

   **Rich modules with no obvious gilt file** (mostly trivial Python-only helpers):
   - `abc.py` (33 LOC) — Python ABCs; not applicable to Rust traits.
   - `errors.py` (34 LOC) — exception classes; gilt has equivalents in `src/error/mod.rs` (`ConsoleError`, `MarkupError` enums).
   - `themes.py` (5 LOC) — re-export shim; covered by `src/color/theme.rs`.
   - `repr.py` (150 LOC) — Python `__rich_repr__` helper; gilt covers via `crates/gilt-derive` proc-macros.
   - `_log_render.py` (94 LOC) — internal helper; folded into `logging_handler.rs`.
   - `_win32_console.py`, `_windows.py`, `_windows_renderer.py` — Windows console adapters; `crossterm` handles this in gilt.
   - `jupyter.py` — see #11.

   **No public rich module is silently absent from gilt.** Audit's coverage map holds.

## Notes

- Branch `phase/1-foundation` has many uncommitted edits; LOC numbers reflect working tree, not last commit. Re-run on `main` if audit numbers need to match a release.
