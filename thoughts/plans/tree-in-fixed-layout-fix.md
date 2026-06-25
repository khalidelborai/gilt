# Fix plan — `Tree` drops all children to root-only inside a fixed-size `Layout` region

- **Status:** plan only (no product code changed in this session)
- **Reproduced:** yes — in plain `gilt` 2.3.0, with and without a wrapping `Panel`
- **One-line root cause:** `Tree::gilt_console` renders each node label through
  `console.render_lines` with the *ambient* `options.height` still set
  (`tree.rs:408` builds `child_opts` via `update_width`, which preserves `height`),
  so the first label (the root) is padded to fill the whole region height and the
  later sibling lines are cropped away by the outer height-shaping pass.

---

## 1. Summary

A `gilt::Tree` with children renders **only its root label** when it is placed
inside a **height-constrained** `Layout` region (the canonical
header / sidebar / body / footer TUI, where the sidebar holds a `Tree`). The
child nodes silently vanish. The same node data rendered as a flat `Markdown`
list shows correctly, and a bare `console.print(&tree)` (no `Layout`) shows all
children — so the defect is specific to the **`Tree` + constrained-height render
path**, which `Layout` (and `Panel`, and `render_lines` with `height=Some(_)`)
exercises.

### Confirmed reproduction

Throwaway example run against this worktree (not committed). Three cases through a
record-mode `Console` + `export_text()`:

| Case | Composition | Children present? |
|------|-------------|-------------------|
| (1) control | `console.print(&tree)` — no Layout | **yes** (`src/`, `Cargo.toml`, `README.md`, `LICENSE`) |
| (2) reported | `Layout` sidebar `size(30)` → `Panel::new(tree)`, split into header/body/footer, `Console` height 24 | **no** — only `Files` |
| (3) minimal | `Layout` sidebar `size(30)` → `tree` directly (no Panel), `Console` height 24 | **no** — only `Files` |

Case (3) proves the `Panel` is **incidental** — a `Tree` placed *directly* in a
fixed Layout region already loses its children.

Repro body (essentials):

```rust
let mut tree = Tree::new(Text::new("Files", Style::null()));
tree.add(Text::new("src/", Style::null()));
tree.add(Text::new("Cargo.toml", Style::null()));
tree.add(Text::new("README.md", Style::null()));
tree.add(Text::new("LICENSE", Style::null()));

let mut root = Layout::default_layout();
let mut body = Layout::new(None, Some("body".into()), None, None, Some(1), None);
let sidebar = Layout::new(None, Some("sidebar".into()), Some(30), None, None, None)
    .with_renderable(Panel::new(tree));            // case (3): .with_renderable(tree)
let main = Layout::new(Some("MAIN".into()), Some("main".into()), None, None, Some(2), None);
body.split_row(vec![sidebar, main]);
root.split_column(vec![ /* header(3), */ body /*, footer(3) */ ]);

let mut console = Console::builder().width(100).height(24).no_color(true).record(true).build();
console.print(&root);
let out = console.export_text(false, false);   // <- children absent
```

**Wrong output (case 2, trimmed):**

```
HEADER

╭────────────────────────────╮MAIN
│ Files                      │
│                            │
│                            │            ← 16 more blank interior rows; no children
│                            │
...
FOOTER
```
`out.contains("src/") == false` for every child.

**Expected output (what a correct render should contain):**

```
╭────────────────────────────╮MAIN
│ Files                      │
│ ├── src/                   │
│ ├── Cargo.toml             │
│ ├── README.md              │
│ └── LICENSE                │
│                            │            ← remaining region rows padded blank
...
```

### Mechanism proof (isolated)

Rendering the **bare tree** through `render_lines` with vs. without an ambient
height (`width=40`):

```
--- height=None: 4 lines ---           --- height=Some(20): 20 lines ---
  [00] "Files"                           [00] "Files"
  [01] "├── src/"                        [01] ""
  [02] "├── Cargo.toml"                  [02] ""        ← root padded to fill 20 rows
  [03] "└── README.md"                   [03] ""           children pushed past crop
                                         ...  (20 total, all blank after [00])
```

With `height=Some(20)` the tree's output is **20 lines where only line 0 has
text** ("Files") — the children are gone. That is the bug, reproduced with no
`Layout` and no `Panel` at all, purely from `render_lines(tree, height=Some(N))`.

---

