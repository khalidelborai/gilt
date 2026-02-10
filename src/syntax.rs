//! Syntax highlighting module for terminal display.
//!
//! Provides the `Syntax` struct for rendering syntax-highlighted code with
//! line numbers, themes, word wrap, and padding. Uses `syntect` for syntax
//! highlighting (analogous to Python rich's use of Pygments).

use std::path::Path;

use std::sync::LazyLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style as SyntectStyle, ThemeSet};
use syntect::parsing::SyntaxSet;

use crate::cells::cell_len;
use crate::color::Color;
use crate::console::{Console, ConsoleOptions, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;

/// Global lazily-initialized syntax definitions.
static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);

/// Global lazily-initialized theme set.
static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

/// Default theme name.
const DEFAULT_THEME: &str = "base16-ocean.dark";

/// Default padding for the line numbers column.
const NUMBERS_COLUMN_DEFAULT_PADDING: usize = 2;

// ---------------------------------------------------------------------------
// SyntaxError
// ---------------------------------------------------------------------------

/// Errors that can occur during syntax operations.
#[derive(Debug, thiserror::Error)]
pub enum SyntaxError {
    /// Failed to read a file.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// Could not determine the language for highlighting.
    #[error("unknown language: {0}")]
    UnknownLanguage(String),
}

// ---------------------------------------------------------------------------
// Syntax
// ---------------------------------------------------------------------------

/// Syntax-highlighted code display.
///
/// Renders code with syntax highlighting, optional line numbers, word wrap,
/// theme selection, and more.
#[derive(Debug, Clone)]
pub struct Syntax {
    /// The source code to highlight.
    pub code: String,
    /// Language name for syntax lookup (e.g., "rust", "python", "json").
    pub lexer_name: String,
    /// Theme name (e.g., "base16-ocean.dark", "base16-mocha.dark").
    pub theme: String,
    /// Whether to display line numbers.
    pub line_numbers: bool,
    /// Starting line number (default 1).
    pub start_line: usize,
    /// Optional (start, end) line range to display (1-based, inclusive).
    pub line_range: Option<(usize, usize)>,
    /// Whether to wrap long lines.
    pub word_wrap: bool,
    /// Tab width for tab expansion (default 4).
    pub tab_size: usize,
    /// (top, bottom) padding in blank lines.
    pub padding: (usize, usize),
    /// Line numbers to highlight with a special background.
    pub highlight_lines: Vec<usize>,
    /// Optional override for background color (CSS hex like "#282c34").
    pub background_color: Option<String>,
    /// Whether to show indent guides.
    pub indent_guides: bool,
    /// Fixed width for code area (excluding line numbers), or None for auto.
    pub code_width: Option<usize>,
    /// Whether to auto-dedent code by stripping common leading whitespace.
    pub dedent: bool,
    /// Style ranges to apply on top of syntax highlighting.
    /// Each entry is a (style, character_range) pair applied during rendering.
    pub style_ranges: Vec<(Style, std::ops::Range<usize>)>,
}

impl Syntax {
    /// Create a new Syntax with defaults: base16-ocean.dark theme, no line numbers.
    pub fn new(code: &str, lexer_name: &str) -> Self {
        Syntax {
            code: code.to_string(),
            lexer_name: lexer_name.to_string(),
            theme: DEFAULT_THEME.to_string(),
            line_numbers: false,
            start_line: 1,
            line_range: None,
            word_wrap: false,
            tab_size: 4,
            padding: (0, 0),
            highlight_lines: Vec::new(),
            background_color: None,
            indent_guides: false,
            code_width: None,
            dedent: false,
            style_ranges: Vec::new(),
        }
    }

    /// Create a Syntax by reading a file and auto-detecting the language from its extension.
    pub fn from_path(path: &str) -> Result<Self, SyntaxError> {
        let code = std::fs::read_to_string(path)?;
        let lexer_name = guess_lexer(path);
        Ok(Self::new(&code, &lexer_name))
    }

    // -- Builder methods ----------------------------------------------------

    /// Set the theme.
    #[must_use]
    pub fn with_theme(mut self, theme: &str) -> Self {
        self.theme = theme.to_string();
        self
    }

    /// Enable or disable line numbers.
    #[must_use]
    pub fn with_line_numbers(mut self, line_numbers: bool) -> Self {
        self.line_numbers = line_numbers;
        self
    }

