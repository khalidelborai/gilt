//! Syntax highlighting module for terminal display.
//!
//! Provides the `Syntax` struct for rendering syntax-highlighted code with
//! line numbers, themes, word wrap, and padding. Uses `syntect` for syntax
//! highlighting (analogous to the use of Pygments).

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use std::sync::LazyLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style as SyntectStyle, Theme as SyntectTheme, ThemeSet};
use syntect::parsing::SyntaxSet;

use crate::cells::cell_len;
use crate::color::blend_rgb;
use crate::color::color_triplet::ColorTriplet;
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

/// Default theme name. Matches the upstream library default — a Monokai
/// variant available in the bundled `syntect` themes (`Solarized (dark)`
/// in syntect = `"Solarized (dark)"`; for "monokai" we use the bundled
/// `base16-mocha.dark` which is the closest near-Monokai theme that
/// `syntect` ships by default). If the named theme is missing at render
/// time, [`Syntax::render_syntax`] falls back to `base16-ocean.dark`.
const DEFAULT_THEME: &str = "base16-mocha.dark";

/// Default padding for the line numbers column.
const NUMBERS_COLUMN_DEFAULT_PADDING: usize = 2;

/// Shorthand input for [`unpack_padding`] — accepts the same forms as CSS
/// `padding`: a single value, two values (vertical, horizontal), three values
/// (top, horizontal, bottom), or four values (top, right, bottom, left).
#[derive(Debug, Clone, Copy)]
pub enum PaddingSpec {
    /// Apply the same value to all four sides.
    Uniform(usize),
    /// `(vertical, horizontal)`.
    VertHoriz(usize, usize),
    /// `(top, horizontal, bottom)`.
    TopHorizBottom(usize, usize, usize),
    /// `(top, right, bottom, left)` — already a full 4-tuple.
    Full(usize, usize, usize, usize),
}

/// Expand a [`PaddingSpec`] into a `(top, right, bottom, left)` tuple, matching
/// rich v14.1.0's `Padding.unpack` and the CSS `padding` shorthand convention.
///
/// ```
/// # use gilt::syntax::{PaddingSpec, unpack_padding};
/// assert_eq!(unpack_padding(PaddingSpec::Uniform(1)), (1, 1, 1, 1));
/// assert_eq!(unpack_padding(PaddingSpec::VertHoriz(2, 4)), (2, 4, 2, 4));
/// ```
pub fn unpack_padding(spec: PaddingSpec) -> (usize, usize, usize, usize) {
    match spec {
        PaddingSpec::Uniform(n) => (n, n, n, n),
        PaddingSpec::VertHoriz(v, h) => (v, h, v, h),
        PaddingSpec::TopHorizBottom(t, h, b) => (t, h, b, h),
        PaddingSpec::Full(t, r, b, l) => (t, r, b, l),
    }
}

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
    /// `(top, right, bottom, left)` padding (top/bottom in blank lines, left/right
    /// in spaces inside the code area). Mirrors the CSS shorthand convention used
    /// by `Padding` and matches rich v14.1.0+ `Syntax.padding` (4-tuple).
    ///
    /// Use `Self::with_padding` / [`unpack_padding`] to construct from any
    /// shorthand variant: `n` → `(n, n, n, n)`, `(v, h)` → `(v, h, v, h)`,
    /// `(t, h, b)` → `(t, h, b, h)`, `(t, r, b, l)` → as-is.
    pub padding: (usize, usize, usize, usize),
    /// Line numbers to highlight with a special background.
    /// Stored as a `HashSet` for O(1) lookup per rendered line.
    pub highlight_lines: HashSet<usize>,
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
    /// Optional syntect theme injected via [`with_syntect_theme`](Self::with_syntect_theme).
    ///
    /// When set, this theme is used instead of the `theme` name field for both
    /// syntax highlighting and background-color derivation.
    pub injected_theme: Option<Arc<SyntectTheme>>,
}