## 2. Root cause (with `file:line` and the precise mechanism)

The constrained height propagates, unreset, from the Layout region all the way
into the per-label render inside `Tree::gilt_console`, where it inflates the
**first** label to the full region height.

### Data flow

1. **Layout sets `height = Some(region.height)`** for the pane's content render:
   - `src/layout.rs:437` — `let child_opts = options.update_dimensions(region.width, region.height);`
   - `update_dimensions` sets `opts.height = Some(height)` — `src/console.rs:256-262`.
   - The pane content is rendered with that: `src/layout.rs:439-447`
     (`console.render_lines(content, Some(&child_opts), …)`).

2. **`Panel` (when present) forwards the height unchanged** to its content:
   - `src/panel.rs:446` — `let child_opts = options.update_width(inner_width);`
   - `update_width` only changes width fields; it **preserves `height`**
     (`src/console.rs:239-246`).
   - `src/panel.rs:475` — `console.render_lines(self.content.as_ref(), Some(&child_opts), …)`
     → the `Tree` receives `options.height = Some(region.height)`.
   - (Direct-in-Layout case 3 skips this step; the Tree gets the height straight
     from step 1.)

3. **`Tree` propagates the ambient height into *every* label render** — the defect:
   - `src/tree.rs:408` — `let mut child_opts = options.update_width(child_width);`
     `update_width` **keeps `height = Some(region.height)`** (it is never reset).
   - `src/tree.rs:417-418` — for each node:
     `let raw_lines = console.render_lines(node.label.as_ref(), Some(&child_opts), None, pad, false);`

4. **`render_lines` pads/truncates each label to that height** — the amplifier:
   - `src/console_render.rs:88-100`:
     ```rust
     if let Some(height) = opts.height {
         lines.truncate(height);
         while lines.len() < height { lines.push(blank); }
     }
     ```
   - So the **root** label ("Files") becomes `region.height` lines: one of text
     + `region.height − 1` blanks. Those are emitted first
     (`src/tree.rs:486-524`). Each child label is *also* expanded to
     `region.height` lines and emitted after, so the whole tree stream is
     `≈ region.height × node_count` lines tall.

5. **The outer height-shaping crops the stream back to `region.height`**, keeping
   only the root's block:
   - `Panel` path: `src/panel.rs:475` `render_lines(…)` already truncates to
     `region.height` (step 4 again, on the full tree), then the Layout re-shapes:
   - `src/layout.rs:450` — `Segment::set_shape(&lines, region.width, Some(region.height), …)`
     truncates to `region.height` (`src/segment.rs:668-677`).
   - Net: the surviving `region.height` lines are *root text + blanks*; all
     children were beyond the crop boundary. → **root-only.**

### Why `Markdown` works but `Tree` doesn't

`Markdown` (like `Text`, `Table`, etc.) emits its content as one natural
segment stream at the given width; the ambient `height` is applied **once** by
the outer `render_lines`/`set_shape`, which simply crops/pads the *whole* widget
to the region — fitting content shows. `Tree` is unique here because it calls
`render_lines` **per node label** and, by not resetting `height`, makes the first
label individually consume the entire region's vertical budget.

### Parity reference (this is a port omission)

rich's `Tree.__rich_console__` renders each label with `height=None` explicitly:

```python
# research_doc/13-tree.md:109-117  (mirrors rich's rich/tree.py)
renderable_lines = console.render_lines(
    Styled(node.label, style),
    options.update(
        width=options.max_width - sum(level.cell_length for level in prefix),
        highlight=self.highlight,
        height=None,            # <-- gilt's port dropped this
    ),
    pad=options.justify is not None,
)
```

gilt's `update_width(child_width)` is the analogue of `options.update(width=…)`
but **without** the `height=None`. That single missing reset is the root cause.

---

## 3. Fix approach

**Primary fix (required, ~1 line):** reset `height` to `None` on the per-label
`child_opts` in `Tree::gilt_console`, matching rich. Each label renders at its
natural height; the region's height is applied only by the outer Layout/Panel
shaping (which is correct).

`src/tree.rs`, around line 403-411 — change:

```rust
let child_width = options.max_width.saturating_sub(prefix_width);
// P3 parity (finding #6): pad=true when justify is set.
let pad = options.justify.is_some();
// P2 parity (finding #5): forward highlight flag into options so
// the label renderer can apply syntax highlighting.
let mut child_opts = options.update_width(child_width);
if self.highlight {
    child_opts.highlight = Some(true);
}
```

