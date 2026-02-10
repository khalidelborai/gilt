//! Inspect any Rust value in a beautifully formatted panel.
//!
//! This is the Rust equivalent of Python's `rich.inspect()`. Since Rust
//! doesn't have runtime reflection, it uses the `Debug` trait to display
//! structured information about values.
//!
//! # Example
//!
//! ```rust
//! use gilt::inspect::Inspect;
//! use gilt::console::Console;
//!
//! let data = vec![1, 2, 3];
//! let inspect = Inspect::new(&data);
//!
//! let mut console = Console::builder().width(80).force_terminal(true).build();
//! console.begin_capture();
//! console.print(&inspect);
//! let output = console.end_capture();
//! assert!(output.contains("Vec"));
//! ```

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::highlighter::{Highlighter, ReprHighlighter};
use crate::panel::Panel;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;
use std::fmt;

// ---------------------------------------------------------------------------
// Inspect
// ---------------------------------------------------------------------------

/// Inspect widget that displays structured information about a value.
///
/// Renders a panel showing:
/// - The type name
/// - The Debug representation (syntax highlighted)
/// - Optional documentation string
/// - Optional value label
pub struct Inspect<'a> {
    /// The value to inspect (as a Debug reference).
    value: &'a dyn fmt::Debug,
    /// The type name string.
    type_name: &'static str,
    /// Optional label for the value.
    label: Option<String>,
    /// Optional documentation.
    doc: Option<String>,
    /// Whether to pretty-print the Debug output.
    pretty: bool,
    /// Title for the panel.
    title: Option<String>,
}

impl<'a> Inspect<'a> {
    /// Create a new Inspect widget for any Debug value.
    ///
    /// Uses `std::any::type_name` to determine the type name.
    pub fn new<T: fmt::Debug + 'static>(value: &'a T) -> Self {
        Self {
            value,
            type_name: std::any::type_name::<T>(),
            label: None,
            doc: None,
            pretty: true,
            title: None,
        }
    }

    /// Set a label for the inspected value.
    #[must_use]
    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Set documentation text to display.
    #[must_use]
    pub fn with_doc(mut self, doc: &str) -> Self {
        self.doc = Some(doc.to_string());
        self
    }

    /// Set whether to pretty-print the Debug output.
    #[must_use]
    pub fn with_pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }

    /// Set a custom title for the panel.
    #[must_use]
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Extract the short type name (last path component).
    fn short_type_name(&self) -> &str {
        let full = self.type_name;
        // Handle generic types like "alloc::vec::Vec<i32>" by finding the
        // last "::" before any '<'.
        let prefix = full.split('<').next().unwrap_or(full);
        prefix.rsplit("::").next().unwrap_or(full)
    }

    /// Build the text content for the panel.
    fn build_content(&self) -> Text {
        let mut parts = Vec::new();

        // Type name header
        parts.push(format!(
            "[bold cyan]Type:[/bold cyan] [italic]{}[/italic]",
            self.type_name
        ));

        // Label
        if let Some(label) = &self.label {
            parts.push(format!("[bold cyan]Name:[/bold cyan] {}", label));
        }

        // Documentation
        if let Some(doc) = &self.doc {
            parts.push(format!(
                "[bold cyan]Doc:[/bold cyan] [dim italic]{}[/dim italic]",
                doc
            ));
        }

        // Separator
        parts.push(String::new());

        // Value header
        parts.push("[bold cyan]Value:[/bold cyan]".to_string());

        let markup_part = parts.join("\n");
        // from_markup returns Result; fall back to plain text on error.
        let mut text = Text::from_markup(&markup_part)
            .unwrap_or_else(|_| Text::new(&markup_part, Style::null()));

        // Debug representation
        let debug_str = if self.pretty {
            format!("{:#?}", self.value)
        } else {
            format!("{:?}", self.value)
        };

        // Add debug output with highlighting via ReprHighlighter
        let mut debug_text = Text::new(&format!("\n{}", debug_str), Style::null());
        let highlighter = ReprHighlighter::new();
        highlighter.highlight(&mut debug_text);
        text.append_text(&debug_text);

        text
    }
}

// ---------------------------------------------------------------------------
// Renderable
// ---------------------------------------------------------------------------

