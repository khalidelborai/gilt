//! Tree widget for rendering hierarchical structures with guide characters.
//!
//! Port of Python's `rich/tree.py`.

use crate::cells::cell_len;
use crate::console::{Console, ConsoleOptions, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;
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

fn make_guide(index: usize, style: &Style, ascii_only: bool) -> Segment {
    if ascii_only {
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
pub struct Tree {
    /// The node's display text.
    pub label: Text,
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
}

impl Tree {
    /// Create a new tree node with the given label.
    pub fn new(label: Text) -> Self {
        Tree {
            label,
            style: Style::null(),
            guide_style: Style::null(),
            children: Vec::new(),
            expanded: true,
            hide_root: false,
        }
    }

    /// Add a child node and return a mutable reference to it.
    pub fn add(&mut self, label: Text) -> &mut Tree {
        self.children.push(Tree {
            label,
            style: self.style.clone(),
            guide_style: self.guide_style.clone(),
            children: Vec::new(),
            expanded: true,
            hide_root: false,
        });
        self.children.last_mut().unwrap()
    }

    /// Set the node style (builder pattern).
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the guide style (builder pattern).
    #[must_use]
    pub fn guide_style(mut self, style: Style) -> Self {
        self.guide_style = style;
        self
    }

    /// Set whether the tree is expanded (builder pattern).
    #[must_use]
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Set whether the root node is hidden (builder pattern).
    #[must_use]
    pub fn hide_root(mut self, hide_root: bool) -> Self {
        self.hide_root = hide_root;
        self
    }

    /// Measure this tree: compute minimum and maximum widths.
    pub fn measure(&self, _console: &Console, _options: &ConsoleOptions) -> Measurement {
        let mut minimum: usize = 0;
        let mut maximum: usize = 0;

        fn measure_recursive(
            tree: &Tree,
            level: usize,
            min: &mut usize,
            max: &mut usize,
            hide_root: bool,
        ) {
            let effective_level = if hide_root {
                level.saturating_sub(1)
            } else {
                level
            };
            let indent = effective_level * 4;
            let label_width = tree.label.cell_len();
            let total = label_width + indent;
            if !(level == 0 && hide_root) {
                *min = (*min).max(total);
                *max = (*max).max(total);
            }
            if tree.expanded {
                for child in &tree.children {
                    measure_recursive(child, level + 1, min, max, hide_root);
                }
            }
        }

        measure_recursive(self, 0, &mut minimum, &mut maximum, self.hide_root);
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
    fn rich_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let mut segments: Vec<Segment> = Vec::new();
        let ascii_only = options.ascii_only();
        let newline = Segment::line();

        // Stack-based DFS (porting Python's stack/iterator approach).
        //
        // `levels` holds the guide segment for each depth level.
        // The stack holds iterators over children at each level.
        let mut levels: Vec<Segment> = vec![make_guide(CONTINUE, &self.guide_style, ascii_only)];
        let mut stack: Vec<StackFrame> = Vec::new();

        // Push the root as a single-element "children" iterator.
        let root_slice = std::slice::from_ref(self);
        stack.push(StackFrame {
            index: 0,
            children: root_slice,
        });

        let mut depth: usize = 0;

        while let Some(frame) = stack.last_mut() {
            if frame.index >= frame.children.len() {
                // This level is exhausted.
                stack.pop();
                levels.pop();
                if !levels.is_empty() {
                    let last_idx = levels.len() - 1;
                    let guide_style = levels[last_idx].style.clone().unwrap_or_else(Style::null);
                    levels[last_idx] = make_guide(FORK, &guide_style, ascii_only);
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
                levels[last_level] = make_guide(END, &guide_style, ascii_only);
            }

            // Build the prefix from levels, skipping levels for hidden root.
            let skip = if self.hide_root { 2 } else { 1 };
            let prefix: Vec<Segment> = if levels.len() > skip {
                levels[skip..].to_vec()
            } else {
                Vec::new()
            };

            // Compute available width for the label.
            let prefix_width: usize = prefix.iter().map(|s| cell_len(&s.text)).sum();
            let child_width = options.max_width.saturating_sub(prefix_width);
            let child_opts = options.update_width(child_width);

            // Render the label into lines.
            let rendered_lines =
                console.render_lines(&node.label, Some(&child_opts), None, false, false);

            // Emit segments (skip if this is the root and hide_root is set).
            let skip_node = depth == 0 && self.hide_root;

            if !skip_node {
                let mut current_prefix = prefix.clone();
                for (i, line) in rendered_lines.iter().enumerate() {
                    // Emit prefix guide segments.
                    for seg in &current_prefix {
                        segments.push(seg.clone());
                    }
                    // Emit line content segments.
                    segments.extend(line.iter().cloned());
                    // Emit newline.
                    segments.push(newline.clone());

                    // After the first line, change the last prefix element
                    // from FORK/END to CONTINUE/SPACE for continuation lines.
                    if i == 0 && !current_prefix.is_empty() {
                        let last_idx = current_prefix.len() - 1;
                        let pstyle = current_prefix[last_idx]
                            .style
                            .clone()
                            .unwrap_or_else(Style::null);
                        current_prefix[last_idx] =
                            make_guide(if last { SPACE } else { CONTINUE }, &pstyle, ascii_only);
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
                );

                // Add a new level for the children.
                let child_guide_style = &node.guide_style;
                let child_count = node.children.len();
                let guide_type = if child_count == 1 { END } else { FORK };
                levels.push(make_guide(guide_type, child_guide_style, ascii_only));

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
        let segments = tree.rich_console(&console, &opts);
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
        let mut tree = Tree::new(Text::new("root", Style::null())).hide_root(true);
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
            .push(Tree::new(Text::new("branch", Style::null())).expanded(false));
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
        opts.encoding = "ascii".to_string();
        let segments = tree.rich_console(&console, &opts);
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
        let style = Style::parse("bold").unwrap();
        let guide_style = Style::parse("red").unwrap();

        let tree = Tree::new(Text::new("root", Style::null()))
            .style(style.clone())
            .guide_style(guide_style.clone())
            .expanded(false)
            .hide_root(true);

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
        assert_eq!(tree.children[0].children[0].label.plain(), "grandchild");
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
        let style = Style::parse("bold").unwrap();
        let guide_style = Style::parse("red").unwrap();

        let mut tree = Tree::new(Text::new("root", Style::null()));
        tree.style = style.clone();
        tree.guide_style = guide_style.clone();

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
        let mut tree = Tree::new(Text::new("ROOT", Style::null())).hide_root(true);
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
        let mut tree = Tree::new(Text::new("LONG_ROOT_NAME", Style::null())).hide_root(true);
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
        let mut branch = Tree::new(Text::new("branch", Style::null())).expanded(false);
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
        let mut tree = Tree::new(Text::new("root", Style::null()));
        tree.guide_style = Style::parse("bold").unwrap();
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
        let mut tree = Tree::new(Text::new("root", Style::null()));
        tree.guide_style = Style::parse("underline2").unwrap();
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
        let segments = tree.rich_console(&console, &opts);

        let newline_count = segments.iter().filter(|s| s.text == "\n").count();
        assert_eq!(newline_count, 2, "Expected 2 newlines (one per line)");
    }

    // -- 25. hide_root with no children produces no output --

    #[test]
    fn test_hide_root_no_children() {
        let tree = Tree::new(Text::new("hidden", Style::null())).hide_root(true);
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
}