to:

```rust
let child_width = options.max_width.saturating_sub(prefix_width);
let pad = options.justify.is_some();
// rich parity (tree.py / research_doc/13-tree.md): render each node label
// with height = None.  Otherwise a constrained ambient height (e.g. a
// fixed-size Layout region or a height-bounded Panel) pads the FIRST label
// to fill the whole region, cropping out every sibling node — leaving a
// root-only tree.
let mut child_opts = options.update_width(child_width);
child_opts.height = None; // equivalently: options.update_width(child_width).reset_height()
if self.highlight {
    child_opts.highlight = Some(true);
}
```

(`ConsoleOptions::reset_height()` already exists at `src/console.rs:265-269`; either
form is fine. Setting the field directly reads most literally as rich's `height=None`.)

That is the whole functional change. It restores all three repro cases.

### Secondary, optional (NOT bundled with the fix — separate parity gap)

`Panel` forwards the **full** ambient `options.height` to its content
(`src/panel.rs:446` via `update_width`), whereas rich passes
`height = options.height - 2` to account for the top/bottom border
(`research_doc/12-panel-and-box.md:86`, `child_options = options.update(width=…, height=child_height …)`
with `child_height = options.height - 2`). This is **not** what drops the Tree
children (the Tree fix alone fixes the reported bug, including the Panel case),
but with a fixed-height region it can let a panel's content occupy 2 rows too
many so the Layout's `set_shape` clips the **bottom border**. Recommend tracking
this separately (it affects *all* panel content in a fixed region, not just
trees) and not folding it into this one-line Tree fix — one change at a time.

---

## 4. Risk / blast radius

- **Scope:** one statement in `src/tree.rs::gilt_console`. No public API/signature
  change, no new dependency.
- **What renders through this path:** only `Tree`. The per-label `render_lines`
  call is internal to `Tree`. Changing `height` to `None` there cannot affect any
  other widget.
- **Behavioral change is strictly corrective:**
  - When the ambient `height` was `None` (the common case: `console.print(&tree)`,
    and every existing inline `tree.rs` test — `Console::options()` defaults
    `height = None`, `src/console.rs:1034`), `child_opts.height` was already
    effectively `None`. **No change** to those outputs.
  - When the ambient `height` was `Some(_)` (Tree inside a sized Layout / Panel /
    `render_lines(height=Some)`), labels stop being padded to the region height —
    i.e. the bug stops happening. The only "lost" output is the spurious blank
    padding that was hiding the children.
- **Existing tests:** none assert the buggy behavior. No test in `tests/` or
  `src/` renders a `Tree` with a constrained height (`grep` confirms: tree tests
  use `console.options()` with no height; `container_composition.rs` /
  `integration.rs` don't combine Tree with a sized region). Risk of regressions
  in the suite ≈ nil; the new regression tests (§5) are the only ones that would
  change status (red→green).
- **WASM / MSRV:** safe. The change uses only existing fields/methods
  (`ConsoleOptions.height` / `reset_height`), no `libc`/`crossterm`/syscalls, no
  new `std` items. Compiles unchanged under `--no-default-features` and
  `wasm32-unknown-unknown`; nothing here touches the 1.82.0 MSRV surface.
- **Performance:** neutral to slightly *better* — labels no longer render N−1
  blank padding rows each, so the per-node `render_lines` returns fewer lines when
  a height was set.

---

## 5. Tests to add

Two layers: one isolating the `Tree` fix, one end-to-end through `Layout`
composition (the reported scenario). Plus edge cases.

### 5a. Inline regression test in `src/tree.rs` (isolates the fix)

```rust
// -- Regression: children survive a constrained ambient height ----------------
// Before the fix, the root label was padded to fill `options.height`, pushing
// the children past the crop boundary so only the root survived.
#[test]
fn test_tree_children_survive_constrained_height() {
    let console = test_console(40);
    // Simulate a fixed-size Layout region of height 20.
    let opts = console.options().update_dimensions(40, 20);

    let mut tree = Tree::new(Text::new("root", Style::null()));
    tree.add(Text::new("child1", Style::null()));
    tree.add(Text::new("child2", Style::null()));
    tree.add(Text::new("child3", Style::null()));

    // render_lines applies the same height crop/pad that Layout and Panel do.
    let lines = console.render_lines(&tree, Some(&opts), None, true, false);
    let text: String = lines
        .iter()
        .flat_map(|l| l.iter())
        .filter(|s| !s.is_control())
        .map(|s| s.text.as_str())
        .collect();

    assert!(text.contains("root"));
    assert!(text.contains("child1"), "child1 dropped under constrained height: {text:?}");
    assert!(text.contains("child2"), "child2 dropped under constrained height: {text:?}");
    assert!(text.contains("child3"), "child3 dropped under constrained height: {text:?}");
    // The region is still filled to its height (children + blank padding).
    assert_eq!(lines.len(), 20);
}
```

### 5b. End-to-end test in `src/layout.rs` (the reported composition)

```rust
#[test]
fn tree_children_survive_fixed_layout_region() {
    use crate::panel::Panel;
    use crate::tree::Tree;

    let mut tree = Tree::new(Text::new("Files", Style::null()));
    tree.add(Text::new("alpha.rs", Style::null()));
    tree.add(Text::new("beta.rs", Style::null()));
    tree.add(Text::new("gamma.rs", Style::null()));

    let mut root = Layout::default_layout();
    let sidebar = Layout::new(None, Some("sidebar".into()), Some(30), None, None, None)
        .with_renderable(Panel::new(tree));
    let main = Layout::new(Some("MAIN".into()), Some("main".into()), None, None, Some(1), None);
    root.split_row(vec![sidebar, main]);

    let console = Console::builder()
        .width(100).height(24).markup(false).no_color(true).build();
    let opts = console.options();
    let segments = root.gilt_console(&console, &opts);
    let text: String = segments
        .iter().filter(|s| !s.is_control()).map(|s| s.text.as_str()).collect();

    assert!(text.contains("Files"));
    assert!(text.contains("alpha.rs"), "Tree child dropped in fixed Layout region: {text:?}");
    assert!(text.contains("beta.rs"),  "Tree child dropped in fixed Layout region: {text:?}");
    assert!(text.contains("gamma.rs"), "Tree child dropped in fixed Layout region: {text:?}");
}
```

### 5c. Edge cases (add as separate `src/tree.rs` tests)

- **Height exactly = node count** (`update_dimensions(W, 4)` for root + 3 children):
  all four labels present, `lines.len() == 4`.
- **Very small region (`height = 1`)**: only the root is present, no panic — this
  matches rich (one row of vertical budget). Assert `text.contains("root")` and
  that a child is *absent* — documents that genuinely-too-short regions still crop
  (the fix restores natural rendering, it doesn't conjure space).
- **Nested Tree (grandchildren) in a constrained region** (`update_dimensions(W, 20)`):
  root, child, and grandchild labels all present — confirms the reset applies at
  every depth, not just the first level.

Run: `cargo nextest run --lib tree && cargo nextest run --lib layout`
(plus `cargo test --doc` if any doctest is touched — none planned).

---

## 6. Acceptance criteria

1. The three §5 tests pass; without the fix, 5a/5b fail (children absent) — i.e.
   they genuinely guard the regression.
2. A `Tree` with ≥3 children inside a fixed-size `Layout` region (with and without
   a wrapping `Panel`) renders **all** child labels in `export_text()`.
3. `console.print(&tree)` (no Layout) output is **byte-for-byte unchanged** from
   before the fix.
4. Full suite stays green: `cargo nextest run --lib`, `cargo test --doc`,
   `cargo clippy --all-features -- -D warnings`, `cargo fmt --check`, MSRV 1.82.0
   `cargo check`, and the wasm32 build
   (`--no-default-features --features json,markdown,syntax`).
5. `CHANGELOG.md` gets a bugfix entry under the next version (e.g.
   "Fix: `Tree` nested in a fixed-size `Layout`/`Panel` region rendered only its
   root, dropping all children — labels now render at natural height (rich parity,
   `height=None`)").

## 7. Estimated effort

- Code fix: **~5 minutes** (1 line + comment in `src/tree.rs`).
- Tests (5a/5b + 3 edge cases): **~30–45 minutes**.
- Full local verification (nextest + doc + clippy + fmt + msrv check + wasm
  build): **~10–15 minutes**.
- Optional secondary `Panel` height-minus-border parity fix (separate change, if
  pursued): **~30 minutes** including its own test. Not required for this bug.

**Total for the required fix + regression coverage: ≈ 1 hour.**
