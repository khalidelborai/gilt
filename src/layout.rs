//! Layout module — a recursive screen-splitting layout system.
//!
//! Port of Python's `rich/layout.py`. Provides [`Layout`] for dividing a
//! fixed-height terminal area into rows and columns, with flexible or
//! fixed sizing via [`ratio_resolve`].

use std::collections::HashMap;

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::ratio::{ratio_resolve, Edge};
use crate::region::Region;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;

// ---------------------------------------------------------------------------
// Splitter trait + implementations
// ---------------------------------------------------------------------------

/// Trait for objects that can divide a [`Region`] among child [`Layout`]s.
pub trait Splitter {
    /// Human-readable name of this splitter (e.g. "row" or "column").
    fn name(&self) -> &str;

    /// Divide `region` among `children`, returning `(child_index, child_region)` pairs.
    fn divide(&self, children: &[Layout], region: Region) -> Vec<(usize, Region)>;
}

/// Splits a region horizontally — children placed side by side.
///
/// Uses [`ratio_resolve`] to distribute **width** among children.
pub struct RowSplitter;

impl Splitter for RowSplitter {
    fn name(&self) -> &str {
        "row"
    }

    fn divide(&self, children: &[Layout], region: Region) -> Vec<(usize, Region)> {
        let widths = ratio_resolve(region.width, children);
        let mut offset: usize = 0;
        let mut result = Vec::with_capacity(children.len());
        for (i, &child_width) in widths.iter().enumerate() {
            result.push((
                i,
                Region::new(
                    region.x + offset as i32,
                    region.y,
                    child_width,
                    region.height,
                ),
            ));
            offset += child_width;
        }
        result
    }
}

/// Splits a region vertically — children stacked on top of each other.
///
/// Uses [`ratio_resolve`] to distribute **height** among children.
pub struct ColumnSplitter;

impl Splitter for ColumnSplitter {
    fn name(&self) -> &str {
        "column"
    }

    fn divide(&self, children: &[Layout], region: Region) -> Vec<(usize, Region)> {
        let heights = ratio_resolve(region.height, children);
        let mut offset: usize = 0;
        let mut result = Vec::with_capacity(children.len());
        for (i, &child_height) in heights.iter().enumerate() {
            result.push((
                i,
                Region::new(
                    region.x,
                    region.y + offset as i32,
                    region.width,
                    child_height,
                ),
            ));
            offset += child_height;
        }
        result
    }
}

// ---------------------------------------------------------------------------
// SplitterType enum
// ---------------------------------------------------------------------------

/// Enum selecting between row and column splitting strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitterType {
    /// Split horizontally -- children placed side by side.
    Row,
    /// Split vertically -- children stacked on top of each other.
    Column,
}

impl SplitterType {
    /// Create a boxed [`Splitter`] corresponding to this type.
    pub fn make_splitter(&self) -> Box<dyn Splitter> {
        match self {
            SplitterType::Row => Box::new(RowSplitter),
            SplitterType::Column => Box::new(ColumnSplitter),
        }
    }

    /// Human-readable name.
    pub fn name(&self) -> &str {
        match self {
            SplitterType::Row => "row",
            SplitterType::Column => "column",
        }
    }
}

// ---------------------------------------------------------------------------
// Layout
// ---------------------------------------------------------------------------

/// A renderable that divides a fixed-height area into rows or columns.
///
/// Layouts can be nested to create complex terminal UIs. Each layout can
/// either hold renderable content (a `String`) or be split into children.
#[derive(Debug, Clone)]
pub struct Layout {
    /// Renderable text content, or `None` for a placeholder.
    pub renderable: Option<String>,
    /// Optional identifier for this layout.
    pub name: Option<String>,
    /// Fixed size (width for row children, height for column children), or `None` for flexible.
    pub size: Option<usize>,
    /// Minimum size for flexible layouts.
    pub minimum_size: usize,
    /// Flex weight for proportional distribution.
    pub ratio: usize,
    /// Whether this layout is visible.
    pub visible: bool,
    /// The splitter type used to divide children.
    pub splitter: SplitterType,
    /// Child layouts.
    pub children: Vec<Layout>,
}

impl Edge for Layout {
    fn size(&self) -> Option<usize> {
        self.size
    }
    fn ratio(&self) -> usize {
        self.ratio
    }
    fn minimum_size(&self) -> usize {
        self.minimum_size
    }
}

impl Layout {
    /// Create a new layout.
    pub fn new(
        renderable: Option<String>,
        name: Option<String>,
        size: Option<usize>,
        minimum_size: Option<usize>,
        ratio: Option<usize>,
        visible: Option<bool>,
    ) -> Self {
        Layout {
            renderable,
            name,
            size,
            minimum_size: minimum_size.unwrap_or(1),
            ratio: ratio.unwrap_or(1),
            visible: visible.unwrap_or(true),
            splitter: SplitterType::Column,
            children: Vec::new(),
        }
    }