impl Syntax {
    /// Create a new Syntax with defaults: `base16-mocha.dark` theme (Monokai-like), no line numbers.
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
            padding: (0, 0, 0, 0),
            highlight_lines: HashSet::new(),
            background_color: None,
            indent_guides: false,
            code_width: None,
            dedent: false,
            style_ranges: Vec::new(),
            injected_theme: None,
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
    ///
    /// Accepts any iterator or `Vec`; lines are stored in a `HashSet` for O(1)
    /// per-line lookup during rendering.
    #[must_use]
    pub fn with_highlight_lines(mut self, lines: impl IntoIterator<Item = usize>) -> Self {
        self.highlight_lines = lines.into_iter().collect();
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

    /// Set padding from a [`PaddingSpec`] shorthand, expanding to the full
    /// `(top, right, bottom, left)` tuple stored in `self.padding`.
    ///
    /// ```
    /// # use gilt::syntax::{Syntax, PaddingSpec};
    /// let s = Syntax::new("fn main() {}", "rs")
    ///     .with_padding(PaddingSpec::VertHoriz(1, 2));
    /// assert_eq!(s.padding, (1, 2, 1, 2));
    /// ```
    #[must_use]
    pub fn with_padding(mut self, spec: PaddingSpec) -> Self {
        self.padding = unpack_padding(spec);
        self
    }

    /// Inject a [`syntect::highlighting::Theme`] to use for highlighting.
    ///
    /// When set, this theme is used instead of the string `theme` field for both
    /// syntax token colouring and background-color derivation. The `theme` name
    /// field is preserved but ignored during rendering.
    ///
    /// The theme is stored behind an [`Arc`] so cloning a `Syntax` value is
    /// cheap even for large themes.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::syntax::Syntax;
    /// use syntect::highlighting::ThemeSet;
    ///
    /// let ts = ThemeSet::load_defaults();
    /// let theme = ts.themes["base16-ocean.dark"].clone();
    /// let syntax = Syntax::new("let x = 1;", "rs").with_syntect_theme(theme);
    /// let s = format!("{}", syntax);
    /// assert!(!s.is_empty());
    /// ```
    #[must_use]
    pub fn with_syntect_theme(mut self, theme: SyntectTheme) -> Self {
        self.injected_theme = Some(Arc::new(theme));
        self
    }

    /// Load a `.tmTheme` file from disk and return a [`syntect::highlighting::Theme`].
    ///
    /// The returned theme can be passed to [`with_syntect_theme`](Self::with_syntect_theme).
    ///
    /// This function is not available on `wasm32` targets.
    ///
    /// # Errors
    ///
    /// Returns a [`SyntaxError::IoError`] if the file cannot be read or parsed
    /// by syntect.
    ///
    /// # Examples
    ///
    /// Requires the opt-in `syntax-theme-file` feature (native only).
    ///
    /// ```no_run
    /// # #[cfg(all(feature = "syntax-theme-file", not(target_arch = "wasm32")))] {
    /// use gilt::syntax::Syntax;
    ///
    /// let theme = Syntax::load_theme_from_file("/path/to/My.tmTheme").unwrap();
    /// let syntax = Syntax::new("fn main() {}", "rs").with_syntect_theme(theme);
    /// # }
    /// ```
    #[cfg(all(feature = "syntax-theme-file", not(target_arch = "wasm32")))]
    pub fn load_theme_from_file(
        path: impl AsRef<std::path::Path>,
    ) -> Result<SyntectTheme, SyntaxError> {
        ThemeSet::get_theme(path.as_ref())
            .map_err(|e| SyntaxError::IoError(std::io::Error::other(e.to_string())))
    }

    /// Add a style to apply over a flat character range of the code.
    ///
    /// The range is in terms of character offsets into the original code string
    /// (after dedent, if enabled). Multiple ranges may overlap; they are applied
    /// in order on top of the syntax highlighting.
    pub fn stylize_range(&mut self, style: Style, range: std::ops::Range<usize>) {
        self.style_ranges.push((style, range));
    }

    /// Add a style to apply over a `(line, col)` range of the code.
    ///
    /// Both `start` and `end` are `(line, col)` pairs where line is 1-based and
    /// col is 0-based character offset within the line.  The pair is converted
    /// to a flat character offset and delegated to [`stylize_range`].
    ///
    /// Matches the `start=(line, col), end=(line, col)` form used by Python rich.
    ///
    /// [`stylize_range`]: Self::stylize_range
    pub fn stylize_range_linecol(
        &mut self,
        style: Style,
        start: (usize, usize),
        end: (usize, usize),
    ) {
        let flat_start = self.linecol_to_offset(start);
        let flat_end = self.linecol_to_offset(end);
        if flat_start <= flat_end {
            self.stylize_range(style, flat_start..flat_end);
        }
    }

    /// Convert a 1-based `(line, col)` pair to a flat character offset into
    /// the code string.  Lines are terminated by `\n`.
    fn linecol_to_offset(&self, (line, col): (usize, usize)) -> usize {
        let mut offset = 0usize;
        let mut current_line = 1usize;
        for ch in self.code.chars() {
            if current_line == line {
                // col is a char-count offset within the line.
                // Use char_indices to avoid a manual counter variable.
                let line_slice = &self.code[offset..];
                for (ci, (byte_off, inner_ch)) in line_slice.char_indices().enumerate() {
                    if ci == col {
                        return offset + byte_off;
                    }
                    if inner_ch == '\n' {
                        break;
                    }
                }
                // col past end of line or line ends without reaching col → clamp.
                return offset + line_slice.find('\n').unwrap_or(line_slice.len());
            }
            if ch == '\n' {
                current_line += 1;
            }
            offset += ch.len_utf8();
        }
        offset // line past end → clamp to EOF
    }

    // -- Internal helpers ---------------------------------------------------

    /// Get the width of the line numbers column (0 if line numbers disabled).
    ///
    /// Uses the exclusive-end line number (`start_line + line_count`) for the
    /// digit count, matching rich's `len(str(start_line + line_count)) + 2`.
    fn numbers_column_width(&self) -> usize {
        if !self.line_numbers {
            return 0;
        }
        let line_count = self.code.lines().count();
        let exclusive_end = self.start_line + line_count;
        let digits = format!("{}", exclusive_end).len();
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
        // min_indent is measured in Unicode *characters* (not bytes) so that
        // non-ASCII leading whitespace (e.g. U+3000 IDEOGRAPHIC SPACE) does not
        // cause a byte-boundary slice panic.
        if self.dedent {
            let min_indent = processed
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
                .min()
                .unwrap_or(0);
            if min_indent > 0 {
                processed = processed
                    .lines()
                    .map(|line| {
                        // Compute byte offset of the min_indent-th character safely.
                        match line.char_indices().nth(min_indent) {
                            Some((byte_idx, _)) => &line[byte_idx..],
                            // Line is shorter than min_indent chars (e.g. empty
                            // lines that passed the non-empty filter).
                            None => line,
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

    /// Return the effective syntect theme: injected if set, otherwise from the name field.
    fn effective_syntect_theme(&self) -> &SyntectTheme {
        if let Some(ref injected) = self.injected_theme {
            return injected.as_ref();
        }
        let ts = &*THEME_SET;
        ts.themes
            .get(&self.theme)
            .or_else(|| ts.themes.values().next())
            .expect("at least one theme must be available")
    }

    /// Highlight the given code and return a `Text` with styled spans.
    fn highlight_code(&self, code: &str) -> Text {
        let ss = &*SYNTAX_SET;

        // Find the syntax definition
        let syntax = ss
            .find_syntax_by_token(&self.lexer_name)
            .or_else(|| ss.find_syntax_by_extension(&self.lexer_name))
            .unwrap_or_else(|| ss.find_syntax_plain_text());

        // Find the theme via the unified helper
        let theme = self.effective_syntect_theme();

        let mut h = HighlightLines::new(syntax, theme);
        let mut text = Text::new("", Style::null());

        // `process_code` guarantees `code` ends with `\n`, so
        // `split_inclusive('\n')` yields lines that already include the
        // trailing newline — no per-line format! allocation needed.
        for line_with_nl in code.split_inclusive('\n') {
            match h.highlight_line(line_with_nl, ss) {
                Ok(ranges) => {
                    for (style, token) in ranges {
                        let gilt_style = syntect_to_gilt_style(style);
                        text.append_str(token, Some(gilt_style));
                    }
                }
                Err(_) => {
                    // Fallback: append unstyled
                    text.append_str(line_with_nl, None);
                }
            }
        }

        text
    }

    /// Highlight a single inline code string and return styled Text spans.
    /// Used by the markdown renderer for inline `code` with a lexer set.
    pub fn highlight_inline(code: &str, lexer_name: &str, theme_name: &str) -> Text {
        let syn = Syntax::new(code, lexer_name).with_theme(theme_name);
        let mut t = syn.highlight_code(code);
        // Drop trailing newline that highlight_code may append.
        t.remove_suffix("\n");
        t
    }

    /// Get the background style from the theme.
    fn get_background_style(&self) -> Style {
        if let Some(ref bg) = self.background_color {
            if let Ok(color) = Color::parse(bg) {
                return Style::from_color(None, Some(color));
            }
        }
        let theme = self.effective_syntect_theme();
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
    }

    /// Return `(normal_style, highlighted_style)` for line number rendering.
    ///
    /// Follows rich: blend bg→fg at 0.3 for normal numbers and 0.9 for
    /// highlighted ones.  Falls back to a dim/bold style when the theme lacks
    /// an explicit foreground color.
    fn line_number_styles(&self) -> (Style, Style) {
        let theme = self.effective_syntect_theme();
        // Obtain bg and fg triplets from the theme settings.
        let bg = theme
            .settings
            .background
            .unwrap_or(syntect::highlighting::Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            });
        let fg = theme
            .settings
            .foreground
            .unwrap_or(syntect::highlighting::Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            });
        let bg_t = ColorTriplet::new(bg.r, bg.g, bg.b);
        let fg_t = ColorTriplet::new(fg.r, fg.g, fg.b);

        let normal_t = blend_rgb(bg_t, fg_t, 0.3);
        let highlight_t = blend_rgb(bg_t, fg_t, 0.9);

        let bg_style = self.get_background_style();
        let normal_style = bg_style.clone()
            + Style::from_color(
                Some(Color::from_rgb(normal_t.red, normal_t.green, normal_t.blue)),
                None,
            );
        let highlighted_style = bg_style
            + Style::from_color(
                Some(Color::from_rgb(
                    highlight_t.red,
                    highlight_t.green,
                    highlight_t.blue,
                )),
                None,
            );

        (normal_style, highlighted_style)
    }

    /// Build the rendered segments for this Syntax object.
    fn render_syntax(&self, max_width: usize) -> Vec<Segment> {
        let (ends_on_nl, processed_code) = self.process_code();
        let mut text = self.highlight_code(&processed_code);

        // Apply indent guides before line splitting (Finding #6).
        if self.indent_guides {
            // Use a dim comment-style guide character to match rich behaviour.
            let guide_style = Style::parse("dim");
            text = text.with_indent_guides(Some(self.tab_size), '│', guide_style);
        }

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

        // Build blended line-number colors (Finding #8).
        // Rich blends bg→fg at 0.3 for normal numbers and ~0.9 for highlighted.
        let (number_style, highlighted_number_style) = self.line_number_styles();

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

        // Horizontal padding (padding.1 = right, padding.3 = left).
        let left_pad = self.padding.3;
        let right_pad = self.padding.1;
        let left_str: String = " ".repeat(left_pad);
        let right_str: String = " ".repeat(right_pad);

        // Top padding (padding.0 == top in 4-tuple)
        for _ in 0..self.padding.0 {
            if self.line_numbers {
                let pad = " ".repeat(numbers_column_width + 1);
                segments.push(Segment::styled(&pad, background_style.clone()));
            }
            let line_pad = " ".repeat(left_pad + code_width + right_pad);
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
                    // Use the strongly-blended (0.9) color for highlighted line numbers.
                    segments.push(Segment::styled(&num_str, highlighted_number_style.clone()));
                } else {
                    // Use the dimly-blended (0.3) color for normal line numbers.
                    segments.push(Segment::styled("  ", background_style.clone()));
                    segments.push(Segment::styled(&num_str, number_style.clone()));
                }
            }

            // Left padding for the code area.
            if left_pad > 0 {
                segments.push(Segment::styled(&left_str, background_style.clone()));
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
                    if wi > 0 && left_pad > 0 {
                        segments.push(Segment::styled(&left_str, background_style.clone()));
                    }
                    let rendered = wline.render();
                    for seg in &rendered {
                        if seg.text == "\n" {
                            continue;
                        }
                        let style = seg.style_owned();
                        segments.push(Segment::styled(&seg.text, background_style.clone() + style));
                    }
                    // Pad to code_width
                    let wline_len = wline.cell_len();
                    if wline_len < code_width {
                        let pad = " ".repeat(code_width - wline_len);
                        segments.push(Segment::styled(&pad, background_style.clone()));
                    }
                    // Right padding for the code area.
                    if right_pad > 0 {
                        segments.push(Segment::styled(&right_str, background_style.clone()));
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
                    let style = seg.style_owned();
                    segments.push(Segment::styled(&seg.text, background_style.clone() + style));
                }
                // Pad to code_width
                if line_cell_len < code_width {
                    let pad = " ".repeat(code_width - line_cell_len);
                    segments.push(Segment::styled(&pad, background_style.clone()));
                }
                // Right padding for the code area.
                if right_pad > 0 {
                    segments.push(Segment::styled(&right_str, background_style.clone()));
                }
                segments.push(Segment::line());
            }
        }

        // Bottom padding (padding.2 == bottom in 4-tuple)
        for _ in 0..self.padding.2 {
            if self.line_numbers {
                let pad = " ".repeat(numbers_column_width + 1);
                segments.push(Segment::styled(&pad, background_style.clone()));
            }
            let line_pad = " ".repeat(left_pad + code_width + right_pad);
            segments.push(Segment::styled(&line_pad, background_style.clone()));
            segments.push(Segment::line());
        }

        segments
    }

    /// Measure the width required to render this Syntax.
    ///
    /// Respects `line_range` when set — only visible lines contribute to the
    /// maximum width, matching what `render_syntax` actually outputs.
    pub fn measure(&self) -> Measurement {
        let numbers_width = self.numbers_column_width();
        if let Some(cw) = self.code_width {
            let total = cw + numbers_width + if self.line_numbers { 1 } else { 0 };
            return Measurement::new(numbers_width, total);
        }
        let (_, processed) = self.process_code();
        let all_lines: Vec<&str> = processed.lines().collect();
        let visible: &[&str] = if let Some((start, end)) = self.line_range {
            let offset = start.saturating_sub(1);
            let end_idx = end.min(all_lines.len());
            if offset >= all_lines.len() {
                &[]
            } else {
                &all_lines[offset..end_idx]
            }
        } else {
            &all_lines
        };
        let max_line_width = visible.iter().map(|l| cell_len(l)).max().unwrap_or(0);
        let total = numbers_width + max_line_width + if self.line_numbers { 1 } else { 0 };
        Measurement::new(numbers_width, total)
    }
}

/// Implement the Renderable trait so Syntax can be printed by Console.
impl Renderable for Syntax {
    fn gilt_console(&self, _console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        self.render_syntax(options.max_width)
    }

    fn gilt_measure(&self, _console: &Console, _options: &ConsoleOptions) -> Measurement {
        self.measure()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a syntect `Style` to a gilt `Style`.
///
/// Maps foreground color and the syntect `FontStyle` bold / italic / underline
/// bits to their gilt equivalents.  Background color from syntect themes is
/// intentionally ignored here; the Syntax renderer applies its own background
/// via `get_background_style`.
fn syntect_to_gilt_style(style: SyntectStyle) -> Style {
    use syntect::highlighting::FontStyle;

    let fg = style.foreground;
    let mut gilt = Style::from_color(Some(Color::from_rgb(fg.r, fg.g, fg.b)), None);

    let fs = style.font_style;
    if fs.contains(FontStyle::BOLD) {
        gilt.set_bold(Some(true));
    }
    if fs.contains(FontStyle::ITALIC) {
        gilt.set_italic(Some(true));
    }
    if fs.contains(FontStyle::UNDERLINE) {
        gilt.set_underline(Some(true));
    }

    gilt
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

    // -- gilt_measure override ----------------------------------------------

    #[test]
    fn syntax_gilt_measure_delegates_to_measure() {
        let code = "fn main() {}\n";
        let syntax = Syntax::new(code, "rs");
        let console = Console::builder()
            .width(80)
            .force_terminal(true)
            .no_color(true)
            .build();
        let opts = console.options();
        assert_eq!(
            syntax.gilt_measure(&console, &opts),
            syntax.measure(),
            "Syntax::gilt_measure must delegate to Syntax::measure",
        );
    }

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
        assert_eq!(
            syntax.highlight_lines,
            [1, 2, 3].iter().copied().collect::<HashSet<_>>()
        );
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
        // Read this very test file relative to CARGO_MANIFEST_DIR so the test
        // works in any working tree.
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let full_path = std::path::Path::new(manifest_dir).join(file!());
        if full_path.exists() {
            let syntax = Syntax::from_path(full_path.to_str().unwrap()).unwrap();
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
        assert_eq!(color.triplet().unwrap().red, 255);
        assert_eq!(color.triplet().unwrap().green, 128);
        assert_eq!(color.triplet().unwrap().blue, 0);
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
        syntax.padding = (1, 0, 1, 0);
        let segments = syntax.render_syntax(40);
        // With top=1, bottom=1, expect ≥3 newlines (top + code + bottom)
        let newline_count = segments.iter().filter(|s| s.text == "\n").count();
        assert!(
            newline_count >= 3,
            "expected at least 3 newlines, got {}",
            newline_count
        );
    }

    #[test]
    fn test_padding_unpack_from_shorthand() {
        // 1 → (1, 1, 1, 1)
        assert_eq!(unpack_padding(PaddingSpec::Uniform(2)), (2, 2, 2, 2));
        // (v, h) → (v, h, v, h)
        assert_eq!(unpack_padding(PaddingSpec::VertHoriz(1, 3)), (1, 3, 1, 3));
        // (t, h, b) → (t, h, b, h)
        assert_eq!(
            unpack_padding(PaddingSpec::TopHorizBottom(1, 2, 3)),
            (1, 2, 3, 2)
        );
        // (t, r, b, l) → as-is
        assert_eq!(unpack_padding(PaddingSpec::Full(1, 2, 3, 4)), (1, 2, 3, 4));
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
        assert_eq!(syntax.padding, (0, 0, 0, 0));
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
                if let Some(style) = seg.style() {
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

    // -- with_syntect_theme / load_theme_from_file -------------------------

    #[test]
    fn test_with_syntect_theme_renders() {
        use syntect::highlighting::ThemeSet;
        let ts = ThemeSet::load_defaults();
        let theme = ts.themes["base16-ocean.dark"].clone();
        let syntax = Syntax::new("let x = 1;\n", "rs").with_syntect_theme(theme);
        let segments = syntax.render_syntax(80);
        assert!(
            !segments.is_empty(),
            "with_syntect_theme produced no output"
        );
    }

    #[test]
    fn test_with_syntect_theme_background_differs() {
        use syntect::highlighting::ThemeSet;
        let ts = ThemeSet::load_defaults();

        // base16-ocean.dark and base16-mocha.dark have different backgrounds.
        let ocean = ts.themes["base16-ocean.dark"].clone();
        let mocha = ts.themes["base16-mocha.dark"].clone();

        let ocean_bg = ocean
            .settings
            .background
            .expect("ocean theme has background");
        let mocha_bg = mocha
            .settings
            .background
            .expect("mocha theme has background");

        // Sanity: the two themes actually differ in background.
        assert_ne!(
            (ocean_bg.r, ocean_bg.g, ocean_bg.b),
            (mocha_bg.r, mocha_bg.g, mocha_bg.b),
            "test pre-condition: themes must have different backgrounds"
        );

        let syntax_ocean = Syntax::new("let x = 1;\n", "rs").with_syntect_theme(ocean);
        let syntax_mocha = Syntax::new("let x = 1;\n", "rs").with_syntect_theme(mocha);

        let bg_ocean = syntax_ocean.get_background_style();
        let bg_mocha = syntax_mocha.get_background_style();

        assert_ne!(
            bg_ocean.bgcolor(),
            bg_mocha.bgcolor(),
            "injected themes should produce different backgrounds"
        );
    }

    #[test]
    fn test_with_syntect_theme_clone_is_cheap() {
        use syntect::highlighting::ThemeSet;
        let ts = ThemeSet::load_defaults();
        let theme = ts.themes["base16-ocean.dark"].clone();
        let s1 = Syntax::new("code", "rs").with_syntect_theme(theme);
        // Clone via Arc is reference-counted, not a deep copy.
        let s2 = s1.clone();
        assert!(
            s2.injected_theme.is_some(),
            "cloned Syntax should retain injected theme"
        );
    }

    #[cfg(all(feature = "syntax-theme-file", not(target_arch = "wasm32")))]
    #[test]
    fn test_load_theme_from_file_nonexistent() {
        let result = Syntax::load_theme_from_file("/nonexistent/path/theme.tmTheme");
        assert!(result.is_err());
    }
}
