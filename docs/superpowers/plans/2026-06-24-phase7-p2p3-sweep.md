# Phase 7 — P2/P3 Long-Tail Sweep (batched by subsystem)

> REQUIRED SUB-SKILL: superpowers:subagent-driven-development. One Task per subsystem batch; fix all its still-open items (TDD each).

**Goal:** close the remaining ~92 P2/P3 (and 1 residual P1) parity findings from audit §3 not covered by Phases 1–6. Each Task = one subsystem; fix every listed finding, TDD.

**Base:** a9bfdb5. **Branch:** parity-2.0.

## Global Constraints
- MSRV 1.82; WASM-safe (no new libc/crossterm in default); clippy `--all-features -D warnings`; **`cargo fmt` + `--check` before each commit**; `cargo nextest run --lib` (+ `--all-features` when touching feature-gated code); `cargo build --examples --all-features` + `cargo test --doc` if signatures change.
- Respect CLAUDE.md deliberate deviations + the v2 ADR — the SKIP items below are intentional, do NOT implement.
- Each finding: `title — file:line — fix sketch — severity`. Read the cited code + audit §3 for full context.

**Order (by value):** 7.1 Progress → 7.2 Markdown → 7.3 Pretty → 7.4 Logging → 7.5 Prompt → 7.6 Traceback → 7.7 Console → 7.8 Control → 7.9 Themes → 7.10 Table → 7.11 Panel → 7.12 Tree → 7.13 Style → 7.14 Markup → 7.15 Syntax → 7.16 Cells → 7.17 Protocols → 7.18 Layout → 7.19 Containers → 7.20 Text → 7.21 Segment → 7.22 Color/Palette → 7.23 Highlighter → 7.24 PublicAPI → 7.25 Windows → 7.26 Inspect → 7.27 Utilities/Scope → 7.28 Export clip-path wiring (deferred).

---

### Task 7.1: Progress (8 items)
1. `add_task` no `start=false` — progress/core.rs:359 — add `start: bool`, only set start_time when true — P2
2. `update()` no `fields` merge — progress/core.rs:376 — add `fields: Option<HashMap<String,String>>`, merge — P2
3. `ProgressColumn::render` returns `Text` not renderable — progress/core.rs:25 — allow `RenderableArc` (companion default) — P2
4. `max_refresh` never rate-limited — progress/core.rs:781 — check `col.max_refresh()` vs elapsed before render — P2
5. `DownloadColumn` independent units — progress/core.rs:80 — shared unit from max(completed,total) — P2
6. `TaskProgressColumn` shows `50/100` not `50%` — progress/columns/progress.rs:62 — format percent — P2
7. `TextColumn` uses `Text::new` not markup — progress/columns/text.rs:113 — `from_markup` fallback — P2
8. `time_remaining()` no ceil — progress/task.rs:143 — `.ceil()` — P3
SKIP: nested stack (done Phase 6); open()/wrap_file() (intentional).

### Task 7.2: Markdown (6-7 items)
1. OSC-8 hyperlinks not emitted — markdown.rs:344 — set `Style::with_link(url)` on link spans when hyperlinks=true — P2
2. `inline_code_lexer` unused — markdown.rs:325 — Syntax-highlight inline code when set — P2
3. blockquote non-paragraph blocks miss ▌ — markdown.rs:172,404,469 — apply prefix in heading/code/hr/list when in blockquote — P2
4. table missing pad_edge(false)+collapse_padding(true) — markdown.rs:728 — add both — P2
5. table header cells stripped to plain — markdown.rs:98,570 — header_cells → Vec<Text> via from_markup — P2
6. ordered list not right-aligned — markdown.rs:496 — `{:>width$}` by max digits — P2
7. image no-alt blank — markdown.rs:370 — filename fallback — P3

### Task 7.3: Pretty (6-8 items)
1. no `pretty_repr()` fn — utils/pretty.rs:56 — add pub fn — P2
2. `+N more` vs `+N` — pretty.rs:453,459,683 — drop " more" (3 sites) — P2
3. max_depth ignored on Debug — pretty.rs:278 — depth-prune — P2
4. expand_all not honored — pretty.rs:278 — try `{:?}` then expand — P2
5. brace `{` collections not truncated — pretty.rs:661 — add `{`/`}` branch — P2
6. no `Console::pprint()` — console_render.rs — add convenience — P2
7. width threshold uses bytes — pretty.rs:440 — cell_len — P3
8. indent guide hard-coded "dim green" — pretty.rs:299 — get_style("repr.indent") fallback — P3

### Task 7.4: Logging (6-8 items)
1. keyword style hard-coded — logging_handler.rs:209 — get_style("logging.keyword") fallback — P2
2. level styles ignore theme — logging_handler.rs:159 — get_style(`logging.level.{lvl}`) — P2
3. no `log_time_format` param — logging_handler.rs:334 — add field — P2
4. no `with_highlighter` — logging_handler.rs:84 — add highlighter field+builder — P2
5. `enabled()` always true — logging_handler.rs:358 — min_level field + check — P2
6. GiltLayer missing markup/keyword/highlighter — tracing_layer.rs:141 — mirror RichHandler builders — P2
7. omit_repeated midnight bug — logging_handler.rs:315 — include date in cache — P3

