# Novel Features Scoping — gilt v1.4+

Three candidates evaluated for differentiating gilt as a Rust-native terminal lib (vs. a rich port). Honest ROI rankings; recommend at most one.

---

## A) Plugin / Theme Registry

### Current state
`src/color/theme.rs` (677 LOC) already has `Theme::from_str`/`from_file`/`read` (INI `[styles]` parser) and `Console::push_theme`. INI is the existing canon. No registry; no built-in named themes beyond `DEFAULT_STYLES`. Zero `// TODO theme` markers in src/.

### Proposed MVP
- `src/color/theme_registry.rs` (~200 LOC) with bundled themes (dracula, solarized-dark, nord, monokai, gruvbox) embedded via `include_str!`.
- `Console::with_theme_named("dracula")` builder method.
- `ThemeRegistry::register(name, theme)` for user themes.

### Effort + risks
- **LOC**: ~250 (registry + 5 themes + tests + example).
- **Deps**: none (reuses INI parser).
- **API churn**: low; purely additive.
- **Maintenance**: small. Keep INI format — don't add TOML/base16 parsers for no user win.
- **Differentiation vs. rich**: marginal. rich-py also has named themes. This is "catching up."

**ROI: 5/10.** Real but small impact; cheap to ship.

---

## B) WebAssembly / xterm.js Export

### Current state — blocking-call audit
- `is_terminal` checks: 4 sites, all guarded behind `Console::is_terminal()` which is already overridable via `ConsoleBuilder`. No `isatty`/`ioctl`/`libc::` calls anywhere in `src/`.
- `std::env::var`: `TERM`, `COLUMNS`, `LINES` only — all have fallbacks. WASM-safe (`getrandom`/env shims handle it; or set defaults).
- `std::fs::write`: 3 sites in `console.rs` (`save_text`/`save_html`/`save_svg`) and tests; trivial to feature-gate.
- `tokio`/`reqwest`: optional features (`async`, `http`); user opts out for WASM.
- **No crossterm, termion, or terminal-size dep** — gilt is already terminal-agnostic at the I/O layer.

### Key insight
The existing `record(true)` + `export_text(styles=true)` already produces ANSI-with-styles output that xterm.js can consume directly. **No new code is strictly required for the basic use case** — a 30-line example showing `gilt → wasm-bindgen → xterm.js write()` would prove it works today.

### Proposed MVP
- Add `wasm32-unknown-unknown` to CI matrix; verify `default-features = false` + small feature set compiles.
- New `examples/wasm_xterm/` with `index.html` + xterm.js boot + a `cargo make` target.
- Document the recipe in README.

### Effort + risks
- **LOC**: ~50 (mostly demo HTML/JS) + a few `#[cfg(not(target_arch = "wasm32"))]` guards on `save_*` methods.
- **Deps**: dev-only `wasm-bindgen`, `web-sys` for the example.
- **API churn**: zero — purely demonstrating existing capability.
- **Maintenance**: ongoing CI matrix cost; risk that future `std::fs` additions silently break wasm.
- **Differentiation**: high. No rich port targets browsers. xterm.js is everywhere (VS Code, JupyterLab, Theia). Opens "rich CLI rendering in cloud IDEs" as a use case.

**ROI: 8/10.** Massive impact-to-effort because the work is already 90% done — gilt is accidentally WASM-friendly. Just needs a demo + CI gate.

---

## C) Flexbox Layout Solver

### Current state
`src/layout.rs` (1237 LOC) is split-pane: `RowSplitter`/`ColumnSplitter` + `ratio_resolve` for proportional widths. No flex-grow/flex-shrink/gap/alignment semantics. No constraint solver. Uses `Region` (x/y/w/h) as the geometric primitive.

### `taffy` evaluation
- v0.10.1, MIT, `rust-version: 1.71` (compatible with gilt's 1.82).
- Modular features: `flexbox`, `grid`, `block_layout` all separable. Pure Rust, no C deps.
- ~5k LOC dependency; transitive deps minimal (`slotmap`, optional `serde`, `grid`).
- Used by Bevy UI, iced, Dioxus — proven solver.

### Proposed MVP
- Optional `flex` cargo feature; `taffy = { version = "0.10", default-features = false, features = ["std", "flexbox", "taffy_tree"], optional = true }`.
- New widget `FlexLayout` in `src/flex_layout.rs` wrapping a `taffy::TaffyTree`.
- Each child is a `Renderable + Style`-bearing node; `FlexLayout::render` runs taffy compute → maps `taffy::Layout` rects → renders each child into its `Region`.
- Existing `Layout` untouched (no breaking change).

### Effort + risks
- **LOC**: ~600 (widget + taffy bridge + 3 examples + tests).
- **Deps**: taffy (optional).
- **API churn**: medium. Two layout systems coexist — confusing for users. Doc cost is real.
- **Maintenance**: taffy versioning; mapping taffy's float-based output to integer cells (rounding errors on edges). Style/border integration is non-trivial — taffy doesn't know about ANSI box-drawing.
- **Differentiation**: medium. ratatui has a constraint-based layout already; this would close that gap but not lead it. Rich-py also lacks flex.

**ROI: 4/10.** High effort, real ergonomic win, but ratatui already serves "constraint layout" buyers and current `Layout` covers 80% of TUI dashboards.

---

## Ranking

| Rank | Candidate | ROI | Recommendation |
|------|-----------|-----|----------------|
| 1 | **B) WASM / xterm.js export** | 8/10 | **Prototype in v1.4** — example + CI gate, no API surface change |
| 2 | A) Theme registry | 5/10 | Ship in v1.5 (low-priority polish) |
| 3 | C) Flex layout via taffy | 4/10 | Skip until concrete user demand; revisit v2.0 |

**Pick: B.** It's the only one that's both differentiating AND cheap because gilt's existing record/export pipeline already does the hard work. A working demo unlocks a category (in-browser CLI rendering) no rich port serves.