impl Renderable for Inspect<'_> {
    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let content = self.build_content();
        let mut panel = Panel::new(content);

        let title_str = self
            .title
            .clone()
            .unwrap_or_else(|| format!("Inspect: {}", self.short_type_name()));
        panel.title = Some(Text::new(&title_str, Style::null()));

        panel.gilt_console(console, options)
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl fmt::Display for Inspect<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let w = f.width().unwrap_or(80);
        let mut console = Console::builder()
            .width(w)
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

    /// Helper: capture inspect output as a string.
    fn capture_inspect(inspect: &Inspect) -> String {
        let mut console = Console::builder()
            .width(80)
            .force_terminal(true)
            .no_color(true)
            .build();
        console.begin_capture();
        console.print(inspect);
        console.end_capture()
    }

    // -- 1. Inspect a Vec<i32> -----------------------------------------------

    #[test]
    fn test_inspect_vec_i32() {
        let data = vec![1, 2, 3];
        let inspect = Inspect::new(&data);
        let output = capture_inspect(&inspect);
        assert!(
            output.contains("Vec"),
            "output should contain 'Vec': {}",
            output
        );
        assert!(
            output.contains("1"),
            "output should contain '1': {}",
            output
        );
        assert!(
            output.contains("2"),
            "output should contain '2': {}",
            output
        );
        assert!(
            output.contains("3"),
            "output should contain '3': {}",
            output
        );
    }

    // -- 2. Inspect a String -------------------------------------------------

    #[test]
    fn test_inspect_string() {
        let data = String::from("hello world");
        let inspect = Inspect::new(&data);
        let output = capture_inspect(&inspect);
        assert!(
            output.contains("String"),
            "output should contain 'String': {}",
            output
        );
        assert!(
            output.contains("hello world"),
            "output should contain the value: {}",
            output
        );
    }

    // -- 3. Inspect a struct -------------------------------------------------

    #[derive(Debug)]
    struct TestPoint {
        x: f64,
        y: f64,
    }

    #[test]
    fn test_inspect_struct() {
        let point = TestPoint { x: 1.5, y: 2.5 };
        let inspect = Inspect::new(&point);
        let output = capture_inspect(&inspect);
        assert!(
            output.contains("TestPoint"),
            "output should contain type name: {}",
            output
        );
        assert!(
            output.contains("1.5"),
            "output should contain x value: {}",
            output
        );
        assert!(
            output.contains("2.5"),
            "output should contain y value: {}",
            output
        );
    }

    // -- 4. Inspect with label -----------------------------------------------

    #[test]
    fn test_inspect_with_label() {
        let data = 42u32;
        let inspect = Inspect::new(&data).with_label("answer");
        let output = capture_inspect(&inspect);
        assert!(
            output.contains("Name:"),
            "output should contain 'Name:': {}",
            output
        );
        assert!(
            output.contains("answer"),
            "output should contain label: {}",
            output
        );
    }

    // -- 5. Inspect with doc -------------------------------------------------

    #[test]
    fn test_inspect_with_doc() {
        let data = 42u32;
        let inspect = Inspect::new(&data).with_doc("The meaning of life");
        let output = capture_inspect(&inspect);
        assert!(
            output.contains("Doc:"),
            "output should contain 'Doc:': {}",
            output
        );
        assert!(
            output.contains("The meaning of life"),
            "output should contain doc text: {}",
            output
        );
    }

    // -- 6. Inspect with custom title ----------------------------------------

    #[test]
    fn test_inspect_with_custom_title() {
        let data = vec![1, 2, 3];
        let inspect = Inspect::new(&data).with_title("My Custom Title");
        let output = capture_inspect(&inspect);
        assert!(
            output.contains("My Custom Title"),
            "output should contain custom title: {}",
            output
        );
    }

    // -- 7. Inspect with pretty=false ----------------------------------------

    #[test]
    fn test_inspect_pretty_false() {
        let data = vec![1, 2, 3];
        let compact = Inspect::new(&data).with_pretty(false);
        let pretty = Inspect::new(&data).with_pretty(true);
        let compact_output = capture_inspect(&compact);
        let pretty_output = capture_inspect(&pretty);
        // Compact output should be shorter (single line debug)
        assert!(
            compact_output.len() <= pretty_output.len(),
            "compact output ({}) should be no longer than pretty output ({})",
            compact_output.len(),
            pretty_output.len()
        );
        // Both should contain the values
        assert!(
            compact_output.contains("[1, 2, 3]"),
            "compact should contain [1, 2, 3]: {}",
            compact_output
        );
    }

    // -- 8. Display trait works ----------------------------------------------

    #[test]
    fn test_display_trait() {
        let data = vec![1, 2, 3];
        let inspect = Inspect::new(&data);
        let output = format!("{}", inspect);
        assert!(!output.is_empty(), "Display output should not be empty");
        assert!(
            output.contains("Vec"),
            "Display output should contain 'Vec': {}",
            output
        );
    }

    // -- 9. Renderable produces segments -------------------------------------

    #[test]
    fn test_renderable_produces_segments() {
        let data = 42u32;
        let inspect = Inspect::new(&data);
        let console = Console::builder().width(80).force_terminal(true).build();
        let options = console.options();
        let segments = inspect.gilt_console(&console, &options);
        assert!(!segments.is_empty(), "Renderable should produce segments");
    }

    // -- 10. Type name is correctly extracted ---------------------------------

    #[test]
    fn test_type_name_extracted() {
        let data = vec![1, 2, 3];
        let inspect = Inspect::new(&data);
        assert!(
            inspect.type_name.contains("Vec"),
            "type_name should contain 'Vec': {}",
            inspect.type_name
        );
    }

    // -- 11. Short type name used in default title ---------------------------

    #[test]
    fn test_short_type_name_in_default_title() {
        let data = vec![1, 2, 3];
        let inspect = Inspect::new(&data);
        let short = inspect.short_type_name();
        assert_eq!(
            short, "Vec",
            "short_type_name should be 'Vec', got: {}",
            short
        );
        // Check it shows in the output
        let output = capture_inspect(&inspect);
        assert!(
            output.contains("Inspect: Vec"),
            "default title should contain 'Inspect: Vec': {}",
            output
        );
    }

    // -- 12. Empty struct inspected ------------------------------------------

    #[derive(Debug)]
    struct EmptyStruct;

    #[test]
    fn test_inspect_empty_struct() {
        let data = EmptyStruct;
        let inspect = Inspect::new(&data);
        let output = capture_inspect(&inspect);
        assert!(
            output.contains("EmptyStruct"),
            "output should contain 'EmptyStruct': {}",
            output
        );
    }

    // -- Additional tests ---------------------------------------------------

    #[test]
    fn test_inspect_option_some() {
        let data: Option<i32> = Some(42);
        let inspect = Inspect::new(&data);
        let output = capture_inspect(&inspect);
        assert!(
            output.contains("Option"),
            "output should contain 'Option': {}",
            output
        );
        assert!(
            output.contains("42"),
            "output should contain '42': {}",
            output
        );
    }

    #[test]
    fn test_inspect_option_none() {
        let data: Option<i32> = None;
        let inspect = Inspect::new(&data);
        let output = capture_inspect(&inspect);
        assert!(
            output.contains("None"),
            "output should contain 'None': {}",
            output
        );
    }

    #[test]
    fn test_inspect_hashmap() {
        use std::collections::HashMap;
        let mut data = HashMap::new();
        data.insert("key", "value");
        let inspect = Inspect::new(&data);
        let output = capture_inspect(&inspect);
        assert!(
            output.contains("HashMap"),
            "output should contain 'HashMap': {}",
            output
        );
        assert!(
            output.contains("key"),
            "output should contain 'key': {}",
            output
        );
    }

    #[test]
    fn test_inspect_builder_chaining() {
        let data = 42u32;
        let inspect = Inspect::new(&data)
            .with_label("answer")
            .with_doc("The answer to everything")
            .with_title("Custom")
            .with_pretty(false);
        let output = capture_inspect(&inspect);
        assert!(output.contains("answer"), "should have label: {}", output);
        assert!(
            output.contains("The answer to everything"),
            "should have doc: {}",
            output
        );
        assert!(
            output.contains("Custom"),
            "should have custom title: {}",
            output
        );
    }

    #[test]
    fn test_inspect_display_with_width() {
        let data = vec![1, 2, 3];
        let inspect = Inspect::new(&data);
        let output = format!("{:40}", inspect);
        assert!(!output.is_empty(), "Display with width should not be empty");
    }

    #[test]
    fn test_short_type_name_simple() {
        let data = 42u32;
        let inspect = Inspect::new(&data);
        assert_eq!(inspect.short_type_name(), "u32");
    }

    #[test]
    fn test_short_type_name_nested_generic() {
        let data: Option<Vec<i32>> = Some(vec![1]);
        let inspect = Inspect::new(&data);
        // Should extract "Option" from "core::option::Option<alloc::vec::Vec<i32>>"
        assert_eq!(inspect.short_type_name(), "Option");
    }

    #[test]
    fn test_type_name_field() {
        let data = String::from("test");
        let inspect = Inspect::new(&data);
        // The full type name should contain the full path
        assert!(inspect.type_name.contains("String"));
    }
}
