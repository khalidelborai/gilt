//! Tree widget for rendering hierarchical structures with guide characters.
//!

use crate::cells::cell_len;
use crate::console::{Console, ConsoleOptions, Renderable, RenderableArc};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::{Style, StyleStack};
// Text is only used in the #[cfg(test)] module below.
#[cfg(test)]
use crate::text::Text;

// ---------------------------------------------------------------------------
// Guide character constants
// ---------------------------------------------------------------------------

/// Indices into guide character arrays.
const SPACE: usize = 0;
const CONTINUE: usize = 1;
const FORK: usize = 2;
const END: usize = 3;

/// ASCII guide characters: (space, continue, fork, end).
const ASCII_GUIDES: [&str; 4] = ["    ", "|   ", "+-- ", "`-- "];

/// Unicode guide sets: thin, bold, double.
const TREE_GUIDES: [[&str; 4]; 3] = [
    [
        "    ",
        "\u{2502}   ",
        "\u{251c}\u{2500}\u{2500} ",
        "\u{2514}\u{2500}\u{2500} ",
    ], // thin
    [
        "    ",
        "\u{2503}   ",
        "\u{2523}\u{2501}\u{2501} ",
        "\u{2517}\u{2501}\u{2501} ",
    ], // bold
    [
        "    ",
        "\u{2551}   ",
        "\u{2560}\u{2550}\u{2550} ",
        "\u{255a}\u{2550}\u{2550} ",
    ], // double
];

// ---------------------------------------------------------------------------
// Helper: create a guide segment
// ---------------------------------------------------------------------------

/// Create a guide segment.
///
/// `legacy_windows` forces ASCII guides when true (P2 parity, finding #3).
fn make_guide(index: usize, style: &Style, ascii_only: bool, legacy_windows: bool) -> Segment {
    if ascii_only || legacy_windows {
        Segment::styled(ASCII_GUIDES[index], style.clone())
    } else {
        let guide_set = if style.bold() == Some(true) {
            1
        } else if style.underline2() == Some(true) {
            2
        } else {
            0
        };
        Segment::styled(TREE_GUIDES[guide_set][index], style.clone())
    }
}

// ---------------------------------------------------------------------------
// Tree
// ---------------------------------------------------------------------------

/// A tree widget that renders a hierarchical structure with guide characters.
#[derive(Clone)]
pub struct Tree {
    /// The node's label — any renderable widget (Text, Panel, Rule, …).
    pub label: RenderableArc,
    /// Node style.
    pub style: Style,
    /// Guide line style.
    pub guide_style: Style,
    /// Child nodes.
    pub children: Vec<Tree>,
    /// Whether to show children.
    pub expanded: bool,
    /// Whether to hide the root node.
    pub hide_root: bool,
    /// Whether to highlight labels (P2 parity, finding #5). Default false.
    pub highlight: bool,
}

// Manual Debug — RenderableArc (Arc<dyn Renderable + Send + Sync>) doesn't
// implement Debug, so we print a placeholder for the label field.
impl std::fmt::Debug for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tree")
            .field("label", &"<renderable>")
            .field("style", &self.style)
            .field("guide_style", &self.guide_style)
            .field("children", &self.children)
            .field("expanded", &self.expanded)
            .field("hide_root", &self.hide_root)
            .field("highlight", &self.highlight)
            .finish()
    }
}

impl Tree {
    /// Create a new tree node with the given label.
    ///
    /// Accepts any type that implements [`Renderable`] — [`Text`], [`Panel`],
    /// [`Rule`], another [`Tree`], etc.  The value is stored as a
    /// [`RenderableArc`] (reference-counted, cheaply cloned).
    pub fn new(label: impl Renderable + Send + Sync + 'static) -> Self {
        Tree {
            label: std::sync::Arc::new(label),
            style: Style::null(),
            guide_style: Style::null(),
            children: Vec::new(),
            expanded: true,
            hide_root: false,
            highlight: false,
        }
    }

    /// Add a child node and return a mutable reference to it.
    ///
    /// Accepts any type that implements [`Renderable`] as the label.
    /// The child inherits `style`, `guide_style`, and `highlight` from the parent,
    /// and defaults to `expanded: true`.
    ///
    /// To override any of these per-node, use [`add_with`](Self::add_with).
    pub fn add(&mut self, label: impl Renderable + Send + Sync + 'static) -> &mut Tree {
        self.children.push(Tree {
            label: std::sync::Arc::new(label),
            style: self.style.clone(),
            guide_style: self.guide_style.clone(),
            children: Vec::new(),
            expanded: true,
            hide_root: false,
            highlight: self.highlight,
        });
        self.children
            .last_mut()
            .expect("children is non-empty after push")
    }

    /// Add a child node with per-node style overrides.
    ///
    /// Rich parity: Python's `Tree.add(label, *, style, guide_style, expand, highlight)`.
    ///
    /// Each `Option` parameter:
    /// - `Some(value)` — use the given value for this node.
    /// - `None` — inherit from the parent (same behaviour as [`add`](Self::add)).
    ///
    /// For `expanded`: `None` inherits from parent (default true unless parent is
    /// collapsed).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::tree::Tree;
    /// use gilt::style::Style;
    /// use gilt::text::Text;
    ///
    /// let mut tree = Tree::new(Text::new("root", Style::null()))
    ///     .with_style(Style::parse("bold"))
    ///     .with_guide_style(Style::parse("red"));
    ///
    /// // Child uses its own style, inherits guide_style and highlight from parent.
    /// tree.add_with(
    ///     Text::new("child", Style::null()),
    ///     Some(Style::parse("italic")), // override style
    ///     None,                         // inherit guide_style ("red")
    ///     Some(false),                  // collapsed
    ///     None,                         // inherit highlight
    /// );
    /// ```
    pub fn add_with(
        &mut self,
        label: impl Renderable + Send + Sync + 'static,
        style: Option<Style>,
        guide_style: Option<Style>,
        expanded: Option<bool>,
        highlight: Option<bool>,
    ) -> &mut Tree {
        self.children.push(Tree {
            label: std::sync::Arc::new(label),
            style: style.unwrap_or_else(|| self.style.clone()),
            guide_style: guide_style.unwrap_or_else(|| self.guide_style.clone()),
            children: Vec::new(),
            expanded: expanded.unwrap_or(self.expanded),
            hide_root: false,
            highlight: highlight.unwrap_or(self.highlight),
        });
        self.children
            .last_mut()
            .expect("children is non-empty after push")
    }