    /// Convenience: create a default layout with no content.
    pub fn default_layout() -> Self {
        Self::new(None, None, None, None, None, None)
    }

    /// Replace children with the given layouts and set the splitter type.
    pub fn split(&mut self, layouts: Vec<Layout>, splitter: SplitterType) {
        self.splitter = splitter;
        self.children = layouts;
    }

    /// Split horizontally (children side by side).
    pub fn split_row(&mut self, layouts: Vec<Layout>) {
        self.split(layouts, SplitterType::Row);
    }

    /// Split vertically (children stacked).
    pub fn split_column(&mut self, layouts: Vec<Layout>) {
        self.split(layouts, SplitterType::Column);
    }

    /// Add layouts to the existing children.
    pub fn add_split(&mut self, layouts: Vec<Layout>) {
        self.children.extend(layouts);
    }

    /// Remove all children (reset to unsplit state).
    pub fn unsplit(&mut self) {
        self.children.clear();
    }

    /// Update the renderable content.
    pub fn update(&mut self, renderable: String) {
        self.renderable = Some(renderable);
    }

    /// Recursively find a layout by name (immutable).
    pub fn get(&self, name: &str) -> Option<&Layout> {
        if self.name.as_deref() == Some(name) {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.get(name) {
                return Some(found);
            }
        }
        None
    }

    /// Recursively find a layout by name (mutable).
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Layout> {
        if self.name.as_deref() == Some(name) {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.get_mut(name) {
                return Some(found);
            }
        }
        None
    }

    /// Get visible children only.
    pub fn visible_children(&self) -> Vec<&Layout> {
        self.children.iter().filter(|c| c.visible).collect()
    }

    /// The effective renderable: if this layout has children, it acts as a
    /// container (returns `None`); otherwise returns the stored renderable.
    pub fn effective_renderable(&self) -> Option<&str> {
        if self.children.is_empty() {
            self.renderable.as_deref()
        } else {
            None
        }
    }

    /// Build a map from leaf layouts to their regions.
    ///
    /// Uses an iterative (stack-based) traversal. Each layout with visible
    /// children has its region subdivided by its splitter.
    pub fn make_region_map(&self, width: usize, height: usize) -> Vec<(&Layout, Region)> {
        let mut stack: Vec<(&Layout, Region)> = vec![(self, Region::new(0, 0, width, height))];
        let mut layout_regions: Vec<(&Layout, Region)> = Vec::new();

        while let Some((layout, region)) = stack.pop() {
            layout_regions.push((layout, region));
            let visible = layout.visible_children();
            if !visible.is_empty() {
                let splitter = layout.splitter.make_splitter();
                // Build a temporary vec of visible children for the splitter
                let visible_layouts: Vec<Layout> = visible.iter().map(|c| (*c).clone()).collect();
                let divisions = splitter.divide(&visible_layouts, region);
                // Map the child indices back to the actual child references
                for (child_idx, child_region) in divisions {
                    let child_ref = visible[child_idx];
                    stack.push((child_ref, child_region));
                }
            }
        }

        // Sort by region (y, x) for deterministic order
        layout_regions.sort_by(|a, b| (a.1.y, a.1.x).cmp(&(b.1.y, b.1.x)));

        layout_regions
    }

    /// Render all leaf layouts within the given dimensions.
    ///
    /// Returns a map from layout name to `(Region, rendered_lines)`.
    /// Unnamed layouts are keyed by a generated index-based name.
    pub fn render(
        &self,
        console: &Console,
        options: &ConsoleOptions,
    ) -> HashMap<String, (Region, Vec<Vec<Segment>>)> {
        let render_width = options.max_width;
        let render_height = options.height.unwrap_or(options.size.height);
        let region_map = self.make_region_map(render_width, render_height);

        let mut render_map: HashMap<String, (Region, Vec<Vec<Segment>>)> = HashMap::new();
        let mut unnamed_counter = 0usize;

        for (layout, region) in &region_map {
            // Only render leaf layouts (no visible children)
            if !layout.visible_children().is_empty() {
                continue;
            }

            let child_opts = options.update_dimensions(region.width, region.height);

            let lines = if let Some(content) = &layout.renderable {
                let text = Text::new(content, Style::null());
                console.render_lines(&text, Some(&child_opts), None, true, false)
            } else {
                // Placeholder: render a space-filled region
                let placeholder = placeholder_text(layout, region.width, region.height);
                console.render_lines(&placeholder, Some(&child_opts), None, true, false)
            };

            // Ensure lines match the expected height
            let lines = Segment::set_shape(&lines, region.width, Some(region.height), None, false);

            let key = match &layout.name {
                Some(n) => n.clone(),
                None => {
                    let k = format!("__unnamed_{}", unnamed_counter);
                    unnamed_counter += 1;
                    k
                }
            };

            render_map.insert(key, (*region, lines));
        }

        render_map
    }
}

