# Phase 4 + Phase 5 Console Split Analysis

**File:** `src/console.rs` (1775 lines after Phases 1–3)
**Goal:** evaluate whether to extract capture path (Phase 4) and render path (Phase 5) from the single residual `impl Console` block.

## 1. Current shape of `src/console.rs`

Single `impl Console` block at line 299, with explicit `// --` section banners. Method inventory by section:

| Section | Lines | Methods | Category |
|---|---|---|---|
| Constructors | 324–415 | `new`, `builder`, `from_builder` | lifecycle |
| Properties | 417–526 | `width`, `height`, `size`, `options`, `color_system_name`, `color_system`, `encoding`, `is_terminal`, `is_interactive`, `is_dumb_terminal` | properties |
| Terminal detection | 528–541 | `detect_terminal_size` | properties |
| Theme/Style | 543–576 | `get_style`, `style_interner`, `push_theme`, `pop_theme` | properties |
| Core rendering | 578–679 | `render`, `render_lines`, `render_str`, `render_widget_to_text` | **render+capture** |
| Print | 681–803 | `print`, `print_styled`, `print_text`, `out` | **render** |
| Convenience | 805–998 | `log`, `rule`, `line`, `input`, `input_password`, `print_json`, `inspect`, `print_error`, `print_exception` | **render** |
| Misc | 1000–1067 | `measure`, `status`, `save_text`, `save_html`, `save_svg` | mixed |
| Segment output | 1070–1095 | `write_segments` | **render** (also touches capture) |
| Buffering | 1098–1228 | `enter_buffer`, `exit_buffer`, `check_buffer`, `flush_buffer`, `render_buffer` | **render** |
| Capture | 1231–1260 | `begin_capture`, `end_capture` | **capture** |
| Control | 1263–1306 | `control`, `bell`, `clear`, `show_cursor`, `set_alt_screen`, `set_window_title` | lifecycle |
| Synchronized output | 1310–1340 | `begin_synchronized`, `end_synchronized`, `synchronized` | lifecycle |
| Clipboard | 1343–1359 | `copy_to_clipboard`, `request_clipboard` | lifecycle |
| Pager | 1362–1377 | `pager` | lifecycle |
| Screen helpers | 1380–1442 | `enter_screen`, `exit_screen`, `update_screen`, `update_screen_lines` | lifecycle |
| Live stack | 1445–1496 | `push_live`, `pop_live`, `current_live`, `live_depth`, `set_live`, `clear_live` | lifecycle |
| Export | 1499–1757 | `export_text`, `export_html`, `export_svg` | export (already-extracted helpers in `console_export.rs`) |
| Trait impls | 1759–1775 | `Default for Console` | lifecycle |

## 2. Multiple `impl Console` across files: yes, with no real gotchas

Rust permits any number of inherent `impl` blocks for a type as long as they live in the same crate (reference: `doc.rust-lang.org/reference/items/implementations.html`). All sibling modules of `src/console.rs` already have full visibility into private fields (`capture_buffer`, `buffer`, `live_stack`, etc.) because privacy is per-module-tree, and the pattern is already proven 4× this session via `#[path]` for tests/builder/export. No doc-inheritance issue: rustdoc concatenates all `impl` blocks into one rendered page.

## 3. Capture path scope

Direct `capture_buffer` references: line 279 (field), 411 (init), 1081 + 1133 (mutation inside `write_segments`/`flush_buffer`), 1251 + 1259 (begin/end). Methods exclusively about capture:

- `begin_capture` (1250–1252)
- `end_capture` (1258–1262)
- `render_widget_to_text` (675–679) — uses begin/end internally

Net extractable LOC: ~30 lines of methods + ~80 lines of doc-comments referencing the API. The mutation sites at 1081 & 1133 are *inside* render-path methods (`write_segments`, `flush_buffer`) and stay there — capture is a single `Option<Vec<Segment>>` passive sink, not a flow-control branch.

## 4. Render path scope

Methods touching render/print/buffer flush:

- `render`, `render_lines`, `render_str` (595–673)
- `print`, `print_styled`, `print_text`, `out` (702–803)
- `log`, `rule`, `line`, `print_json`, `inspect`, `print_error`, `print_exception` (824–998)
- `write_segments` (1072–1095)
- `enter_buffer`, `exit_buffer`, `check_buffer`, `flush_buffer`, `render_buffer` (1101–1228)

Total render-path LOC: ~705 lines (≈40% of file).

## 5. Residual after Phases 4+5

Subtract render (~705) + capture (~30) = remove ~735 lines. Console struct definition + lifecycle (constructors, properties, theme, control, sync, clipboard, pager, screen, live stack, Default impl) + export entry-points = **~1040 lines residual**. Still large but homogeneous (state + lifecycle), no longer the rendering god-object.

## 6. Recommendation

**Option (a): two separate PRs using the proven `#[path]` pattern**, in this order:

1. Phase 4 first (smaller, ~30 LOC of methods + tightly scoped doc churn) — validates that the capture sink can move without disturbing `write_segments`. Land as `console_capture.rs`.
2. Phase 5 next (~705 LOC) — extract `console_render.rs`. Larger blast radius but Phase 4 will have shaken out the doc-link rewriting.

Reject (b) directory restructure: `src/console/{mod,capture,render}.rs` requires `pub use` re-export gymnastics, breaks every `use crate::console::Console` import in the workspace, and gains nothing over multiple `impl` blocks since field privacy already works across sibling files.

Reject (c) declare-done-at-1775: the file is still the largest in the crate and `print` + `render_buffer` are the two most-edited methods historically — keeping them in a 1775-line file taxes every future PR that touches them.

**Per-PR gate:** `cargo check --workspace --all-targets` + `cargo test` + `cargo clippy -- -D warnings`. No public API change, so no semver bump needed.