### Task 7.5: Prompt (7-8 items)
1. no typed IntPrompt/FloatPrompt/Confirm with default — prompt.rs:592,629 — add typed structs — P2
2. errors hard-code `[bold red]` (15×) — prompt.rs:340,608,729 — get_style("prompt.invalid") — P2
3. password renders plain — prompt.rs:470 — render through pipeline — P2
4. `Console::input()` no `stream` — console_render.rs:401 — add stream param — P2
5. ask_int/float_with_input strip ANSI — prompt.rs:600,637 — render_buffer — P2
6. no pre_prompt/on_validate_error hooks — prompt.rs:302 — add default hooks — P3
7. prompt_suffix not configurable — prompt.rs:234 — add field — P3
8. Confirm default `[Y/n]` vs `(y)` — prompt.rs:544 — markup default — P3

### Task 7.6: Traceback (5 items)
1. PEP 678 notes not rendered — error/traceback.rs — add notes field+render — P2
2. ExceptionGroup nested panel — traceback.rs — is_group field+nested Panel — P2
3. install_panic_hook missing config params — traceback.rs:347 — add config — P2
4. locals below code not side-by-side — traceback.rs:754 — Columns layout — P3
5. no code_width param — traceback.rs:716 — add field — P3
SKIP: Stack/Trace hierarchy, cause/context separator, SyntaxError special-case (intentional Rust-idiom deviations).

### Task 7.7: Console render/API (7 items)
1. render_str no emoji+highlight — console_render.rs:123 — emoji_replace + highlighter — P2
2. render_lines no style-apply before crop — console_render.rs:59 — apply_style first — P2
3. log() no variadic/log_locals — console_render.rs:302 — log_objects overload — P2
4. soft_wrap=true no effect — console.rs:372 — wire into print_styled — P2
5. pager() dumps buffer not realtime — console.rs:1272 — record+closure+pipe — P2
6. no RenderHook pipeline — console.rs — render_hooks field+invoke — P2
7. print() no sep/end — console_render.rs:160 — print_sep_end overload — P2
SKIP: thread-local buffers (intentional).

### Task 7.8: Control/Terminal (5 items)
1. clear() emits [Home,Clear] not Clear — control.rs:193 — Clear-only + clear_with_home() — P2
2. console.clear() no `home` param — console.rs:1089 — add — P2
3. show_cursor no is_terminal guard — console.rs:1094 — guard — P2
4. set_alt_screen no is_terminal/legacy guard — console.rs:1101 — guard — P2
5. update_screen no Region param — console.rs:1316 — region overload — P2

### Task 7.9: Themes (4 items)
1. push_theme no `inherit` — console.rs:1061 — add param — P2
2. no use_theme RAII guard — console.rs:982 — ThemeGuard — P2
3. INI only `#` comments not `;` — color/theme.rs:99 — add `;` — P3
4. from_file/read hardcode inherit=true — color/theme.rs:148,158 — add param — P3

### Task 7.10: Table (5 items)
1. leading emits Mid box not blank — table/core.rs:1679 — blank line — P2
2. show_lines+no-box no separators — table/core.rs:1671 — separator outside box gate — P2
3. ratio_distribute panics all-zero — utils/ratio.rs:176 — guard return — P2
4. ratio_distribute ceil not round — ratio.rs:192 — rounding arithmetic — P2
5. row_style double-lookup via to_string — table/core.rs:1271 — cache Style — P3

### Task 7.11: Panel/Box (4 items)
1. title background not composited — panel.rs:322 — apply bg style to title segs — P2
2. ignores Console.safe_box — panel.rs:366 — unwrap_or(console.safe_box) — P2
3. BoxChars::substitute collapses legacy-Win to ASCII — box_chars.rs:288 — legacy_windows param + substitutions — P2
4. get_row Mid uses horizontal not space — box_chars.rs:229 — space fill — P3

### Task 7.12: Tree (4 items)
1. add() missing style/guide_style/expanded/highlight — tree.rs:130 — add params — P2
2. remove_guide_styles adds instead of negates — tree.rs:369 — negating style — P2
3. measure indent-aware bypass — tree.rs:160 — confirm gilt_measure dispatch (likely done Phase 3) — P2
4. default tree/tree.line styles unresolved — tree.rs:96 — get_style fallback — P3

### Task 7.13: Style misc (4 items)
1. render_buffer OSC-8 ignores legacy_windows — console_render.rs:729 — guard — P2
2. Style::render no legacy_windows param — style.rs:550 — add param — P2
3. normalize instance vs class method — style.rs:506 — add normalize_str(def) — P3
4. parse cache 256 vs 4096 — style.rs:1135 — bump to 4096 — P3

### Task 7.14: Markup (4 items)
1. `@`-key stripped — markup.rs:350 — keep `@click` — P2
2. `@click(args)` not parsed — markup.rs:163 — parse parens — P2
3. Text::markup() drops meta spans — text/core.rs:1527 — preserve — P2
4. no emoji_variant param — markup.rs:186,203 — add+thread — P3