    /// Set the node style (builder pattern).
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the guide style (builder pattern).
    #[must_use]
    pub fn with_guide_style(mut self, style: Style) -> Self {
        self.guide_style = style;
        self
    }

    /// Set whether the tree is expanded (builder pattern).
    #[must_use]
    pub fn with_expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Set whether the root node is hidden (builder pattern).
    #[must_use]
    pub fn with_hide_root(mut self, hide_root: bool) -> Self {
        self.hide_root = hide_root;
        self
    }

    /// Set whether to highlight labels (builder pattern, P2 parity finding #5).
    #[must_use]
    pub fn with_highlight(mut self, highlight: bool) -> Self {
        self.highlight = highlight;
        self
    }

    /// Measure this tree: compute minimum and maximum widths.
    ///
    /// Uses `Renderable::gilt_measure` on each node's label so that any widget
    /// type (Text, Panel, Rule, …) is measured correctly (P1 parity, finding #1).
    pub fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        let mut minimum: usize = 0;
        let mut maximum: usize = 0;

        fn measure_recursive(
            tree: &Tree,
            level: usize,
            min: &mut usize,
            max: &mut usize,
            hide_root: bool,
            console: &Console,
            options: &ConsoleOptions,
        ) {
            let effective_level = if hide_root {
                level.saturating_sub(1)
            } else {
                level
            };
            let indent = effective_level * 4;
            // Use gilt_measure on the RenderableArc so any widget type is measured.
            let label_m = tree.label.gilt_measure(console, options);
            if !(level == 0 && hide_root) {
                *min = (*min).max(label_m.minimum + indent);
                *max = (*max).max(label_m.maximum + indent);
            }
            if tree.expanded {
                for child in &tree.children {
                    measure_recursive(child, level + 1, min, max, hide_root, console, options);
                }
            }
        }

        measure_recursive(
            self,
            0,
            &mut minimum,
            &mut maximum,
            self.hide_root,
            console,
            options,
        );
        Measurement::new(minimum, maximum)
    }
}

// ---------------------------------------------------------------------------
// Renderable
// ---------------------------------------------------------------------------

/// Stack frame for iterative DFS traversal.
struct StackFrame<'a> {
    /// Iterator position within the children list.
    index: usize,
    /// The children being iterated.
    children: &'a [Tree],
}

