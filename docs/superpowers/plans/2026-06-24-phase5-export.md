# Phase 5 — Export Correctness (detailed plan)

> REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use `- [ ]`.

**Goal:** SVG/HTML export matches rich (line-cropping, reverse, dim, clip-paths, chrome, record guard, param forwarding). Closes audit §3.10 (#22, #23 + 8 more).

**Base:** 80a2dbe. **Branch:** parity-2.0. **2.0 — some breaking (export return types, save_* sigs).**

## Global Constraints
- MSRV 1.82; WASM-safe (export = pure string building); clippy `--all-features -D warnings`; **`cargo fmt` + `--check` before each commit**; `cargo nextest run --lib` + `--all-features` + `cargo test --doc` + `cargo build --examples --all-features`.
- Existing export golden tests WILL change for the SVG fixes — that's EXPECTED (new output is the correct rich-matching one); update with justification, never to mask a real defect.
- Reference: `../research_doc/10-export.md`.

**Order:** 5.1 (record guard + save_* forwarding; HIGHEST RISK) → 5.2 (SVG line-crop) → 5.3 (SVG reverse+dim) → 5.4 (SVG clip-paths/chrome/space-skip) → 5.5 (HTML anchor). 5.3 depends on 5.2's `width` param.

---

### Task 5.1: record guard (panic, rich-faithful) + `save_*` param forwarding (#f, #e, #g)
**Files:** `src/console_export.rs` (5 export_* methods), `src/console_render.rs` (save_html/save_svg), tests, CHANGELOG.
**DESIGN DECISION (controller):** rich `assert self.record` (raises AssertionError) when exporting without record. Match that with an always-on Rust `assert!(self.record, "export requires record mode — build the Console with .record(true)")` at the top of each of the five `export_*` methods. **Keep their `String` return type** (NOT Result — changing it would break every export caller for a pure programming-error guard, disproportionate and not more rich-faithful than a panic). Only `save_*` signatures change (smaller surface).
**Interfaces:** `save_html(path)` → `save_html(path, theme, clear, inline_styles, code_format)`; `save_svg(path, title)` → `save_svg(path, title, theme, clear, unique_id, code_format)`; both delegate to the export_* methods, forwarding the params.
- [ ] **RED:** `#[should_panic]` test: `export_text(false,false)` on a non-record console panics with the record message. Test `save_html`/`save_svg` forward params (saved file reflects unique_id/theme/clear). Run → fail (currently returns empty string silently; save_* ignore params).
- [ ] **IMPLEMENT:** add the `assert!(self.record, ..)` guard to all five export_* methods (return type unchanged); update save_* signatures + bodies to forward params.
- [ ] **GREEN:** `cargo nextest run --lib` + `cargo build --examples --all-features` + `cargo test --doc` green (fix save_* call sites in examples/tests); clippy; fmt. CHANGELOG entry.
- [ ] **Commit:** `fix(export): assert record on export_*; forward params through save_html/save_svg (#f #e #g)`
**Breaking:** save_* gain params (export_* return types UNCHANGED — non-breaking record guard via panic).

---

### Task 5.2: SVG line-cropping to console width (#a)
**Files:** `src/console_export.rs` (`build_svg_text` + callers `export_svg`/`export_svg_opts`).
**Interface:** `build_svg_text` gains `width: usize`.
- [ ] **RED:** width-10 record console prints 20×'X'; exported SVG must NOT contain the full 20-char run (cropped to 10). Run → fail.
- [ ] **IMPLEMENT:** replace the manual newline-split with `Segment::split_and_crop_lines(buffer, width, None, false, false)`; iterate the resulting lines; thread `self.width()` from callers.
- [ ] **GREEN:** suite + examples + doctests; update SVG goldens (lines now cropped to width — justify); clippy; fmt.
- [ ] **Commit:** `fix(export): crop SVG lines to console width via split_and_crop_lines (#a)`

---

### Task 5.3: SVG `reverse` on bg rects + `dim` fallback to theme bg (#b, #d)
**Files:** `src/console_export.rs` (`build_svg_text` ~157-175). (Do NOT change `get_html_style` — SVG path only.)
- [ ] **RED:** `"white on black reverse"` segment → SVG `<rect fill>` uses the swapped (fg) color; `"dim"` segment with NO bg → SVG `<text fill>` is blended toward `SVG_EXPORT_THEME.background_color` at 0.4. Run → fail.
- [ ] **IMPLEMENT:** compute effective `(fg,bg)` applying `reverse` swap before emitting the bg `<rect>`; when `dim` && segment bg is None, blend fg against theme bg (`blend_rgb`, factor 0.4).
- [ ] **GREEN:** suite + update goldens (justify); clippy; fmt.
- [ ] **Commit:** `fix(export): SVG reverse on bg rects; dim blends to theme bg when no seg bg (#b #d)`

---

### Task 5.4: SVG per-line clip-paths + traffic-light chrome + space-segment skip (#c, #h, #i)
**Files:** `src/console_export.rs` (`build_svg_text` `lines_defs`; `build_svg_chrome` dot geometry), `src/export_format.rs` (`{lines}` placeholder).
**Interface:** `build_svg_text` gains `terminal_pixel_width: f64` for clip-rect width.
- [ ] **RED:** multi-line SVG contains `<clipPath id="…-line-0">`/`-line-1`; chrome has `r="7"` cx=26/48/70 cy=22; an all-space unstyled segment emits NO `<text>`. Run → fail (lines_defs empty; r=5; spaces emit text).
- [ ] **IMPLEMENT:** populate `lines_defs` with a `<clipPath>` per line; fix dot `r`/`cx`/`cy`; skip `<text>` for all-space null-style segments (still advance x).
- [ ] **GREEN:** suite + update goldens (justify); clippy; fmt.
- [ ] **Commit:** `fix(export): SVG per-line clip-paths; traffic-light geometry; skip all-space segments (#c #h #i)`

---

### Task 5.5: HTML class-mode anchor wraps inner content, not outer span (#j)
**Files:** `src/console_export.rs` (`export_html` ~394-425, `export_html_opts` ~534-566).
- [ ] **RED:** class-mode (`inline_styles=false`) segment with `bold link https://ex.com` → HTML has `<span class="…"><a href="…">text</a></span>` (span wraps anchor), NOT `<a><span></span></a>`. Run → fail.
- [ ] **IMPLEMENT:** in class mode, anchor wraps the escaped text; span wraps the anchor. Apply to both export_html and export_html_opts.
- [ ] **GREEN:** suite + update goldens (justify); clippy; fmt.
- [ ] **Commit:** `fix(export): class-mode HTML anchor wraps inner content not outer span (#j)`

---

### Phase 5 gate
- [ ] full `cargo nextest run --lib` + `--all-features` + `cargo test --doc` + `cargo build --examples --all-features` green
- [ ] clippy `--all-features -D warnings` + fmt + `--no-default-features` + wasm32
- [ ] CHANGELOG `[Unreleased]`: Added/Changed-Breaking (export_* Result, save_* sigs) / Fixed (#22 #23 + §3.10).