impl Renderable for Layout {
    /// Compose the layout into a flat sequence of segments.
    ///
    /// Each row of the terminal is built by merging segments from all
    /// regions that occupy that row.
    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let width = options.max_width;
        let height = options.height.unwrap_or(options.size.height);
        let opts = options.update_dimensions(width, height);
        let render_map = self.render(console, &opts);

        // Collect all (region, lines) sorted by position
        let mut entries: Vec<(Region, &Vec<Vec<Segment>>)> =
            render_map.values().map(|(r, lines)| (*r, lines)).collect();
        entries.sort_by(|a, b| (a.0.y, a.0.x).cmp(&(b.0.y, b.0.x)));

        let mut layout_lines: Vec<Vec<Segment>> = vec![Vec::new(); height];

        for (region, lines) in &entries {
            let y = region.y as usize;
            let region_height = region.height;
            for (row_offset, line) in lines.iter().enumerate() {
                let target_row = y + row_offset;
                if target_row < height {
                    layout_lines[target_row].extend(line.iter().cloned());
                }
            }
            // If lines has fewer entries than region_height, pad remaining rows
            for row_offset in lines.len()..region_height {
                let target_row = y + row_offset;
                if target_row < height {
                    let padding = " ".repeat(region.width);
                    layout_lines[target_row].push(Segment::text(&padding));
                }
            }
        }

        let mut segments = Vec::new();
        for row in &layout_lines {
            segments.extend(row.iter().cloned());
            segments.push(Segment::line());
        }

        segments
    }
}