impl Renderable for Tree {
    fn gilt_measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        self.measure(console, options)
    }

    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let mut segments: Vec<Segment> = Vec::new();
        let ascii_only = options.ascii_only();
        // P2 parity (finding #3): legacy_windows forces ASCII guide characters.
        let legacy_windows = options.legacy_windows;
        let newline = Segment::line();

        // P3 parity (Item 4): resolve default theme styles for the root node.
        // If the root has a null style, fall back to the console's "tree" and
        // "tree.line" theme entries.  Use local variables — do NOT mutate self.
        let root_style = if self.style.is_null() {
            console.get_style("tree").unwrap_or_else(|_| Style::null())
        } else {
            self.style.clone()
        };
        let root_guide_style = if self.guide_style.is_null() {
            console
                .get_style("tree.line")
                .unwrap_or_else(|_| Style::null())
        } else {
            self.guide_style.clone()
        };

        // Stack-based DFS (porting Python's stack/iterator approach).
        //
        // `levels` holds the guide segment for each depth level.
        // The stack holds iterators over children at each level.
        let mut levels: Vec<Segment> = vec![make_guide(
            CONTINUE,
            &root_guide_style,
            ascii_only,
            legacy_windows,
        )];
        let mut stack: Vec<StackFrame> = Vec::new();

        // --- Style stacks (rich parity: ancestor style tints subtree labels)  ---
        // `style_stack` accumulates combined node styles top-down so that each
        // label is rendered with the fully-resolved ancestor style.
        // `guide_style_stack` accumulates guide styles the same way.
        let mut style_stack = StyleStack::new(root_style);
        let mut guide_style_stack = StyleStack::new(root_guide_style);

        // Push the root as a single-element "children" iterator.
        let root_slice = std::slice::from_ref(self);
        stack.push(StackFrame {
            index: 0,
            children: root_slice,
        });

        let mut depth: usize = 0;

        while let Some(frame) = stack.last_mut() {
            if frame.index >= frame.children.len() {
                // This level is exhausted — ascend.
                stack.pop();
                levels.pop();
                if !levels.is_empty() {
                    let last_idx = levels.len() - 1;
                    let guide_style = levels[last_idx].style.clone().unwrap_or_else(Style::null);
                    levels[last_idx] = make_guide(FORK, &guide_style, ascii_only, legacy_windows);
                    // Pop the style stacks to return to the parent's context.
                    let _ = guide_style_stack.pop();
                    let _ = style_stack.pop();
                }
                depth = depth.saturating_sub(1);
                continue;
            }

            let child_idx = frame.index;
            let total = frame.children.len();
            let last = child_idx == total - 1;
            let node = &frame.children[child_idx];
            frame.index += 1;

            if last {
                let last_level = levels.len() - 1;
                let guide_style = levels[last_level].style.clone().unwrap_or_else(Style::null);
                levels[last_level] = make_guide(END, &guide_style, ascii_only, legacy_windows);
            }

            // Build the prefix from levels, skipping levels for hidden root.
            // P3 perf (finding #7): compute prefix_width from the slice without
            // allocating a Vec first; build current_prefix only once for actual
            // segment emission so there is a single allocation per node.
            let skip = if self.hide_root { 2 } else { 1 };
            let prefix_slice: &[Segment] = if levels.len() > skip {
                &levels[skip..]
            } else {
                &[]
            };

            // Accumulated styles from the stacks (rich parity):
            // - `node_style`: the fully-resolved style for this node's label
            //   (ancestor stack + node's own style).
            // - `node_guide_style`: resolved guide style for this node.
            let node_style = style_stack.current().clone() + node.style.clone();
            let node_guide_style = guide_style_stack.current().clone() + node.guide_style.clone();

            // Compute available width for the label directly from the slice.
            let prefix_width: usize = prefix_slice.iter().map(|s| cell_len(&s.text)).sum();
            let child_width = options.max_width.saturating_sub(prefix_width);
            // P3 parity (finding #6): pad=true when justify is set.
            let pad = options.justify.is_some();
            // P2 parity (finding #5): forward highlight flag into options so
            // the label renderer can apply syntax highlighting.
            let mut child_opts = options.update_width(child_width);
            if self.highlight {
                child_opts.highlight = Some(true);
            }

            // Render the label into lines.  To apply the accumulated ancestor
            // style (rich parity: "Styled(node.label, style)"), we first render
            // the label normally, then apply `node_style` over every segment via
            // `Segment::apply_style`.  We do this line-by-line after the render.
            let raw_lines =
                console.render_lines(node.label.as_ref(), Some(&child_opts), None, pad, false);
            let rendered_lines: Vec<Vec<Segment>> = if node_style.is_null() {
                raw_lines
            } else {
                raw_lines
                    .into_iter()
                    .map(|line| Segment::apply_style(&line, Some(node_style.clone()), None))
                    .collect()
            };

            // Emit segments (skip if this is the root and hide_root is set).
            let skip_node = depth == 0 && self.hide_root;

            if !skip_node {
                // Guide-prefix styling (rich parity): apply the accumulated
                // label style's background to the prefix so the background tints
                // the guide characters, and strip the guide-line style as a
                // post_style so it does not bleed into label rendering.
                let prefix_bg = node_style.background_style();
                // `prefix_post` is the `remove_guide_styles` negating style
                // (rich parity): it has the same attribute bits SET as the guide
                // style, but all values set to false, so any guide-specific
                // decorations (bold, italic, underline, …) are turned off in the
                // post_style pass rather than being additively layered on.
                // fg color, bgcolor, and link are left as None — we negate only
                // boolean attribute bits, matching rich's `remove_guide_styles`.
                let prefix_post = {
                    let mut neg = Style::null();
                    if node_guide_style.bold().is_some() {
                        neg.set_bold(Some(false));
                    }
                    if node_guide_style.dim().is_some() {
                        neg.set_dim(Some(false));
                    }
                    if node_guide_style.italic().is_some() {
                        neg.set_italic(Some(false));
                    }
                    if node_guide_style.underline().is_some() {
                        neg.set_underline(Some(false));
                    }
                    if node_guide_style.blink().is_some() {
                        neg.set_blink(Some(false));
                    }
                    if node_guide_style.reverse().is_some() {
                        neg.set_reverse(Some(false));
                    }
                    if node_guide_style.conceal().is_some() {
                        neg.set_conceal(Some(false));
                    }
                    if node_guide_style.strike().is_some() {
                        neg.set_strike(Some(false));
                    }
                    neg
                };
                let has_prefix_style = !prefix_bg.is_null() || !prefix_post.is_null();

                // Build current_prefix once; mutated after the first line only.
                let raw_prefix: Vec<Segment> = prefix_slice.to_vec();
                let mut current_prefix: Vec<Segment> = if has_prefix_style {
                    Segment::apply_style(
                        &raw_prefix,
                        Some(prefix_bg.clone()),
                        Some(prefix_post.clone()),
                    )
                } else {
                    raw_prefix.clone()
                };

                for (i, line) in rendered_lines.iter().enumerate() {
                    // Emit prefix guide segments — extend_from_slice is one
                    // memcpy + N clones rather than N iterations of push.
                    segments.extend_from_slice(&current_prefix);
                    // Emit line content segments.
                    segments.extend(line.iter().cloned());
                    // Emit newline.
                    segments.push(newline.clone());

                    // After the first line, change the last prefix element
                    // from FORK/END to CONTINUE/SPACE for continuation lines.
                    if i == 0 && !current_prefix.is_empty() {
                        let last_idx = raw_prefix.len() - 1;
                        let pstyle = raw_prefix[last_idx]
                            .style
                            .clone()
                            .unwrap_or_else(Style::null);
                        let cont_seg = make_guide(
                            if last { SPACE } else { CONTINUE },
                            &pstyle,
                            ascii_only,
                            legacy_windows,
                        );
                        let cont_styled = if has_prefix_style {
                            Segment::apply_style(
                                std::slice::from_ref(&cont_seg),
                                Some(prefix_bg.clone()),
                                Some(prefix_post.clone()),
                            )
                            .into_iter()
                            .next()
                            .unwrap_or(cont_seg)
                        } else {
                            cont_seg
                        };
                        let last_cp_idx = current_prefix.len() - 1;
                        current_prefix[last_cp_idx] = cont_styled;
                    }
                }
            }

            // Recurse into children if expanded.
            if node.expanded && !node.children.is_empty() {
                // Update the current level's guide to continuation.
                let last_level = levels.len() - 1;
                let guide_style = levels[last_level].style.clone().unwrap_or_else(Style::null);
                levels[last_level] = make_guide(
                    if last { SPACE } else { CONTINUE },
                    &guide_style,
                    ascii_only,
                    legacy_windows,
                );

                // Add a new level for the children.
                // Use `node_guide_style` (the fully-resolved guide style from the
                // stack) so that theme defaults from `root_guide_style` propagate
                // into child-level guide characters.
                let child_count = node.children.len();
                let guide_type = if child_count == 1 { END } else { FORK };
                levels.push(make_guide(
                    guide_type,
                    &node_guide_style,
                    ascii_only,
                    legacy_windows,
                ));

                // Push the node's styles onto the stacks for the child subtree.
                // Use the already-resolved styles (node_style / node_guide_style)
                // so that ancestor accumulation carries forward correctly.
                style_stack.push(node_style.clone());
                guide_style_stack.push(node_guide_style.clone());

                stack.push(StackFrame {
                    index: 0,
                    children: &node.children,
                });
                depth += 1;
            }
        }

        segments
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl std::fmt::Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut console = Console::builder()
            .width(f.width().unwrap_or(80))
            .force_terminal(true)
            .no_color(true)
            .build();
        console.begin_capture();
        console.print(self);
        let output = console.end_capture();
        write!(f, "{}", output.trim_end_matches('\n'))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a console with a fixed width and no markup/highlight.
    fn test_console(width: usize) -> Console {
        Console::builder()
            .width(width)
            .markup(false)
            .highlight(false)
            .no_color(true)
            .build()
    }

    /// Helper: render a tree to plain text (no ANSI codes).
    fn render_tree(tree: &Tree, width: usize) -> String {
        let console = test_console(width);
        let opts = console.options();
        let segments = tree.gilt_console(&console, &opts);
        segments
            .iter()
            .filter(|s| !s.is_control())
            .map(|s| s.text.as_str())
            .collect()
    }

    // -- 1. Single node (no children) --

    #[test]
    fn test_single_node() {
        let tree = Tree::new(Text::new("root", Style::null()));
        let output = render_tree(&tree, 80);
        assert!(output.contains("root"));
        // Should have exactly one line.
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 1);
        // No guide characters for root.
        assert!(!output.contains("\u{251c}"));
        assert!(!output.contains("\u{2514}"));
    }

    // -- 2. Node with one child --

    #[test]
    fn test_one_child() {
        let mut tree = Tree::new(Text::new("root", Style::null()));
        tree.add(Text::new("child", Style::null()));
        let output = render_tree(&tree, 80);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("root"));
        assert!(lines[1].contains("child"));
        // Single child should use END guide.
        assert!(output.contains("\u{2514}\u{2500}\u{2500}"));
    }

    // -- 3. Node with multiple children --

    #[test]
    fn test_multiple_children() {
        let mut tree = Tree::new(Text::new("root", Style::null()));
        tree.add(Text::new("child1", Style::null()));
        tree.add(Text::new("child2", Style::null()));
        tree.add(Text::new("child3", Style::null()));
        let output = render_tree(&tree, 80);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 4);
        assert!(lines[0].contains("root"));
        // First two children use FORK, last uses END.
        assert!(lines[1].contains("\u{251c}\u{2500}\u{2500}"));
        assert!(lines[2].contains("\u{251c}\u{2500}\u{2500}"));
        assert!(lines[3].contains("\u{2514}\u{2500}\u{2500}"));
    }

    // -- 4. Nested children (grandchildren) --

    #[test]
    fn test_nested_children() {
        let mut tree = Tree::new(Text::new("root", Style::null()));
        let child = tree.add(Text::new("child", Style::null()));
        child
            .children
            .push(Tree::new(Text::new("grandchild", Style::null())));
        let output = render_tree(&tree, 80);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("root"));
        assert!(lines[1].contains("child"));
        assert!(lines[2].contains("grandchild"));
    }

    // -- 5. hide_root option --

    #[test]
    fn test_hide_root() {
        let mut tree = Tree::new(Text::new("root", Style::null())).with_hide_root(true);
        tree.add(Text::new("child1", Style::null()));
        tree.add(Text::new("child2", Style::null()));
        let output = render_tree(&tree, 80);
        // Root should not appear.
        assert!(!output.contains("root"));
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("child1"));
        assert!(lines[1].contains("child2"));
    }

    // -- 6. Collapsed node (expanded=false) --

    #[test]
    fn test_collapsed_node() {
        let mut tree = Tree::new(Text::new("root", Style::null()));
        tree.children
            .push(Tree::new(Text::new("branch", Style::null())).with_expanded(false));
        // Add a child to the branch that should NOT be rendered.
        tree.children[0]
            .children
            .push(Tree::new(Text::new("hidden", Style::null())));

        let output = render_tree(&tree, 80);
        assert!(output.contains("branch"));
        assert!(!output.contains("hidden"));
    }

    // -- 7. ASCII mode guides --

    #[test]
    fn test_ascii_mode() {
        let mut tree = Tree::new(Text::new("root", Style::null()));
        tree.add(Text::new("child1", Style::null()));
        tree.add(Text::new("child2", Style::null()));

        let console = Console::builder()
            .width(80)
            .markup(false)
            .highlight(false)
            .no_color(true)
            .build();
        let mut opts = console.options();
        opts.encoding = std::borrow::Cow::Borrowed("ascii");
        let segments = tree.gilt_console(&console, &opts);
        let output: String = segments
            .iter()
            .filter(|s| !s.is_control())
            .map(|s| s.text.as_str())
            .collect();

        assert!(output.contains("+-- "));
        assert!(output.contains("`-- "));
        // Should not contain Unicode guide chars.
        assert!(!output.contains("\u{251c}"));
        assert!(!output.contains("\u{2514}"));
    }

    // -- 8. Guide character correctness --

    #[test]
    fn test_guide_characters() {
        let mut tree = Tree::new(Text::new("root", Style::null()));
        tree.add(Text::new("a", Style::null()));
        tree.add(Text::new("b", Style::null()));
        let output = render_tree(&tree, 80);

        let lines: Vec<&str> = output.lines().collect();
        // "a" line should have FORK guide (not last child).
        assert!(lines[1].starts_with("\u{251c}\u{2500}\u{2500} "));
        // "b" line should have END guide (last child).
        assert!(lines[2].starts_with("\u{2514}\u{2500}\u{2500} "));
    }

    // -- 9. Multi-line label --

    #[test]
    fn test_multiline_label() {
        // Force wrapping by using a narrow width.
        let mut tree = Tree::new(Text::new("root", Style::null()));
        tree.add(Text::new(
            "This is a very long label that should wrap",
            Style::null(),
        ));
        let output = render_tree(&tree, 20);
        let lines: Vec<&str> = output.lines().collect();
        // Should have more than 2 lines due to wrapping.
        assert!(lines.len() > 2);
        // First continuation line of child should use SPACE (since it's the last child).
        // The first line has the END guide, continuation lines have SPACE guide.
    }

    // -- 10. Measure --

    #[test]
    fn test_measure() {
        let mut tree = Tree::new(Text::new("root", Style::null()));
        tree.add(Text::new("child", Style::null()));

        let console = test_console(80);
        let opts = console.options();
        let measurement = tree.measure(&console, &opts);
        // root: 4 cells, child: 5 + 4 indent = 9 cells.
        assert_eq!(measurement.minimum, 9);
        assert_eq!(measurement.maximum, 9);
    }

    // -- 11. Builder pattern --

    #[test]
    fn test_builder_pattern() {
        let style = Style::parse("bold");
        let guide_style = Style::parse("red");

        let tree = Tree::new(Text::new("root", Style::null()))
            .with_style(style.clone())
            .with_guide_style(guide_style.clone())
            .with_expanded(false)
            .with_hide_root(true);

        assert_eq!(tree.style, style);
        assert_eq!(tree.guide_style, guide_style);
        assert!(!tree.expanded);
        assert!(tree.hide_root);
    }

    // -- 12. add() returns mutable ref to child --

    #[test]
    fn test_add_returns_mut_ref() {
        let mut tree = Tree::new(Text::new("root", Style::null()));
        let child = tree.add(Text::new("child", Style::null()));
        // We can modify the child through the returned ref.
        child
            .children
            .push(Tree::new(Text::new("grandchild", Style::null())));
        assert_eq!(tree.children.len(), 1);
        assert_eq!(tree.children[0].children.len(), 1);
        // label is now RenderableArc — verify the grandchild renders its text.
        let console = test_console(80);
        let opts = console.options();
        let segs = tree.children[0].children[0]
            .label
            .gilt_console(&console, &opts);
        let text: String = segs
            .iter()
            .filter(|s| !s.is_control())
            .map(|s| s.text.as_str())
            .collect();
        assert!(
            text.contains("grandchild"),
            "grandchild label should render its text: {:?}",
            text
        );
    }

    // -- 13. Deep nesting (3+ levels) --

    #[test]
    fn test_deep_nesting() {
        let mut tree = Tree::new(Text::new("L0", Style::null()));
        let l1 = tree.add(Text::new("L1", Style::null()));
        l1.children.push(Tree::new(Text::new("L2", Style::null())));
        l1.children[0]
            .children
            .push(Tree::new(Text::new("L3", Style::null())));

        let output = render_tree(&tree, 80);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 4);
        assert!(lines[0].contains("L0"));
        assert!(lines[1].contains("L1"));
        assert!(lines[2].contains("L2"));
        assert!(lines[3].contains("L3"));
        // Verify increasing indentation.
        for i in 1..lines.len() {
            let current_content_start = lines[i].find('L').unwrap_or(0);
            let prev_content_start = lines[i - 1].find('L').unwrap_or(0);
            assert!(
                current_content_start > prev_content_start,
                "L{} should be indented more than L{}",
                i,
                i - 1
            );
        }
    }

    // -- 14. add() inherits style and guide_style --

    #[test]
    fn test_add_inherits_styles() {
        let style = Style::parse("bold");
        let guide_style = Style::parse("red");

        let mut tree = Tree::new(Text::new("root", Style::null()))
            .with_style(style.clone())
            .with_guide_style(guide_style.clone());

        tree.add(Text::new("child", Style::null()));

        assert_eq!(tree.children[0].style, style);
        assert_eq!(tree.children[0].guide_style, guide_style);
    }

    // -- 15. Empty tree (root only, no children) renders single line --

    #[test]
    fn test_empty_tree_no_guides() {
        let tree = Tree::new(Text::new("alone", Style::null()));
        let output = render_tree(&tree, 80);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].trim(), "alone");
    }

    // -- 16. Multiple children with nesting shows correct continuation --

    #[test]
    fn test_continuation_guides() {
        let mut tree = Tree::new(Text::new("root", Style::null()));
        {
            let child1 = tree.add(Text::new("child1", Style::null()));
            child1
                .children
                .push(Tree::new(Text::new("grandchild1", Style::null())));
        }
        tree.add(Text::new("child2", Style::null()));

        let output = render_tree(&tree, 80);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 4);
        // The grandchild line should have CONTINUE (|) guide from the parent level
        // because child1 is not the last child (child2 follows).
        assert!(
            lines[2].contains("\u{2502}"),
            "grandchild1 line should contain continue guide: {:?}",
            lines[2]
        );
    }

    // -- 17. hide_root with deep nesting --

    #[test]
    fn test_hide_root_deep() {
        let mut tree = Tree::new(Text::new("ROOT", Style::null())).with_hide_root(true);
        let child = tree.add(Text::new("child", Style::null()));
        child
            .children
            .push(Tree::new(Text::new("grandchild", Style::null())));

        let output = render_tree(&tree, 80);
        assert!(!output.contains("ROOT"));
        assert!(output.contains("child"));
        assert!(output.contains("grandchild"));
    }

    // -- 18. Measure with hide_root --

    #[test]
    fn test_measure_hide_root() {
        let mut tree = Tree::new(Text::new("LONG_ROOT_NAME", Style::null())).with_hide_root(true);
        tree.add(Text::new("short", Style::null()));

        let console = test_console(80);
        let opts = console.options();
        let measurement = tree.measure(&console, &opts);
        // With hide_root: root is excluded from measurement.
        // child: "short" (5) + 0 indent (since root is hidden, child is at effective level 0).
        assert_eq!(measurement.minimum, 5);
    }

    // -- 19. Measure deep nesting --

    #[test]
    fn test_measure_deep() {
        let mut tree = Tree::new(Text::new("r", Style::null()));
        let c = tree.add(Text::new("cc", Style::null()));
        c.children.push(Tree::new(Text::new("ggg", Style::null())));

        let console = test_console(80);
        let opts = console.options();
        let measurement = tree.measure(&console, &opts);
        // r: 1 + 0 = 1
        // cc: 2 + 4 = 6
        // ggg: 3 + 8 = 11
        assert_eq!(measurement.maximum, 11);
    }

    // -- 20. Collapsed subtree not measured --

    #[test]
    fn test_measure_collapsed() {
        let mut tree = Tree::new(Text::new("r", Style::null()));
        let mut branch = Tree::new(Text::new("branch", Style::null())).with_expanded(false);
        branch.children.push(Tree::new(Text::new(
            "very_very_very_long_hidden_label",
            Style::null(),
        )));
        tree.children.push(branch);

        let console = test_console(80);
        let opts = console.options();
        let measurement = tree.measure(&console, &opts);
        // The hidden label should not affect measurement.
        // r: 1, branch: 6 + 4 = 10.
        assert_eq!(measurement.maximum, 10);
    }

    // -- 21. Guide style selects bold guide set --

    #[test]
    fn test_bold_guide_style() {
        let mut tree =
            Tree::new(Text::new("root", Style::null())).with_guide_style(Style::parse("bold"));
        tree.add(Text::new("child1", Style::null()));
        tree.add(Text::new("child2", Style::null()));

        let output = render_tree(&tree, 80);
        // Bold guides: FORK = \u{2523}\u{2501}\u{2501}, END = \u{2517}\u{2501}\u{2501}
        assert!(output.contains("\u{2523}\u{2501}\u{2501}"));
        assert!(output.contains("\u{2517}\u{2501}\u{2501}"));
    }

    // -- 22. Guide style selects double guide set --

    #[test]
    fn test_double_guide_style() {
        let mut tree = Tree::new(Text::new("root", Style::null()))
            .with_guide_style(Style::parse("underline2"));
        tree.add(Text::new("child1", Style::null()));
        tree.add(Text::new("child2", Style::null()));

        let output = render_tree(&tree, 80);
        // Double guides: FORK = \u{2560}\u{2550}\u{2550}, END = \u{255a}\u{2550}\u{2550}
        assert!(output.contains("\u{2560}\u{2550}\u{2550}"));
        assert!(output.contains("\u{255a}\u{2550}\u{2550}"));
    }

    // -- 23. Guide characters are exactly 4 cells wide --

    #[test]
    fn test_guide_width() {
        for guide in &ASCII_GUIDES {
            assert_eq!(cell_len(guide), 4, "ASCII guide {:?} is not 4 cells", guide);
        }
        for set in &TREE_GUIDES {
            for guide in set {
                assert_eq!(
                    cell_len(guide),
                    4,
                    "Unicode guide {:?} is not 4 cells",
                    guide
                );
            }
        }
    }

    // -- 24. Render produces Segment::line() newlines --

    #[test]
    fn test_segments_contain_newlines() {
        let mut tree = Tree::new(Text::new("root", Style::null()));
        tree.add(Text::new("child", Style::null()));

        let console = test_console(80);
        let opts = console.options();
        let segments = tree.gilt_console(&console, &opts);

        let newline_count = segments.iter().filter(|s| s.text == "\n").count();
        assert_eq!(newline_count, 2, "Expected 2 newlines (one per line)");
    }

    // -- 25. hide_root with no children produces no output --

    #[test]
    fn test_hide_root_no_children() {
        let tree = Tree::new(Text::new("hidden", Style::null())).with_hide_root(true);
        let output = render_tree(&tree, 80);
        assert!(
            output.trim().is_empty(),
            "hide_root with no children should produce empty output"
        );
    }

    #[test]
    fn test_display_trait() {
        let mut tree = Tree::new(Text::new("root", Style::null()));
        tree.add(Text::new("child1", Style::null()));
        tree.add(Text::new("child2", Style::null()));
        let s = format!("{}", tree);
        assert!(!s.is_empty());
        assert!(s.contains("root"));
        assert!(s.contains("child1"));
        assert!(s.contains("child2"));
    }

    // -- CJK / emoji content tests ------------------------------------------

    #[test]
    fn test_tree_cjk_labels() {
        let mut tree = Tree::new(Text::new("根", Style::null()));
        tree.add(Text::new("子ノード一", Style::null()));
        tree.add(Text::new("子ノード二", Style::null()));
        let output = render_tree(&tree, 40);
        assert!(output.contains("根"));
        assert!(output.contains("子ノード一"));
        assert!(output.contains("子ノード二"));
    }

    #[test]
    fn test_tree_emoji_labels() {
        let mut tree = Tree::new(Text::new("🌳 Root", Style::null()));
        tree.add(Text::new("🍎 Apple", Style::null()));
        tree.add(Text::new("🍊 Orange", Style::null()));
        let output = render_tree(&tree, 40);
        assert!(output.contains("🌳"));
        assert!(output.contains("🍎"));
        assert!(output.contains("🍊"));
    }

    // -- Deep nesting test --------------------------------------------------

    #[test]
    fn test_tree_deep_nesting() {
        // Build a tree 50 levels deep — should not stack overflow.
        // Each guide is 4 cells wide, so 50 levels needs ~200 cells of guides
        // plus room for labels. Use width=300 to ensure labels fit.
        let mut root = Tree::new(Text::new("level_0", Style::null()));
        let mut current = &mut root;
        for i in 1..50 {
            current = current.add(Text::new(&format!("level_{}", i), Style::null()));
        }
        let output = render_tree(&root, 300);
        assert!(output.contains("level_0"));
        assert!(output.contains("level_49"));
    }

    // -- Extreme width boundary tests ---------------------------------------

    #[test]
    fn test_tree_width_one() {
        let mut tree = Tree::new(Text::new("root", Style::null()));
        tree.add(Text::new("child", Style::null()));
        // Should not panic at width=1
        let _output = render_tree(&tree, 1);
    }

    // -- Task 1 new tests: ancestor style-stack + guide-prefix styling ------

    /// A styled parent should tint its child label: the child's rendered
    /// segments must carry a style that includes the parent's bold attribute.
    #[test]
    fn test_styled_parent_tints_child_label() {
        // Build a console that preserves ANSI styles (force_terminal so that
        // color/bold is not stripped to plain text).
        let console = Console::builder()
            .width(80)
            .markup(false)
            .highlight(false)
            .force_terminal(true)
            .build();
        let opts = console.options();

        let parent_style = Style::parse("bold");
        let mut tree = Tree::new(Text::new("root", Style::null())).with_style(parent_style.clone());
        tree.add(Text::new("child", Style::null()));

        let segments = tree.gilt_console(&console, &opts);

        // Collect non-control, non-newline segments that contain "child".
        let child_segments: Vec<&Segment> = segments
            .iter()
            .filter(|s| !s.is_control() && s.text.contains("child"))
            .collect();

        // At least one segment carrying "child" must have a bold style
        // (inherited from the parent's accumulated style stack).
        let has_bold = child_segments.iter().any(|s| {
            s.style
                .as_ref()
                .map(|st| st.bold() == Some(true))
                .unwrap_or(false)
        });
        assert!(
            has_bold,
            "Child label segment should carry parent's bold style; segments = {:?}",
            child_segments
        );
    }

    /// Guide prefix segments should carry the resolved guide style so that
    /// a red guide style shows up on the prefix characters.
    #[test]
    fn test_guide_prefix_carries_guide_style() {
        let console = Console::builder()
            .width(80)
            .markup(false)
            .highlight(false)
            .force_terminal(true)
            .build();
        let opts = console.options();

        // A tree with a red guide style.
        let guide_style = Style::parse("red");
        let mut tree =
            Tree::new(Text::new("root", Style::null())).with_guide_style(guide_style.clone());
        tree.add(Text::new("child", Style::null()));

        let segments = tree.gilt_console(&console, &opts);

        // Guide prefix segments are non-control segments that contain guide
        // characters (e.g. "└── " or "+-- ") and should carry a red style.
        let guide_segs: Vec<&Segment> = segments
            .iter()
            .filter(|s| {
                !s.is_control()
                    && (s.text.contains('\u{2514}')   // └
                        || s.text.contains('\u{251c}') // ├
                        || s.text.contains("`-- ")
                        || s.text.contains("+-- "))
            })
            .collect();

        assert!(
            !guide_segs.is_empty(),
            "Expected at least one guide prefix segment in the output"
        );

        // Every guide segment must carry a non-None style (the guide style
        // applies background/foreground color from the stacked guide style).
        // After apply_style the guide segments get the red color from the
        // label style's background (or guide style post-application).
        // We check that style is present (non-null after apply_style).
        for seg in &guide_segs {
            let style_present = seg.style.as_ref().map(|s| !s.is_null()).unwrap_or(false);
            // The guide segments were built with the guide_style (red), so the
            // segment style should be Some and non-null.
            assert!(
                style_present,
                "Guide prefix segment {:?} should have a non-null style",
                seg
            );
        }
    }

    /// Ancestor guide style accumulates: a child's guide_style combines with
    /// the parent's guide_style for nested levels.
    #[test]
    fn test_guide_style_stack_accumulation() {
        let console = Console::builder()
            .width(80)
            .markup(false)
            .highlight(false)
            .force_terminal(true)
            .build();
        let opts = console.options();

        let root_guide = Style::parse("bold");
        let mut tree =
            Tree::new(Text::new("root", Style::null())).with_guide_style(root_guide.clone());
        let child = tree.add(Text::new("child", Style::null()));
        child
            .children
            .push(Tree::new(Text::new("grandchild", Style::null())));

        // Should not panic, and should render all three nodes.
        let segments = tree.gilt_console(&console, &opts);
        let text: String = segments
            .iter()
            .filter(|s| !s.is_control())
            .map(|s| s.text.as_str())
            .collect();
        assert!(text.contains("root"));
        assert!(text.contains("child"));
        assert!(text.contains("grandchild"));
    }

    // -- gilt_measure override ------------------------------------------

    #[test]
    fn tree_gilt_measure_matches_standalone() {
        let console = test_console(80);
        let opts = console.options();
        let mut tree = Tree::new(Text::new("root", Style::null()));
        tree.add(Text::new("child node", Style::null()));
        assert_eq!(
            tree.gilt_measure(&console, &opts),
            tree.measure(&console, &opts),
            "Tree::gilt_measure must delegate to Tree::measure"
        );
    }

    #[test]
    fn tree_gilt_measure_nested_matches_standalone() {
        let console = test_console(80);
        let opts = console.options();
        let mut tree = Tree::new(Text::new("parent", Style::null()));
        let child = tree.add(Text::new("child", Style::null()));
        child.children.push(Tree::new(Text::new(
            "a long grandchild label",
            Style::null(),
        )));
        assert_eq!(
            tree.gilt_measure(&console, &opts),
            tree.measure(&console, &opts),
            "Tree::gilt_measure nested must delegate to Tree::measure"
        );
    }

    // -- Task 4.3: RenderableArc label tests ----------------------------------

    /// Tree::new must accept a Panel as the root label (any Renderable).
    #[test]
    fn tree_new_accepts_panel_label() {
        use crate::panel::Panel;
        let label = Panel::new(Text::new("root panel", Style::null()));
        let tree = Tree::new(label);
        let output = render_tree(&tree, 80);
        assert!(
            output.contains("root panel"),
            "Panel label text should appear in tree output: {:?}",
            output
        );
    }

    /// tree.add must accept a Rule as the child label.
    #[test]
    fn tree_add_accepts_rule_label() {
        use crate::rule::Rule;
        let mut tree = Tree::new(Text::new("root", Style::null()));
        tree.add(Rule::with_title("divider"));
        let output = render_tree(&tree, 40);
        // Rule renders a horizontal line; the tree should not panic and should
        // include at least the root label.
        assert!(
            output.contains("root"),
            "Root label should appear in output: {:?}",
            output
        );
    }

    /// Debug output for Tree should print "<renderable>" for the label field.
    #[test]
    fn tree_debug_impl() {
        let tree = Tree::new(Text::new("debug_test", Style::null()));
        let debug_str = format!("{:?}", tree);
        assert!(
            debug_str.contains("<renderable>"),
            "Debug impl should print '<renderable>' for label: {}",
            debug_str
        );
        assert!(
            !debug_str.contains("debug_test"),
            "Debug impl should NOT print the label content: {}",
            debug_str
        );
    }

    // -- Item 1: add_with() companion method ------------------------------------

    /// add_with() with explicit style overrides must use the provided style,
    /// not inherit from the parent.
    #[test]
    fn test_add_with_style_override() {
        let parent_style = Style::parse("bold");
        let child_style = Style::parse("italic");
        let mut tree = Tree::new(Text::new("root", Style::null())).with_style(parent_style.clone());
        tree.add_with(
            Text::new("child", Style::null()),
            Some(child_style.clone()),
            None,
            None,
            None,
        );
        assert_eq!(
            tree.children[0].style, child_style,
            "add_with() style override should be used, not inherited from parent"
        );
    }

    /// add_with() with None style must inherit from parent (same as add()).
    #[test]
    fn test_add_with_style_inherits_when_none() {
        let parent_style = Style::parse("bold");
        let mut tree = Tree::new(Text::new("root", Style::null())).with_style(parent_style.clone());
        tree.add_with(Text::new("child", Style::null()), None, None, None, None);
        assert_eq!(
            tree.children[0].style, parent_style,
            "add_with() with None style should inherit parent's style"
        );
    }

    /// add_with() with explicit guide_style must use the provided guide_style.
    #[test]
    fn test_add_with_guide_style_override() {
        let parent_guide = Style::parse("red");
        let child_guide = Style::parse("blue");
        let mut tree =
            Tree::new(Text::new("root", Style::null())).with_guide_style(parent_guide.clone());
        tree.add_with(
            Text::new("child", Style::null()),
            None,
            Some(child_guide.clone()),
            None,
            None,
        );
        assert_eq!(
            tree.children[0].guide_style, child_guide,
            "add_with() guide_style override should be used"
        );
    }

    /// add_with() with explicit expanded=false must not be expanded.
    #[test]
    fn test_add_with_expanded_override() {
        let mut tree = Tree::new(Text::new("root", Style::null()));
        tree.add_with(
            Text::new("child", Style::null()),
            None,
            None,
            Some(false),
            None,
        );
        assert!(
            !tree.children[0].expanded,
            "add_with() expanded=false should set node to not expanded"
        );
    }

    /// add_with() with expanded=None must inherit from the parent's expanded state.
    #[test]
    fn test_add_with_expanded_none_inherits_parent() {
        // Parent is expanded (default true) — child should also be expanded.
        let mut tree = Tree::new(Text::new("root", Style::null())); // expanded=true
        tree.add_with(Text::new("child", Style::null()), None, None, None, None);
        assert!(
            tree.children[0].expanded,
            "add_with() with expanded=None should inherit parent's expanded=true"
        );

        // Parent is collapsed — child with None should also be collapsed.
        let mut collapsed_tree = Tree::new(Text::new("root", Style::null())).with_expanded(false);
        collapsed_tree.add_with(Text::new("child", Style::null()), None, None, None, None);
        assert!(
            !collapsed_tree.children[0].expanded,
            "add_with() with expanded=None should inherit parent's expanded=false"
        );
    }

    /// add_with() with highlight=Some(true) must set highlight on the child.
    #[test]
    fn test_add_with_highlight_override() {
        let mut tree = Tree::new(Text::new("root", Style::null())); // parent highlight=false
        tree.add_with(
            Text::new("child", Style::null()),
            None,
            None,
            None,
            Some(true),
        );
        assert!(
            tree.children[0].highlight,
            "add_with() highlight=Some(true) should enable highlight"
        );
    }

    /// add_with() with highlight=None must inherit from parent.
    #[test]
    fn test_add_with_highlight_inherits_when_none() {
        let mut tree = Tree::new(Text::new("root", Style::null())).with_highlight(true);
        tree.add_with(Text::new("child", Style::null()), None, None, None, None);
        assert!(
            tree.children[0].highlight,
            "add_with() with highlight=None should inherit parent's highlight"
        );
    }

    // -- Item 2: remove_guide_styles — negating style, not additive -------------

    /// Guide-style bold must NOT appear on the label segments.
    /// When guide_style is "bold", the label segments must not be bold — the
    /// negating remove_guide_styles style should strip the bold attribute.
    #[test]
    fn test_guide_style_bold_not_on_label() {
        let console = Console::builder()
            .width(80)
            .markup(false)
            .highlight(false)
            .force_terminal(true)
            .build();
        let opts = console.options();

        // Tree with bold guide_style; label has no explicit style.
        let mut tree =
            Tree::new(Text::new("root", Style::null())).with_guide_style(Style::parse("bold"));
        tree.add(Text::new("child", Style::null()));

        let segments = tree.gilt_console(&console, &opts);

        // Find label segments that contain "child" text.
        let child_label_segs: Vec<&Segment> = segments
            .iter()
            .filter(|s| !s.is_control() && s.text.contains("child"))
            .collect();

        assert!(
            !child_label_segs.is_empty(),
            "Expected segments containing 'child'"
        );

        // None of the label segments should have bold set to true.
        let has_bold = child_label_segs.iter().any(|s| {
            s.style
                .as_ref()
                .map(|st| st.bold() == Some(true))
                .unwrap_or(false)
        });
        assert!(
            !has_bold,
            "Child label segments must NOT be bold when guide_style has bold; \
             guide bold should be negated so it doesn't bleed into labels. \
             Segments: {:?}",
            child_label_segs
        );
    }

    /// The guide-removal negation (`prefix_post`) must actually strip bold from
    /// guide/prefix segments after `Segment::apply_style`.  This unit-tests the
    /// negation logic directly: build a bold segment, apply a negating style that
    /// sets bold=false, and confirm the result has bold == Some(false).
    #[test]
    fn test_guide_removal_negation_applies_to_prefix() {
        // A bold guide segment — simulates what make_guide produces when the
        // guide_style carries "bold".
        let bold_style = Style::parse("bold");
        let guide_seg = Segment::styled("\u{2514}\u{2500}\u{2500} ", bold_style.clone());

        // Build the negating prefix_post style the same way the renderer does:
        // if node_guide_style.bold().is_some() → set bold = Some(false).
        let node_guide_style = bold_style.clone();
        let mut prefix_post = Style::null();
        if node_guide_style.bold().is_some() {
            prefix_post.set_bold(Some(false));
        }

        // Apply the negating style (post_style) to the guide segment.
        let negated =
            Segment::apply_style(std::slice::from_ref(&guide_seg), None, Some(prefix_post));

        assert_eq!(negated.len(), 1);
        let result_bold = negated[0].style.as_ref().and_then(|s| s.bold());
        assert_eq!(
            result_bold,
            Some(false),
            "After applying the negating prefix_post style, bold must be Some(false) \
             (negated off), not Some(true). Result segment: {:?}",
            negated[0]
        );
    }

    // -- Item 4: Default tree/tree.line theme styles ----------------------------

    /// When tree style is null and a "tree" theme style is installed,
    /// the rendered output should reflect that theme style.
    #[test]
    fn test_default_tree_theme_style_applied() {
        use crate::theme::Theme;
        use std::collections::HashMap;

        // Build a console with a custom theme that maps "tree" -> "bold".
        let mut console = Console::builder()
            .width(80)
            .markup(false)
            .highlight(false)
            .force_terminal(true)
            .build();

        // Install "tree" -> "bold" into the theme.
        let mut styles = HashMap::new();
        styles.insert("tree".to_string(), Style::parse("bold"));
        let theme = Theme::new(Some(styles), true);
        console.push_theme(theme, true);

        // Tree with null style — should pick up theme "tree" as base style.
        let tree = Tree::new(Text::new("root", Style::null()));
        let opts = console.options();
        let segments = tree.gilt_console(&console, &opts);

        // The label "root" should appear with bold style from the theme.
        let root_segs: Vec<&Segment> = segments
            .iter()
            .filter(|s| !s.is_control() && s.text.contains("root"))
            .collect();

        assert!(!root_segs.is_empty(), "Expected segments containing 'root'");

        let has_bold = root_segs.iter().any(|s| {
            s.style
                .as_ref()
                .map(|st| st.bold() == Some(true))
                .unwrap_or(false)
        });
        assert!(
            has_bold,
            "Root label segments should carry bold from the 'tree' theme style; \
             segments: {:?}",
            root_segs
        );
    }

    /// When tree guide_style is null and "tree.line" theme style is installed,
    /// guide segments should carry that theme style.
    #[test]
    fn test_default_tree_line_theme_style_applied() {
        use crate::theme::Theme;
        use std::collections::HashMap;

        let mut console = Console::builder()
            .width(80)
            .markup(false)
            .highlight(false)
            .force_terminal(true)
            .build();

        // Install "tree.line" -> "red" into the theme.
        let mut styles = HashMap::new();
        styles.insert("tree.line".to_string(), Style::parse("red"));
        let theme = Theme::new(Some(styles), true);
        console.push_theme(theme, true);

        // Tree with null guide_style — should pick up theme "tree.line".
        let mut tree = Tree::new(Text::new("root", Style::null()));
        tree.add(Text::new("child", Style::null()));
        let opts = console.options();
        let segments = tree.gilt_console(&console, &opts);

        // Guide segments contain guide chars (like └──).
        let guide_segs: Vec<&Segment> = segments
            .iter()
            .filter(|s| {
                !s.is_control()
                    && (s.text.contains('\u{2514}')
                        || s.text.contains('\u{251c}')
                        || s.text.contains('\u{2502}')
                        || s.text.contains("`-- ")
                        || s.text.contains("+-- "))
            })
            .collect();

        assert!(
            !guide_segs.is_empty(),
            "Expected at least one guide segment in output"
        );

        // At least one guide segment should carry a foreground color from the theme.
        let has_color = guide_segs.iter().any(|s| {
            s.style
                .as_ref()
                .map(|st| st.color().is_some())
                .unwrap_or(false)
        });
        assert!(
            has_color,
            "Guide segments should carry a color from the 'tree.line' theme style; \
             segments: {:?}",
            guide_segs
        );
    }
}