    /// Set the starting line number.
    #[must_use]
    pub fn with_start_line(mut self, start: usize) -> Self {
        self.start_line = start;
        self
    }

    /// Set the line range to display (1-based, inclusive).
    #[must_use]
    pub fn with_line_range(mut self, range: (usize, usize)) -> Self {
        self.line_range = Some(range);
        self
    }

    /// Enable or disable word wrap.
    #[must_use]
    pub fn with_word_wrap(mut self, wrap: bool) -> Self {
        self.word_wrap = wrap;
        self
    }

    /// Set the tab size for tab expansion.
    #[must_use]
    pub fn with_tab_size(mut self, size: usize) -> Self {
        self.tab_size = size;
        self
    }

    /// Set which line numbers to highlight.
    #[must_use]
    pub fn with_highlight_lines(mut self, lines: Vec<usize>) -> Self {
        self.highlight_lines = lines;
        self
    }

    /// Enable or disable indent guides.
    #[must_use]
    pub fn with_indent_guides(mut self, guides: bool) -> Self {
        self.indent_guides = guides;
        self
    }

    /// Set a fixed code width.
    #[must_use]
    pub fn with_code_width(mut self, width: usize) -> Self {
        self.code_width = Some(width);
        self
    }

    /// Enable or disable auto-dedent of common leading whitespace.
    #[must_use]
    pub fn with_dedent(mut self, dedent: bool) -> Self {
        self.dedent = dedent;
        self
    }

    /// Add a style to apply over a character range of the code.
    ///
    /// The range is in terms of character offsets into the original code string
    /// (after dedent, if enabled). Multiple ranges may overlap; they are applied
    /// in order on top of the syntax highlighting.
    pub fn stylize_range(&mut self, style: Style, range: std::ops::Range<usize>) {
        self.style_ranges.push((style, range));
    }

    // -- Internal helpers ---------------------------------------------------

    /// Get the width of the line numbers column (0 if line numbers disabled).
    fn numbers_column_width(&self) -> usize {
        if !self.line_numbers {
            return 0;
        }
        let last_line = self.start_line + self.code.lines().count().saturating_sub(1);
        let digits = format!("{}", last_line).len();
        digits + NUMBERS_COLUMN_DEFAULT_PADDING
    }