/// Create placeholder text for an unnamed/empty layout.
fn placeholder_text(layout: &Layout, width: usize, height: usize) -> Text {
    let title = match &layout.name {
        Some(n) => format!("'{}' ({} x {})", n, width, height),
        None => format!("({} x {})", width, height),
    };
    Text::new(&title, Style::null())
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl std::fmt::Display for Layout {
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

    // -- RowSplitter tests --------------------------------------------------

    #[test]
    fn test_row_splitter_name() {
        let splitter = RowSplitter;
        assert_eq!(splitter.name(), "row");
    }

    #[test]
    fn test_row_splitter_divide_equal() {
        let splitter = RowSplitter;
        let children = vec![
            Layout::new(None, None, None, None, Some(1), None),
            Layout::new(None, None, None, None, Some(1), None),
        ];
        let region = Region::new(0, 0, 80, 24);
        let result = splitter.divide(&children, region);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, 0);
        assert_eq!(result[1].0, 1);
        assert_eq!(result[0].1.width + result[1].1.width, 80);
        assert_eq!(result[0].1.height, 24);
        assert_eq!(result[1].1.height, 24);
        assert_eq!(result[0].1.x, 0);
        assert_eq!(result[0].1.y, 0);
    }

    #[test]
    fn test_row_splitter_divide_unequal() {
        let splitter = RowSplitter;
        let children = vec![
            Layout::new(None, None, None, None, Some(2), None),
            Layout::new(None, None, None, None, Some(1), None),
        ];
        let region = Region::new(0, 0, 90, 24);
        let result = splitter.divide(&children, region);
        // 2:1 ratio of 90 => 60, 30
        assert_eq!(result[0].1.width, 60);
        assert_eq!(result[1].1.width, 30);
    }

    #[test]
    fn test_row_splitter_divide_with_fixed_size() {
        let splitter = RowSplitter;
        let children = vec![
            Layout::new(None, None, Some(20), None, None, None),
            Layout::new(None, None, None, None, Some(1), None),
        ];
        let region = Region::new(0, 0, 80, 24);
        let result = splitter.divide(&children, region);
        assert_eq!(result[0].1.width, 20);
        assert_eq!(result[1].1.width, 60);
    }

    #[test]
    fn test_row_splitter_divide_offsets() {
        let splitter = RowSplitter;
        let children = vec![
            Layout::new(None, None, Some(30), None, None, None),
            Layout::new(None, None, Some(50), None, None, None),
        ];
        let region = Region::new(5, 10, 80, 24);
        let result = splitter.divide(&children, region);
        assert_eq!(result[0].1.x, 5);
        assert_eq!(result[0].1.y, 10);
        assert_eq!(result[1].1.x, 35); // 5 + 30
        assert_eq!(result[1].1.y, 10);
    }

    // -- ColumnSplitter tests -----------------------------------------------

    #[test]
    fn test_column_splitter_name() {
        let splitter = ColumnSplitter;
        assert_eq!(splitter.name(), "column");
    }

    #[test]
    fn test_column_splitter_divide_equal() {
        let splitter = ColumnSplitter;
        let children = vec![
            Layout::new(None, None, None, None, Some(1), None),
            Layout::new(None, None, None, None, Some(1), None),
        ];
        let region = Region::new(0, 0, 80, 24);
        let result = splitter.divide(&children, region);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].1.height + result[1].1.height, 24);
        assert_eq!(result[0].1.width, 80);
        assert_eq!(result[1].1.width, 80);
    }

    #[test]
    fn test_column_splitter_divide_unequal() {
        let splitter = ColumnSplitter;
        let children = vec![
            Layout::new(None, None, None, None, Some(3), None),
            Layout::new(None, None, None, None, Some(1), None),
        ];
        let region = Region::new(0, 0, 80, 40);
        let result = splitter.divide(&children, region);
        // 3:1 ratio of 40 => 30, 10
        assert_eq!(result[0].1.height, 30);
        assert_eq!(result[1].1.height, 10);
    }

    #[test]
    fn test_column_splitter_divide_with_fixed_size() {
        let splitter = ColumnSplitter;
        let children = vec![
            Layout::new(None, None, Some(5), None, None, None),
            Layout::new(None, None, None, None, Some(1), None),
        ];
        let region = Region::new(0, 0, 80, 24);
        let result = splitter.divide(&children, region);
        assert_eq!(result[0].1.height, 5);
        assert_eq!(result[1].1.height, 19);
    }

    #[test]
    fn test_column_splitter_divide_offsets() {
        let splitter = ColumnSplitter;
        let children = vec![
            Layout::new(None, None, Some(10), None, None, None),
            Layout::new(None, None, Some(14), None, None, None),
        ];
        let region = Region::new(5, 10, 80, 24);
        let result = splitter.divide(&children, region);
        assert_eq!(result[0].1.x, 5);
        assert_eq!(result[0].1.y, 10);
        assert_eq!(result[1].1.x, 5);
        assert_eq!(result[1].1.y, 20); // 10 + 10
    }

    // -- SplitterType tests -------------------------------------------------

    #[test]
    fn test_splitter_type_name() {
        assert_eq!(SplitterType::Row.name(), "row");
        assert_eq!(SplitterType::Column.name(), "column");
    }

    #[test]
    fn test_splitter_type_make_splitter() {
        let row = SplitterType::Row.make_splitter();
        assert_eq!(row.name(), "row");
        let col = SplitterType::Column.make_splitter();
        assert_eq!(col.name(), "column");
    }

    // -- Layout::new defaults -----------------------------------------------

    #[test]
    fn test_layout_new_defaults() {
        let layout = Layout::new(None, None, None, None, None, None);
        assert!(layout.renderable.is_none());
        assert!(layout.name.is_none());
        assert!(layout.size.is_none());
        assert_eq!(layout.minimum_size, 1);
        assert_eq!(layout.ratio, 1);
        assert!(layout.visible);
        assert_eq!(layout.splitter, SplitterType::Column);
        assert!(layout.children.is_empty());
    }

    #[test]
    fn test_layout_new_with_values() {
        let layout = Layout::new(
            Some("hello".to_string()),
            Some("main".to_string()),
            Some(20),
            Some(5),
            Some(3),
            Some(false),
        );
        assert_eq!(layout.renderable.as_deref(), Some("hello"));
        assert_eq!(layout.name.as_deref(), Some("main"));
        assert_eq!(layout.size, Some(20));
        assert_eq!(layout.minimum_size, 5);
        assert_eq!(layout.ratio, 3);
        assert!(!layout.visible);
    }

    #[test]
    fn test_layout_default_layout() {
        let layout = Layout::default_layout();
        assert!(layout.renderable.is_none());
        assert_eq!(layout.ratio, 1);
        assert_eq!(layout.minimum_size, 1);
    }

    // -- Edge trait implementation -------------------------------------------

    #[test]
    fn test_edge_impl_flexible() {
        let layout = Layout::new(None, None, None, None, Some(2), None);
        assert_eq!(Edge::size(&layout), None);
        assert_eq!(Edge::ratio(&layout), 2);
        assert_eq!(Edge::minimum_size(&layout), 1);
    }

    #[test]
    fn test_edge_impl_fixed() {
        let layout = Layout::new(None, None, Some(10), Some(3), Some(1), None);
        assert_eq!(Edge::size(&layout), Some(10));
        assert_eq!(Edge::ratio(&layout), 1);
        assert_eq!(Edge::minimum_size(&layout), 3);
    }

    // -- split / split_row / split_column -----------------------------------

    #[test]
    fn test_split() {
        let mut layout = Layout::default_layout();
        let child1 = Layout::new(None, Some("a".to_string()), None, None, None, None);
        let child2 = Layout::new(None, Some("b".to_string()), None, None, None, None);
        layout.split(vec![child1, child2], SplitterType::Row);
        assert_eq!(layout.children.len(), 2);
        assert_eq!(layout.splitter, SplitterType::Row);
    }

    #[test]
    fn test_split_row() {
        let mut layout = Layout::default_layout();
        let child1 = Layout::default_layout();
        let child2 = Layout::default_layout();
        layout.split_row(vec![child1, child2]);
        assert_eq!(layout.splitter, SplitterType::Row);
        assert_eq!(layout.children.len(), 2);
    }

    #[test]
    fn test_split_column() {
        let mut layout = Layout::default_layout();
        let child1 = Layout::default_layout();
        let child2 = Layout::default_layout();
        layout.split_column(vec![child1, child2]);
        assert_eq!(layout.splitter, SplitterType::Column);
        assert_eq!(layout.children.len(), 2);
    }

    #[test]
    fn test_split_replaces_children() {
        let mut layout = Layout::default_layout();
        let child1 = Layout::default_layout();
        layout.split_column(vec![child1]);
        assert_eq!(layout.children.len(), 1);

        let child2 = Layout::default_layout();
        let child3 = Layout::default_layout();
        layout.split_row(vec![child2, child3]);
        assert_eq!(layout.children.len(), 2);
        assert_eq!(layout.splitter, SplitterType::Row);
    }

    // -- add_split ----------------------------------------------------------

    #[test]
    fn test_add_split() {
        let mut layout = Layout::default_layout();
        let child1 = Layout::default_layout();
        layout.split_column(vec![child1]);
        assert_eq!(layout.children.len(), 1);

        let child2 = Layout::default_layout();
        layout.add_split(vec![child2]);
        assert_eq!(layout.children.len(), 2);
    }

    // -- unsplit -------------------------------------------------------------

    #[test]
    fn test_unsplit() {
        let mut layout = Layout::default_layout();
        let child1 = Layout::default_layout();
        let child2 = Layout::default_layout();
        layout.split_column(vec![child1, child2]);
        assert_eq!(layout.children.len(), 2);

        layout.unsplit();
        assert!(layout.children.is_empty());
    }

    // -- update --------------------------------------------------------------

    #[test]
    fn test_update() {
        let mut layout = Layout::default_layout();
        assert!(layout.renderable.is_none());
        layout.update("new content".to_string());
        assert_eq!(layout.renderable.as_deref(), Some("new content"));
    }

    // -- get / get_mut -------------------------------------------------------

    #[test]
    fn test_get_self() {
        let layout = Layout::new(None, Some("root".to_string()), None, None, None, None);
        assert!(layout.get("root").is_some());
        assert_eq!(layout.get("root").unwrap().name.as_deref(), Some("root"));
    }

    #[test]
    fn test_get_child() {
        let mut layout = Layout::new(None, Some("root".to_string()), None, None, None, None);
        let child = Layout::new(None, Some("child".to_string()), None, None, None, None);
        layout.split_column(vec![child]);
        assert!(layout.get("child").is_some());
        assert_eq!(layout.get("child").unwrap().name.as_deref(), Some("child"));
    }

    #[test]
    fn test_get_nested() {
        let mut layout = Layout::new(None, Some("root".to_string()), None, None, None, None);
        let mut child = Layout::new(None, Some("child".to_string()), None, None, None, None);
        let grandchild = Layout::new(None, Some("grandchild".to_string()), None, None, None, None);
        child.split_column(vec![grandchild]);
        layout.split_column(vec![child]);

        assert!(layout.get("grandchild").is_some());
        assert_eq!(
            layout.get("grandchild").unwrap().name.as_deref(),
            Some("grandchild")
        );
    }

    #[test]
    fn test_get_not_found() {
        let layout = Layout::new(None, Some("root".to_string()), None, None, None, None);
        assert!(layout.get("nonexistent").is_none());
    }

    #[test]
    fn test_get_mut_update() {
        let mut layout = Layout::new(None, Some("root".to_string()), None, None, None, None);
        let child = Layout::new(None, Some("child".to_string()), None, None, None, None);
        layout.split_column(vec![child]);

        let found = layout.get_mut("child").unwrap();
        found.update("updated".to_string());

        assert_eq!(
            layout.get("child").unwrap().renderable.as_deref(),
            Some("updated")
        );
    }

    // -- visible_children ---------------------------------------------------

    #[test]
    fn test_visible_children_all_visible() {
        let mut layout = Layout::default_layout();
        let child1 = Layout::new(None, Some("a".to_string()), None, None, None, Some(true));
        let child2 = Layout::new(None, Some("b".to_string()), None, None, None, Some(true));
        layout.split_column(vec![child1, child2]);
        assert_eq!(layout.visible_children().len(), 2);
    }

    #[test]
    fn test_visible_children_some_hidden() {
        let mut layout = Layout::default_layout();
        let child1 = Layout::new(None, Some("a".to_string()), None, None, None, Some(true));
        let child2 = Layout::new(None, Some("b".to_string()), None, None, None, Some(false));
        let child3 = Layout::new(None, Some("c".to_string()), None, None, None, Some(true));
        layout.split_column(vec![child1, child2, child3]);
        let visible = layout.visible_children();
        assert_eq!(visible.len(), 2);
        assert_eq!(visible[0].name.as_deref(), Some("a"));
        assert_eq!(visible[1].name.as_deref(), Some("c"));
    }

    #[test]
    fn test_visible_children_all_hidden() {
        let mut layout = Layout::default_layout();
        let child1 = Layout::new(None, None, None, None, None, Some(false));
        layout.split_column(vec![child1]);
        assert!(layout.visible_children().is_empty());
    }

    // -- effective_renderable -----------------------------------------------

    #[test]
    fn test_effective_renderable_leaf() {
        let layout = Layout::new(Some("content".to_string()), None, None, None, None, None);
        assert_eq!(layout.effective_renderable(), Some("content"));
    }

    #[test]
    fn test_effective_renderable_with_children() {
        let mut layout = Layout::new(Some("content".to_string()), None, None, None, None, None);
        let child = Layout::default_layout();
        layout.split_column(vec![child]);
        // Has children, so effective renderable is None
        assert!(layout.effective_renderable().is_none());
    }

    #[test]
    fn test_effective_renderable_none() {
        let layout = Layout::default_layout();
        assert!(layout.effective_renderable().is_none());
    }

    // -- make_region_map ----------------------------------------------------

    #[test]
    fn test_make_region_map_single() {
        let layout = Layout::default_layout();
        let map = layout.make_region_map(80, 24);
        assert_eq!(map.len(), 1);
        assert_eq!(map[0].1, Region::new(0, 0, 80, 24));
    }

    #[test]
    fn test_make_region_map_two_columns() {
        let mut layout = Layout::default_layout();
        let child1 = Layout::new(None, Some("top".to_string()), None, None, Some(1), None);
        let child2 = Layout::new(None, Some("bottom".to_string()), None, None, Some(1), None);
        layout.split_column(vec![child1, child2]);

        let map = layout.make_region_map(80, 24);
        // Should have root + 2 children = 3
        assert_eq!(map.len(), 3);

        // Find the children's regions
        let top = map.iter().find(|(l, _)| l.name.as_deref() == Some("top"));
        let bottom = map
            .iter()
            .find(|(l, _)| l.name.as_deref() == Some("bottom"));
        assert!(top.is_some());
        assert!(bottom.is_some());

        let top_region = top.unwrap().1;
        let bottom_region = bottom.unwrap().1;
        assert_eq!(top_region.x, 0);
        assert_eq!(top_region.y, 0);
        assert_eq!(top_region.width, 80);
        assert_eq!(bottom_region.x, 0);
        assert_eq!(bottom_region.width, 80);
        assert_eq!(top_region.height + bottom_region.height, 24);
    }

    #[test]
    fn test_make_region_map_two_rows() {
        let mut layout = Layout::default_layout();
        let child1 = Layout::new(None, Some("left".to_string()), None, None, Some(1), None);
        let child2 = Layout::new(None, Some("right".to_string()), None, None, Some(1), None);
        layout.split_row(vec![child1, child2]);

        let map = layout.make_region_map(80, 24);
        let left = map.iter().find(|(l, _)| l.name.as_deref() == Some("left"));
        let right = map.iter().find(|(l, _)| l.name.as_deref() == Some("right"));
        assert!(left.is_some());
        assert!(right.is_some());

        let left_region = left.unwrap().1;
        let right_region = right.unwrap().1;
        assert_eq!(left_region.y, 0);
        assert_eq!(right_region.y, 0);
        assert_eq!(left_region.height, 24);
        assert_eq!(right_region.height, 24);
        assert_eq!(left_region.width + right_region.width, 80);
    }

    #[test]
    fn test_make_region_map_nested() {
        let mut layout = Layout::default_layout();
        let mut top = Layout::new(None, Some("top".to_string()), Some(10), None, None, None);
        let top_left = Layout::new(
            None,
            Some("top_left".to_string()),
            None,
            None,
            Some(1),
            None,
        );
        let top_right = Layout::new(
            None,
            Some("top_right".to_string()),
            None,
            None,
            Some(1),
            None,
        );
        top.split_row(vec![top_left, top_right]);

        let bottom = Layout::new(None, Some("bottom".to_string()), None, None, Some(1), None);
        layout.split_column(vec![top, bottom]);

        let map = layout.make_region_map(80, 24);
        let tl = map
            .iter()
            .find(|(l, _)| l.name.as_deref() == Some("top_left"));
        let tr = map
            .iter()
            .find(|(l, _)| l.name.as_deref() == Some("top_right"));
        let bot = map
            .iter()
            .find(|(l, _)| l.name.as_deref() == Some("bottom"));

        assert!(tl.is_some());
        assert!(tr.is_some());
        assert!(bot.is_some());

        let tl_r = tl.unwrap().1;
        let tr_r = tr.unwrap().1;
        let bot_r = bot.unwrap().1;

        // Top row is 10 high
        assert_eq!(tl_r.height, 10);
        assert_eq!(tr_r.height, 10);
        // Bottom gets the rest
        assert_eq!(bot_r.height, 14);
        // Top left and right share the width
        assert_eq!(tl_r.width + tr_r.width, 80);
    }

    #[test]
    fn test_make_region_map_hidden_child() {
        let mut layout = Layout::default_layout();
        let child1 = Layout::new(None, Some("visible".to_string()), None, None, Some(1), None);
        let child2 = Layout::new(
            None,
            Some("hidden".to_string()),
            None,
            None,
            Some(1),
            Some(false),
        );
        layout.split_column(vec![child1, child2]);

        let map = layout.make_region_map(80, 24);
        // Hidden child should not be in the division
        let visible = map
            .iter()
            .find(|(l, _)| l.name.as_deref() == Some("visible"));
        assert!(visible.is_some());
        // The visible child gets the full region
        assert_eq!(visible.unwrap().1, Region::new(0, 0, 80, 24));

        // Hidden child is still in children but not in region map traversal
        let hidden = map
            .iter()
            .find(|(l, _)| l.name.as_deref() == Some("hidden"));
        assert!(hidden.is_none());
    }

    // -- Render tests -------------------------------------------------------

    #[test]
    fn test_render_single_layout() {
        let console = Console::builder().width(40).height(5).build();
        let options = console.options();

        let mut layout = Layout::default_layout();
        layout.update("Hello".to_string());
        layout.name = Some("main".to_string());

        let render_map = layout.render(&console, &options);
        assert!(render_map.contains_key("main"));
        let (region, lines) = &render_map["main"];
        assert_eq!(region.width, 40);
        assert_eq!(lines.len(), 5);
    }

    #[test]
    fn test_render_two_row_split() {
        let console = Console::builder().width(40).height(10).build();
        let options = console.options();

        let mut layout = Layout::default_layout();
        let mut left = Layout::new(None, Some("left".to_string()), None, None, Some(1), None);
        left.update("LEFT".to_string());
        let mut right = Layout::new(None, Some("right".to_string()), None, None, Some(1), None);
        right.update("RIGHT".to_string());
        layout.split_row(vec![left, right]);

        let render_map = layout.render(&console, &options);
        assert!(render_map.contains_key("left"));
        assert!(render_map.contains_key("right"));
        let (lr, _) = &render_map["left"];
        let (rr, _) = &render_map["right"];
        assert_eq!(lr.width + rr.width, 40);
    }

    #[test]
    fn test_render_two_column_split() {
        let console = Console::builder().width(40).height(10).build();
        let options = console.options();

        let mut layout = Layout::default_layout();
        let mut top = Layout::new(None, Some("top".to_string()), None, None, Some(1), None);
        top.update("TOP".to_string());
        let mut bottom = Layout::new(None, Some("bottom".to_string()), None, None, Some(1), None);
        bottom.update("BOTTOM".to_string());
        layout.split_column(vec![top, bottom]);

        let render_map = layout.render(&console, &options);
        assert!(render_map.contains_key("top"));
        assert!(render_map.contains_key("bottom"));
        let (tr, _) = &render_map["top"];
        let (br, _) = &render_map["bottom"];
        assert_eq!(tr.height + br.height, 10);
    }

    #[test]
    fn test_render_nested_grid() {
        let console = Console::builder().width(80).height(24).build();
        let options = console.options();

        let mut layout = Layout::default_layout();

        let mut top_row = Layout::new(None, None, Some(12), None, None, None);
        let tl = Layout::new(
            Some("TL".to_string()),
            Some("tl".to_string()),
            None,
            None,
            Some(1),
            None,
        );
        let tr = Layout::new(
            Some("TR".to_string()),
            Some("tr".to_string()),
            None,
            None,
            Some(1),
            None,
        );
        top_row.split_row(vec![tl, tr]);

        let mut bot_row = Layout::new(None, None, None, None, Some(1), None);
        let bl = Layout::new(
            Some("BL".to_string()),
            Some("bl".to_string()),
            None,
            None,
            Some(1),
            None,
        );
        let br = Layout::new(
            Some("BR".to_string()),
            Some("br".to_string()),
            None,
            None,
            Some(1),
            None,
        );
        bot_row.split_row(vec![bl, br]);

        layout.split_column(vec![top_row, bot_row]);

        let render_map = layout.render(&console, &options);
        assert!(render_map.contains_key("tl"));
        assert!(render_map.contains_key("tr"));
        assert!(render_map.contains_key("bl"));
        assert!(render_map.contains_key("br"));

        let (tl_r, _) = &render_map["tl"];
        let (tr_r, _) = &render_map["tr"];
        let (bl_r, _) = &render_map["bl"];
        let (br_r, _) = &render_map["br"];

        assert_eq!(tl_r.height, 12);
        assert_eq!(tr_r.height, 12);
        assert_eq!(bl_r.height, 12);
        assert_eq!(br_r.height, 12);
        assert_eq!(tl_r.width + tr_r.width, 80);
        assert_eq!(bl_r.width + br_r.width, 80);
    }

    // -- Renderable implementation ------------------------------------------

    #[test]
    fn test_renderable_single_layout() {
        let console = Console::builder().width(20).height(3).markup(false).build();
        let options = console.options().update_dimensions(20, 3);

        let mut layout = Layout::default_layout();
        layout.update("Test".to_string());

        let segments = layout.gilt_console(&console, &options);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("Test"));
    }

    #[test]
    fn test_renderable_produces_correct_height() {
        let console = Console::builder().width(20).height(5).markup(false).build();
        let options = console.options().update_dimensions(20, 5);

        let mut layout = Layout::default_layout();
        layout.update("X".to_string());

        let segments = layout.gilt_console(&console, &options);
        // Count newlines — should be exactly height (one per row)
        let newlines = segments.iter().filter(|s| s.text == "\n").count();
        assert_eq!(newlines, 5);
    }

    // -- Placeholder text ---------------------------------------------------

    #[test]
    fn test_placeholder_text_with_name() {
        let layout = Layout::new(None, Some("sidebar".to_string()), None, None, None, None);
        let text = placeholder_text(&layout, 40, 10);
        let plain = text.plain().to_string();
        assert!(plain.contains("sidebar"));
        assert!(plain.contains("40"));
        assert!(plain.contains("10"));
    }

    #[test]
    fn test_placeholder_text_without_name() {
        let layout = Layout::default_layout();
        let text = placeholder_text(&layout, 80, 24);
        let plain = text.plain().to_string();
        assert!(plain.contains("80"));
        assert!(plain.contains("24"));
        assert!(!plain.contains("'"));
    }

    // -- Clone test ---------------------------------------------------------

    #[test]
    fn test_layout_clone() {
        let mut layout = Layout::new(
            Some("content".to_string()),
            Some("main".to_string()),
            Some(20),
            Some(5),
            Some(2),
            Some(true),
        );
        let child = Layout::default_layout();
        layout.split_column(vec![child]);

        let cloned = layout.clone();
        assert_eq!(cloned.name, layout.name);
        assert_eq!(cloned.renderable, layout.renderable);
        assert_eq!(cloned.size, layout.size);
        assert_eq!(cloned.minimum_size, layout.minimum_size);
        assert_eq!(cloned.ratio, layout.ratio);
        assert_eq!(cloned.visible, layout.visible);
        assert_eq!(cloned.splitter, layout.splitter);
        assert_eq!(cloned.children.len(), layout.children.len());
    }

    // -- Three-way row split ------------------------------------------------

    #[test]
    fn test_row_splitter_three_children() {
        let splitter = RowSplitter;
        let children = vec![
            Layout::new(None, None, None, None, Some(1), None),
            Layout::new(None, None, None, None, Some(1), None),
            Layout::new(None, None, None, None, Some(1), None),
        ];
        let region = Region::new(0, 0, 90, 24);
        let result = splitter.divide(&children, region);
        assert_eq!(result.len(), 3);
        let total_width: usize = result.iter().map(|(_, r)| r.width).sum();
        assert!(total_width <= 90);
        // Each should get 30
        assert_eq!(result[0].1.width, 30);
        assert_eq!(result[1].1.width, 30);
        assert_eq!(result[2].1.width, 30);
    }

    // -- Column splitter three children -------------------------------------

    #[test]
    fn test_column_splitter_three_children() {
        let splitter = ColumnSplitter;
        let children = vec![
            Layout::new(None, None, None, None, Some(1), None),
            Layout::new(None, None, None, None, Some(1), None),
            Layout::new(None, None, None, None, Some(1), None),
        ];
        let region = Region::new(0, 0, 80, 30);
        let result = splitter.divide(&children, region);
        assert_eq!(result.len(), 3);
        let total_height: usize = result.iter().map(|(_, r)| r.height).sum();
        assert_eq!(total_height, 30);
    }

    // -- Layout with size, ratio, minimum_size, visible=false ---------------

    #[test]
    fn test_layout_size_ratio_minimum_size() {
        let layout = Layout::new(None, None, Some(15), Some(3), Some(4), None);
        assert_eq!(layout.size, Some(15));
        assert_eq!(layout.minimum_size, 3);
        assert_eq!(layout.ratio, 4);
        assert!(layout.visible);
    }

    #[test]
    fn test_layout_visible_false() {
        let layout = Layout::new(None, None, None, None, None, Some(false));
        assert!(!layout.visible);
    }

    #[test]
    fn test_display_trait() {
        let layout = Layout::new(
            Some("root".to_string()),
            Some("root".to_string()),
            None,
            None,
            None,
            None,
        );
        let s = format!("{}", layout);
        assert!(!s.is_empty());
    }
}