### Task 7.15: Syntax (5 items)
1. stylize_range no style_before — syntax.rs:328 — add param — P2
2. highlight_code private — syntax.rs:457 — make pub `highlight` — P2
3. pointer `> ` not `❱ ` — syntax.rs:654 — legacy_windows check — P3
4. guess_lexer no shebang — syntax.rs:809 — first-line fallback — P3
5. line-number style — syntax.rs:531 — Token::Text color — P3

### Task 7.16: Cells/Measure (3 items)
1. split_graphemes absent — utils/cells.rs — implement — P2
2. split_text absent — cells.rs — implement — P2
3. chop_cells/is_single_cell_widths not re-exported — utils/mod.rs:43 — re-export — P3

### Task 7.17: Protocols/ABC (3 items)
1. GiltCast not auto-invoked by print — console_render.rs:161 — blanket impl — P2
2. Syntax::measure missing console/options — syntax.rs:746 — add params — P2
3. ConsoleOptionsUpdates.no_wrap can't reset to None — console.rs:155 — triple-option — P2

### Task 7.18: Layout (4 items)
1. refresh_screen returns segments — layout.rs:586 — call update_screen_lines — P2
2. Columns measures after text-convert — columns.rs:233 — use gilt_measure — P2
3. PaddingDimensions no From<[usize;1]> — padding.rs:36 — add — P3
4. Align.measure minimum uses cell_len — align_widget.rs:117 — longest-word min — P3

### Task 7.19: Containers/Wrappers (4 items)
1. Lines::justify(Full) no console param — text/lines.rs:59 — add — P2
2. Rule title truncation off-by-1 — rule.rs:245,277 — fix width calc — P2
3. Rule no gilt_measure — rule.rs — Measurement(1,1) — P3
4. Renderables::measure not wired — containers.rs:52 — gilt_measure delegate — P3

### Task 7.20: Text (3 items)
1. highlight_regex no callable style — text/core.rs:890 — add overload — P2
2. highlight_regex + style_prefix not combinable — core.rs:890,927 — unified method — P2
3. with_indent_guides byte length — core.rs:1170 — chars().count() — P2
SKIP: named theme styles at parse-time (deferred-ok, tracked).

### Task 7.21: Segment (3 items, P3)
1. Segments/SegmentLines wrappers absent — segment.rs — add types — P3
2. split_cells no LRU cache — segment.rs:276 — add cache — P3
3. no impl Display — segment.rs:112 — add — P3

### Task 7.22: Color/Palette (2-3 items)
1. EightBit(0-15)→Windows passthrough missing — color/mod.rs:483 — n<16 shortcut — P1
2. Palette::match_color no LRU cache — palette.rs:34 — add — P3
3. ANSI_COLOR_NAMES not public — color/mod.rs:609,632 — make ansi_color_name/get_ansi_color_number pub — P3

### Task 7.23: Highlighter/ANSI (3 items, P3)
1. attrib_name capped 50 — highlighter.rs:149 — `+` — P3
2. attrib_value optional should be required — highlighter.rs:149 — drop `?` — P3
3. JSON whitespace ASCII-only — highlighter.rs:251 — char::is_whitespace — P3

### Task 7.24: Public API/Misc (4 items)
1. FileProxy no isatty() — file_proxy.rs:73 — add — P2
2. print_json no ensure_ascii/data — lib.rs:967 — add opts variant — P3
3. VerticalCenter type absent — align_widget.rs — add type+ctor — P3
4. console.log() no log_time=false — console_render.rs:302 — add field+gate — P3

### Task 7.25: Windows (1 item)
1. legacy_windows always false, no setter/auto-detect — console.rs:669, console_builder.rs — add builder setter + env auto-detect (cfg windows) — P2

### Task 7.26: Inspect/Misc (3 items)
1. Pager ignores $PAGER — pager.rs:49 — read env — P2
2. console.pager() no styles param — console.rs:1272 — add styles — P2
3. diagnose omits Jupyter/VSCode env — utils/diagnose.rs:361 — add env vars — P3

### Task 7.27: Utilities/Scope (1 item)
1. Scope missing scope.key/equals styling + ReprHighlighter on values — utils/scope.rs:124 — Text cells + get_style + highlight — P2

### Task 7.28: Export clip-path wiring (1 deferred item)
1. SVG per-line clip-paths not wired to <text> — console_export.rs:216 — add `clip-path="url(#..-line-N)"` to each <text> + test — P2 (deferred from Phase 5)

---

### Phase 7 gate (after all batches)
- [ ] full `cargo nextest run --lib` + `--all-features` + `cargo test --doc` + `cargo build --examples --all-features` green
- [ ] clippy `--all-features -D warnings` + fmt + `--no-default-features` + wasm32
- [ ] CHANGELOG `[Unreleased]` updated with the P2/P3 fixes (grouped); note remaining intentional deviations.
- [ ] Phase R (release): MIGRATION_v1.md v2 section, `just release 2.0.0` (user-triggered).