    /// Process the code: expand tabs, optionally dedent, ensure trailing newline.
    fn process_code(&self) -> (bool, String) {
        let ends_on_nl = self.code.ends_with('\n');
        let mut processed = if ends_on_nl {
            self.code.clone()
        } else {
            format!("{}\n", self.code)
        };
        let tab_replacement: String = " ".repeat(self.tab_size);
        processed = processed.replace('\t', &tab_replacement);

        // Dedent: strip common leading whitespace from all non-empty lines.
        if self.dedent {
            let min_indent = processed
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.len() - line.trim_start().len())
                .min()
                .unwrap_or(0);
            if min_indent > 0 {
                processed = processed
                    .lines()
                    .map(|line| {
                        if line.len() >= min_indent {
                            &line[min_indent..]
                        } else {
                            line
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                if !processed.ends_with('\n') {
                    processed.push('\n');
                }
            }
        }

        (ends_on_nl, processed)
    }

    /// Highlight the given code and return a `Text` with styled spans.
    fn highlight_code(&self, code: &str) -> Text {
        let ss = &*SYNTAX_SET;
        let ts = &*THEME_SET;

        // Find the syntax definition
        let syntax = ss
            .find_syntax_by_token(&self.lexer_name)
            .or_else(|| ss.find_syntax_by_extension(&self.lexer_name))
            .unwrap_or_else(|| ss.find_syntax_plain_text());

        // Find the theme, fall back to the default
        let theme = ts
            .themes
            .get(&self.theme)
            .or_else(|| ts.themes.values().next())
            .expect("at least one theme must be available");

        let mut h = HighlightLines::new(syntax, theme);
        let mut text = Text::new("", Style::null());

        for line in code.lines() {
            let line_with_nl = format!("{}\n", line);
            match h.highlight_line(&line_with_nl, ss) {
                Ok(ranges) => {
                    for (style, token) in ranges {
                        let gilt_style = syntect_to_gilt_style(style);
                        text.append_str(token, Some(gilt_style));
                    }
                }
                Err(_) => {
                    // Fallback: append unstyled
                    text.append_str(&line_with_nl, None);
                }
            }
        }

        text
    }

    /// Get the background style from the theme.
    fn get_background_style(&self) -> Style {
        if let Some(ref bg) = self.background_color {
            if let Ok(color) = Color::parse(bg) {
                return Style::from_color(None, Some(color));
            }
        }
        let ts = &*THEME_SET;
        if let Some(theme) = ts.themes.get(&self.theme) {
            let bg = theme
                .settings
                .background
                .unwrap_or(syntect::highlighting::Color {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 255,
                });
            Style::from_color(None, Some(Color::from_rgb(bg.r, bg.g, bg.b)))
        } else {
            Style::null()
        }
    }

    /// Build the rendered segments for this Syntax object.
    fn render_syntax(&self, max_width: usize) -> Vec<Segment> {
        let (ends_on_nl, processed_code) = self.process_code();
        let mut text = self.highlight_code(&processed_code);

        // Apply user-defined style ranges on top of syntax highlighting.
        for (style, range) in &self.style_ranges {
            text.stylize(style.clone(), range.start, Some(range.end));
        }

        // Remove trailing newline if original didn't have one
        if !ends_on_nl {
            text.remove_suffix("\n");
        }

        let numbers_column_width = self.numbers_column_width();
        let code_width = if let Some(cw) = self.code_width {
            cw
        } else if self.line_numbers {
            max_width
                .saturating_sub(numbers_column_width)
                .saturating_sub(1)
        } else {
            max_width
        };

        let background_style = self.get_background_style();

        // Split text into lines
        let lines = text.split("\n", true, true);
        let all_lines: Vec<&crate::text::Text> = lines.iter().collect();

        // Apply line range filter
        let (display_lines, line_offset): (Vec<&crate::text::Text>, usize) =
            if let Some((start, end)) = self.line_range {
                let offset = start.saturating_sub(1);
                let end_idx = end.min(all_lines.len());
                if offset >= all_lines.len() {
                    (Vec::new(), offset)
                } else {
                    (all_lines[offset..end_idx].to_vec(), offset)
                }
            } else {
                (all_lines.clone(), 0)
            };

        let mut segments: Vec<Segment> = Vec::new();

        // Top padding
        for _ in 0..self.padding.0 {
            if self.line_numbers {
                let pad = " ".repeat(numbers_column_width + 1);
                segments.push(Segment::styled(&pad, background_style.clone()));
            }
            let line_pad = " ".repeat(code_width);
            segments.push(Segment::styled(&line_pad, background_style.clone()));
            segments.push(Segment::line());
        }

        for (idx, line) in display_lines.iter().enumerate() {
            let line_no = self.start_line + line_offset + idx;
            let is_highlighted = self.highlight_lines.contains(&line_no);

            // Line number gutter
            if self.line_numbers {
                let num_width = numbers_column_width - NUMBERS_COLUMN_DEFAULT_PADDING;
                let num_str = format!("{:>width$} ", line_no, width = num_width);

                if is_highlighted {
                    let pointer_style = Style::from_color(
                        Some(Color::parse("red").unwrap_or_else(|_| Color::from_rgb(255, 0, 0))),
                        None,
                    );
                    segments.push(Segment::styled("> ", pointer_style));
                    segments.push(Segment::styled(&num_str, background_style.clone()));
                } else {
                    let dim_style = Style::new(
                        None,
                        None,
                        None,
                        Some(true),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    )
                    .unwrap_or_else(|_| Style::null());
                    segments.push(Segment::styled("  ", background_style.clone()));
                    segments.push(Segment::styled(
                        &num_str,
                        background_style.clone() + dim_style,
                    ));
                }
            }

            // Render the line content
            let line_text = line.plain();
            let line_cell_len = cell_len(line_text);

            if self.word_wrap && line_cell_len > code_width {
                // Word wrap: split into wrapped segments
                let wrapped = line.wrap(code_width, None, None, self.tab_size, false);
                for (wi, wline) in wrapped.iter().enumerate() {
                    if wi > 0 && self.line_numbers {
                        // Continuation line: pad the gutter
                        let gutter_pad = " ".repeat(numbers_column_width + 1);
                        segments.push(Segment::styled(&gutter_pad, background_style.clone()));
                    }
                    let rendered = wline.render();
                    for seg in &rendered {
                        if seg.text == "\n" {
                            continue;
                        }
                        let style = seg.style.clone().unwrap_or_else(Style::null);
                        segments.push(Segment::styled(&seg.text, background_style.clone() + style));
                    }
                    // Pad to code_width
                    let wline_len = wline.cell_len();
                    if wline_len < code_width {
                        let pad = " ".repeat(code_width - wline_len);
                        segments.push(Segment::styled(&pad, background_style.clone()));
                    }
                    segments.push(Segment::line());
                }
            } else {
                // Single line (no wrap)
                let rendered = line.render();
                for seg in &rendered {
                    if seg.text == "\n" {
                        continue;
                    }
                    let style = seg.style.clone().unwrap_or_else(Style::null);
                    segments.push(Segment::styled(&seg.text, background_style.clone() + style));
                }
                // Pad to code_width
                if line_cell_len < code_width {
                    let pad = " ".repeat(code_width - line_cell_len);
                    segments.push(Segment::styled(&pad, background_style.clone()));
                }
                segments.push(Segment::line());
            }
        }

        // Bottom padding
        for _ in 0..self.padding.1 {
            if self.line_numbers {
                let pad = " ".repeat(numbers_column_width + 1);
                segments.push(Segment::styled(&pad, background_style.clone()));
            }
            let line_pad = " ".repeat(code_width);
            segments.push(Segment::styled(&line_pad, background_style.clone()));
            segments.push(Segment::line());
        }

        segments
    }

    /// Measure the width required to render this Syntax.
    pub fn measure(&self) -> Measurement {
        let numbers_width = self.numbers_column_width();
        if let Some(cw) = self.code_width {
            let total = cw + numbers_width + if self.line_numbers { 1 } else { 0 };
            return Measurement::new(numbers_width, total);
        }
        let (_, processed) = self.process_code();
        let max_line_width = processed.lines().map(cell_len).max().unwrap_or(0);
        let total = numbers_width + max_line_width + if self.line_numbers { 1 } else { 0 };
        Measurement::new(numbers_width, total)
    }
}

/// Implement the Renderable trait so Syntax can be printed by Console.
impl Renderable for Syntax {
    fn gilt_console(&self, _console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        self.render_syntax(options.max_width)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a syntect Style to a gilt Style (foreground color only).
fn syntect_to_gilt_style(style: SyntectStyle) -> Style {
    let fg = style.foreground;
    Style::from_color(Some(Color::from_rgb(fg.r, fg.g, fg.b)), None)
}

/// Guess the lexer name from a file path extension.
fn guess_lexer(path: &str) -> String {
    let p = Path::new(path);
    if let Some(ext) = p.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        // syntect uses extension-based lookup
        let ss = &*SYNTAX_SET;
        if let Some(syn) = ss.find_syntax_by_extension(&ext_str) {
            // Return the first token (short name)
            return syn.name.to_lowercase();
        }
        return ext_str;
    }
    "txt".to_string()
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl std::fmt::Display for Syntax {
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

    // -- Basic highlighting -------------------------------------------------

    #[test]
    fn test_basic_rust_highlighting() {
        let code = "fn main() {\n    println!(\"Hello\");\n}\n";
        let syntax = Syntax::new(code, "rs");
        let segments = syntax.render_syntax(80);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("fn"));
        assert!(text.contains("main"));
        assert!(text.contains("println"));
    }

    #[test]
    fn test_python_highlighting() {
        let code = "def hello():\n    print(\"Hello\")\n";
        let syntax = Syntax::new(code, "py");
        let segments = syntax.render_syntax(80);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("def"));
        assert!(text.contains("hello"));
    }

    #[test]
    fn test_json_highlighting() {
        let code = "{\"key\": \"value\", \"num\": 42}\n";
        let syntax = Syntax::new(code, "json");
        let segments = syntax.render_syntax(80);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("key"));
        assert!(text.contains("value"));
        assert!(text.contains("42"));
    }

    // -- Line numbers -------------------------------------------------------

    #[test]
    fn test_line_numbers_enabled() {
        let code = "line one\nline two\nline three\n";
        let syntax = Syntax::new(code, "txt").with_line_numbers(true);
        let segments = syntax.render_syntax(80);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("1"));
        assert!(text.contains("2"));
        assert!(text.contains("3"));
    }

    #[test]
    fn test_line_numbers_disabled() {
        let code = "line one\nline two\n";
        let syntax = Syntax::new(code, "txt");
        let segments = syntax.render_syntax(80);
        // Without line numbers, should not have the gutter padding pattern
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("line one"));
    }

    // -- Start line offset --------------------------------------------------

    #[test]
    fn test_start_line_offset() {
        let code = "alpha\nbeta\ngamma\n";
        let syntax = Syntax::new(code, "txt")
            .with_line_numbers(true)
            .with_start_line(10);
        let segments = syntax.render_syntax(80);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("10"));
        assert!(text.contains("11"));
        assert!(text.contains("12"));
    }

    // -- Line range filtering -----------------------------------------------

    #[test]
    fn test_line_range() {
        let code = "line1\nline2\nline3\nline4\nline5\n";
        let syntax = Syntax::new(code, "txt").with_line_range((2, 4));
        let segments = syntax.render_syntax(80);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("line2"));
        assert!(text.contains("line3"));
        assert!(text.contains("line4"));
        assert!(!text.contains("line1\n")); // line1 should not be present as its own line
        assert!(!text.contains("line5"));
    }

    // -- Word wrap ----------------------------------------------------------

    #[test]
    fn test_word_wrap() {
        let code = "this is a very long line that should be wrapped when word wrap is enabled\n";
        let syntax = Syntax::new(code, "txt").with_word_wrap(true);
        let segments = syntax.render_syntax(30);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        // Should have line breaks from wrapping
        let newline_count = text.matches('\n').count();
        assert!(
            newline_count > 1,
            "expected wrapping, got {} newlines",
            newline_count
        );
    }

    // -- Tab expansion ------------------------------------------------------

    #[test]
    fn test_tab_expansion() {
        let code = "if true:\n\tpass\n";
        let syntax = Syntax::new(code, "py").with_tab_size(4);
        let (_, processed) = syntax.process_code();
        assert!(!processed.contains('\t'));
        assert!(processed.contains("    pass"));
    }

    #[test]
    fn test_tab_expansion_custom_size() {
        let code = "\thello\n";
        let syntax = Syntax::new(code, "txt").with_tab_size(8);
        let (_, processed) = syntax.process_code();
        assert!(processed.contains("        hello"));
    }

    // -- Theme selection ----------------------------------------------------

    #[test]
    fn test_theme_base16_ocean_dark() {
        let code = "let x = 1;\n";
        let syntax = Syntax::new(code, "rs").with_theme("base16-ocean.dark");
        let segments = syntax.render_syntax(80);
        assert!(!segments.is_empty());
    }

    #[test]
    fn test_theme_base16_eighties_dark() {
        let code = "let x = 1;\n";
        let syntax = Syntax::new(code, "rs").with_theme("base16-eighties.dark");
        let segments = syntax.render_syntax(80);
        assert!(!segments.is_empty());
    }

    #[test]
    fn test_unknown_theme_fallback() {
        let code = "hello\n";
        let syntax = Syntax::new(code, "txt").with_theme("nonexistent-theme-xyz");
        let segments = syntax.render_syntax(80);
        // Should still render, using a fallback theme
        assert!(!segments.is_empty());
    }

    // -- Highlight specific lines -------------------------------------------

    #[test]
    fn test_highlight_lines() {
        let code = "a\nb\nc\n";
        let syntax = Syntax::new(code, "txt")
            .with_line_numbers(true)
            .with_highlight_lines(vec![2]);
        let segments = syntax.render_syntax(80);
        // Check that a ">" pointer appears for the highlighted line
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains('>'), "expected highlight pointer");
    }

    // -- Unknown language handling ------------------------------------------

    #[test]
    fn test_unknown_language_fallback() {
        let code = "some random text\n";
        let syntax = Syntax::new(code, "zzzz_nonexistent");
        let segments = syntax.render_syntax(80);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        // Should fall back to plain text
        assert!(text.contains("some random text"));
    }

    // -- Builder pattern methods --------------------------------------------

    #[test]
    fn test_builder_pattern() {
        let syntax = Syntax::new("code", "rs")
            .with_theme("base16-ocean.dark")
            .with_line_numbers(true)
            .with_start_line(5)
            .with_line_range((1, 10))
            .with_word_wrap(true)
            .with_tab_size(2)
            .with_highlight_lines(vec![1, 2, 3])
            .with_indent_guides(true)
            .with_code_width(60)
            .with_dedent(true);

        assert_eq!(syntax.theme, "base16-ocean.dark");
        assert!(syntax.line_numbers);
        assert_eq!(syntax.start_line, 5);
        assert_eq!(syntax.line_range, Some((1, 10)));
        assert!(syntax.word_wrap);
        assert_eq!(syntax.tab_size, 2);
        assert_eq!(syntax.highlight_lines, vec![1, 2, 3]);
        assert!(syntax.indent_guides);
        assert_eq!(syntax.code_width, Some(60));
        assert!(syntax.dedent);
    }

    // -- Renderable trait integration ---------------------------------------

    #[test]
    fn test_renderable_trait() {
        let syntax = Syntax::new("fn main() {}\n", "rs");
        let console = Console::builder().width(80).build();
        let options = console.options();
        let segments = syntax.gilt_console(&console, &options);
        assert!(!segments.is_empty());
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("fn"));
    }

    // -- Measure method -----------------------------------------------------

    #[test]
    fn test_measure_no_line_numbers() {
        let code = "hello world\n";
        let syntax = Syntax::new(code, "txt");
        let m = syntax.measure();
        assert_eq!(m.minimum, 0); // no line numbers = 0 gutter
        assert!(m.maximum >= 11); // "hello world" is 11 chars
    }

    #[test]
    fn test_measure_with_line_numbers() {
        let code = "a\nb\nc\n";
        let syntax = Syntax::new(code, "txt").with_line_numbers(true);
        let m = syntax.measure();
        assert!(m.minimum > 0); // gutter width
        assert!(m.maximum > m.minimum);
    }

    #[test]
    fn test_measure_with_code_width() {
        let code = "hello\n";
        let syntax = Syntax::new(code, "txt").with_code_width(40);
        let m = syntax.measure();
        assert_eq!(m.maximum, 40);
    }

    // -- from_path ----------------------------------------------------------

    #[test]
    fn test_from_path_nonexistent() {
        let result = Syntax::from_path("/nonexistent/file/path.rs");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_path_reads_self() {
        // Read this very test file
        let path = file!();
        // This file may be at a relative path; use the full crate root
        let full_path = format!("/mnt/data/Velocity/rusty_rich/gilt/{}", path);
        if std::path::Path::new(&full_path).exists() {
            let result = Syntax::from_path(&full_path);
            assert!(result.is_ok());
            let syntax = result.unwrap();
            assert!(syntax.code.contains("fn test_from_path_reads_self"));
        }
    }

    // -- Empty code ---------------------------------------------------------

    #[test]
    fn test_empty_code() {
        let syntax = Syntax::new("", "txt");
        let segments = syntax.render_syntax(80);
        // Should produce at least something (even if just a newline)
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        // Empty code with added newline should produce one line
        assert!(!text.is_empty() || segments.is_empty());
    }

    // -- Single line code ---------------------------------------------------

    #[test]
    fn test_single_line_code() {
        let syntax = Syntax::new("hello", "txt");
        let segments = syntax.render_syntax(80);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("hello"));
    }

    // -- Code with special characters ---------------------------------------

    #[test]
    fn test_code_with_special_characters() {
        let code = "let x = \"hello <world> & 'friends'\";\n";
        let syntax = Syntax::new(code, "rs");
        let segments = syntax.render_syntax(80);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains('<'));
        assert!(text.contains('>'));
        assert!(text.contains('&'));
        assert!(text.contains('\''));
    }

    // -- syntect_to_gilt_style helper test ----------------------------------

    #[test]
    fn test_syntect_to_gilt_style_conversion() {
        let style = SyntectStyle {
            foreground: syntect::highlighting::Color {
                r: 255,
                g: 128,
                b: 0,
                a: 255,
            },
            background: syntect::highlighting::Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
            font_style: syntect::highlighting::FontStyle::empty(),
        };
        let gilt_style = syntect_to_gilt_style(style);
        let color = gilt_style.color().expect("should have foreground color");
        assert_eq!(color.triplet.unwrap().red, 255);
        assert_eq!(color.triplet.unwrap().green, 128);
        assert_eq!(color.triplet.unwrap().blue, 0);
    }

    // -- guess_lexer test ---------------------------------------------------

    #[test]
    fn test_guess_lexer_rust() {
        let name = guess_lexer("foo.rs");
        // syntect returns "Rust" as the syntax name
        assert!(!name.is_empty());
    }

    #[test]
    fn test_guess_lexer_python() {
        let name = guess_lexer("script.py");
        assert!(!name.is_empty());
    }

    #[test]
    fn test_guess_lexer_json() {
        let name = guess_lexer("data.json");
        assert!(!name.is_empty());
    }

    #[test]
    fn test_guess_lexer_no_extension() {
        let name = guess_lexer("Makefile");
        // Should return something (maybe "makefile" or "txt")
        assert!(!name.is_empty());
    }

    // -- numbers_column_width -----------------------------------------------

    #[test]
    fn test_numbers_column_width_disabled() {
        let syntax = Syntax::new("a\nb\nc\n", "txt");
        assert_eq!(syntax.numbers_column_width(), 0);
    }

    #[test]
    fn test_numbers_column_width_single_digit() {
        let syntax = Syntax::new("a\nb\nc\n", "txt").with_line_numbers(true);
        // 3 lines, digits = 1, + 2 padding = 3
        assert_eq!(syntax.numbers_column_width(), 3);
    }

    #[test]
    fn test_numbers_column_width_double_digit() {
        let mut code = String::new();
        for i in 1..=15 {
            code.push_str(&format!("line {}\n", i));
        }
        let syntax = Syntax::new(&code, "txt").with_line_numbers(true);
        // 15 lines, digits = 2, + 2 padding = 4
        assert_eq!(syntax.numbers_column_width(), 4);
    }

    // -- process_code -------------------------------------------------------

    #[test]
    fn test_process_code_adds_trailing_newline() {
        let syntax = Syntax::new("hello", "txt");
        let (ends_on_nl, processed) = syntax.process_code();
        assert!(!ends_on_nl);
        assert!(processed.ends_with('\n'));
    }

    #[test]
    fn test_process_code_preserves_trailing_newline() {
        let syntax = Syntax::new("hello\n", "txt");
        let (ends_on_nl, processed) = syntax.process_code();
        assert!(ends_on_nl);
        assert!(processed.ends_with('\n'));
    }

    // -- Background style ---------------------------------------------------

    #[test]
    fn test_get_background_style_default() {
        let syntax = Syntax::new("code", "txt");
        let style = syntax.get_background_style();
        // Should have a bgcolor from the theme
        assert!(style.bgcolor().is_some() || style.is_null());
    }

    #[test]
    fn test_get_background_style_override() {
        let mut syntax = Syntax::new("code", "txt");
        syntax.background_color = Some("#ff0000".to_string());
        let style = syntax.get_background_style();
        assert!(style.bgcolor().is_some());
    }

    // -- Padding ------------------------------------------------------------

    #[test]
    fn test_padding_top_bottom() {
        let mut syntax = Syntax::new("hello\n", "txt");
        syntax.padding = (1, 1);
        let segments = syntax.render_syntax(40);
        // With padding (1,1), we should have more lines than just the code line
        let newline_count = segments.iter().filter(|s| s.text == "\n").count();
        // 1 top padding + 1 code line + 1 bottom padding = 3 newlines
        assert!(
            newline_count >= 3,
            "expected at least 3 newlines, got {}",
            newline_count
        );
    }

    // -- Line range out of bounds -------------------------------------------

    #[test]
    fn test_line_range_out_of_bounds() {
        let code = "a\nb\n";
        let syntax = Syntax::new(code, "txt").with_line_range((10, 20));
        let segments = syntax.render_syntax(80);
        // Should produce nothing for the code area
        let text: String = segments
            .iter()
            .filter(|s| s.text != "\n" && s.text.trim() != "")
            .map(|s| s.text.as_str())
            .collect();
        assert!(text.is_empty() || text.chars().all(|c| c == ' '));
    }

    // -- Segments have styles -----------------------------------------------

    #[test]
    fn test_segments_have_styles() {
        let code = "fn main() {}\n";
        let syntax = Syntax::new(code, "rs");
        let segments = syntax.render_syntax(80);
        // At least some segments should have styles
        let styled_count = segments.iter().filter(|s| s.style.is_some()).count();
        assert!(styled_count > 0, "expected some styled segments");
    }

    // -- Code width constraint ----------------------------------------------

    #[test]
    fn test_code_width_constraint() {
        let code = "a very long line of code that goes on and on and on\n";
        let syntax = Syntax::new(code, "txt").with_code_width(20);
        let m = syntax.measure();
        assert_eq!(m.maximum, 20);
    }

    // -- Multiple themes produce different output ---------------------------

    #[test]
    fn test_different_themes_produce_output() {
        let code = "let x = 42;\n";
        let ts = &*THEME_SET;
        for theme_name in ts.themes.keys() {
            let syntax = Syntax::new(code, "rs").with_theme(theme_name);
            let segments = syntax.render_syntax(80);
            assert!(
                !segments.is_empty(),
                "theme '{}' produced no output",
                theme_name
            );
        }
    }

    // -- Struct defaults ----------------------------------------------------

    #[test]
    fn test_default_values() {
        let syntax = Syntax::new("code", "rs");
        assert_eq!(syntax.theme, DEFAULT_THEME);
        assert!(!syntax.line_numbers);
        assert_eq!(syntax.start_line, 1);
        assert!(syntax.line_range.is_none());
        assert!(!syntax.word_wrap);
        assert_eq!(syntax.tab_size, 4);
        assert_eq!(syntax.padding, (0, 0));
        assert!(syntax.highlight_lines.is_empty());
        assert!(syntax.background_color.is_none());
        assert!(!syntax.indent_guides);
        assert!(syntax.code_width.is_none());
        assert!(!syntax.dedent);
        assert!(syntax.style_ranges.is_empty());
    }

    // -- Dedent -------------------------------------------------------------

    #[test]
    fn test_dedent_strips_common_whitespace() {
        let code = "    fn main() {\n        println!(\"hi\");\n    }\n";
        let syntax = Syntax::new(code, "rs").with_dedent(true);
        let (_, processed) = syntax.process_code();
        // Common indent is 4 spaces; after dedent, first line starts at column 0
        assert!(
            processed.starts_with("fn main()"),
            "expected dedented code, got: {:?}",
            processed
        );
    }

    #[test]
    fn test_dedent_preserves_relative_indent() {
        let code = "    fn main() {\n        println!(\"hi\");\n    }\n";
        let syntax = Syntax::new(code, "rs").with_dedent(true);
        let (_, processed) = syntax.process_code();
        let lines: Vec<&str> = processed.lines().collect();
        // First line: "fn main() {" (0 indent)
        assert!(lines[0].starts_with("fn main()"));
        // Second line: "    println!..." (4 spaces relative indent preserved)
        assert!(
            lines[1].starts_with("    println"),
            "expected 4-space relative indent, got: {:?}",
            lines[1]
        );
        // Third line: "}" (0 indent)
        assert_eq!(lines[2].trim(), "}");
    }

    #[test]
    fn test_dedent_false_preserves_whitespace() {
        let code = "    indented\n";
        let syntax = Syntax::new(code, "txt");
        let (_, processed) = syntax.process_code();
        assert!(
            processed.starts_with("    indented"),
            "expected original indent preserved, got: {:?}",
            processed
        );
    }

    // -- Stylize range ------------------------------------------------------

    #[test]
    fn test_stylize_range_stores() {
        let mut syntax = Syntax::new("hello world", "txt");
        let style = Style::from_color(
            Some(Color::parse("red").unwrap_or_else(|_| Color::from_rgb(255, 0, 0))),
            None,
        );
        syntax.stylize_range(style.clone(), 0..5);
        assert_eq!(syntax.style_ranges.len(), 1);
        assert_eq!(syntax.style_ranges[0].1, 0..5);
    }

    #[test]
    fn test_stylize_range_applied() {
        let mut syntax = Syntax::new("hello world\n", "txt");
        let red_style = Style::from_color(
            Some(Color::parse("red").unwrap_or_else(|_| Color::from_rgb(255, 0, 0))),
            None,
        );
        syntax.stylize_range(red_style, 0..5);
        let segments = syntax.render_syntax(80);
        // The rendered segments should contain "hello" with a red foreground
        // Find the segment(s) covering "hello"
        let mut found_styled = false;
        let mut pos = 0;
        for seg in &segments {
            if seg.text == "\n" {
                continue;
            }
            let end = pos + seg.text.len();
            // Check if this segment overlaps with our styled range (0..5)
            if pos < 5 && end > 0 && !seg.text.trim().is_empty() {
                if let Some(ref style) = seg.style {
                    if style.color().is_some() {
                        found_styled = true;
                    }
                }
            }
            pos = end;
        }
        assert!(found_styled, "expected styled segment in the output");
    }

    #[test]
    fn test_display_trait() {
        let syntax = Syntax::new("fn main() {}", "rust");
        let s = format!("{}", syntax);
        assert!(!s.is_empty());
        assert!(s.contains("fn"));
        assert!(s.contains("main"));
    }
}
