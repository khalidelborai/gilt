//! Export format templates for Console HTML and SVG output.

// ---------------------------------------------------------------------------
// HtmlExportOptions (Task 1)
// ---------------------------------------------------------------------------

/// Options for [`Console::export_html_opts`].
///
/// Build via the [`Default`] impl and the builder methods:
///
/// ```
/// use gilt::export_format::HtmlExportOptions;
///
/// let opts = HtmlExportOptions::default()
///     .inline_styles(true)
///     .dark_mode(true)
///     .copy_button(true)
///     .font_family("JetBrains Mono");
/// ```
#[derive(Debug, Clone, Default)]
pub struct HtmlExportOptions {
    /// When `true`, emit inline `style=""` attributes instead of CSS classes.
    pub inline_styles: bool,
    /// When `true`, clear the record buffer after export.
    pub clear: bool,
    /// Optional font-family CSS name (overrides the default `Menlo, …, monospace`).
    pub font_family: Option<String>,
    /// Optional URL from which the font should be loaded (`@import` / `<link>`).
    pub font_url: Option<String>,
    /// When `true`, inject a `@media (prefers-color-scheme: dark)` block that
    /// inverts the terminal palette.
    pub dark_mode: bool,
    /// When `true`, inject a small "Copy" button and the minimal JS to copy the
    /// `<pre>` block text to the clipboard.
    pub copy_button: bool,
    /// Optional full HTML template override.  Use `None` to keep the default
    /// [`CONSOLE_HTML_FORMAT`] template.  Placeholders must include at minimum
    /// `{stylesheet}`, `{foreground}`, `{background}`, and `{code}`.
    pub code_format: Option<String>,
}

impl HtmlExportOptions {
    /// Set whether to use inline `style=""` attributes instead of CSS classes.
    #[must_use]
    pub fn inline_styles(mut self, v: bool) -> Self {
        self.inline_styles = v;
        self
    }

    /// Set whether to clear the record buffer after export.
    #[must_use]
    pub fn clear(mut self, v: bool) -> Self {
        self.clear = v;
        self
    }

    /// Set an optional font-family override (CSS font-family value).
    #[must_use]
    pub fn font_family(mut self, v: impl Into<String>) -> Self {
        self.font_family = Some(v.into());
        self
    }

    /// Set an optional URL from which the font should be loaded.
    #[must_use]
    pub fn font_url(mut self, v: impl Into<String>) -> Self {
        self.font_url = Some(v.into());
        self
    }

    /// When `true`, inject a dark-mode CSS block.
    #[must_use]
    pub fn dark_mode(mut self, v: bool) -> Self {
        self.dark_mode = v;
        self
    }

    /// When `true`, inject a "Copy" button and clipboard JS.
    #[must_use]
    pub fn copy_button(mut self, v: bool) -> Self {
        self.copy_button = v;
        self
    }

    /// Override the full HTML template.
    #[must_use]
    pub fn code_format(mut self, v: impl Into<String>) -> Self {
        self.code_format = Some(v.into());
        self
    }
}

// ---------------------------------------------------------------------------
// SvgExportOptions (Task 2)
// ---------------------------------------------------------------------------

/// Font embedding mode for SVG export.
///
/// ```
/// use gilt::export_format::FontEmbedding;
///
/// // No embedding — reference font by name
/// let none = FontEmbedding::None;
///
/// // Embed raw font bytes as a base64 data: URL
/// let bytes: Vec<u8> = b"FONT_DATA".to_vec();
/// let embedded = FontEmbedding::Base64(bytes);
/// ```
#[derive(Debug, Clone)]
pub enum FontEmbedding {
    /// Reference the font family by name only (default).  The SVG relies on
    /// the viewer having the font installed or the default CDN `@font-face`
    /// rules in the template.
    None,
    /// Embed the provided raw font bytes as a base64 `data:` URL inside a
    /// `@font-face` rule, making the SVG fully self-contained.
    Base64(Vec<u8>),
}

/// Options for [`Console::export_svg_opts`].
#[derive(Debug, Clone)]
pub struct SvgExportOptions {
    /// Title shown in the SVG chrome (window title bar).
    pub title: String,
    /// Font embedding mode.
    pub font_embedding: FontEmbedding,
    /// Whether to clear the record buffer after export.
    pub clear: bool,
    /// Optional unique ID for CSS class namespacing (derived from content hash when `None`).
    pub unique_id: Option<String>,
    /// Font aspect ratio (char_width = char_height * aspect_ratio).
    pub font_aspect_ratio: f64,
}

impl Default for SvgExportOptions {
    fn default() -> Self {
        SvgExportOptions {
            title: String::new(),
            font_embedding: FontEmbedding::None,
            clear: false,
            unique_id: None,
            font_aspect_ratio: 0.61,
        }
    }
}

impl SvgExportOptions {
    /// Set the SVG title.
    #[must_use]
    pub fn title(mut self, v: impl Into<String>) -> Self {
        self.title = v.into();
        self
    }

    /// Set the font embedding mode.
    #[must_use]
    pub fn font_embedding(mut self, v: FontEmbedding) -> Self {
        self.font_embedding = v;
        self
    }

    /// Set whether to clear the record buffer after export.
    #[must_use]
    pub fn clear(mut self, v: bool) -> Self {
        self.clear = v;
        self
    }

    /// Set the unique ID for CSS namespacing.
    #[must_use]
    pub fn unique_id(mut self, v: impl Into<String>) -> Self {
        self.unique_id = Some(v.into());
        self
    }

    /// Set the font aspect ratio.
    #[must_use]
    pub fn font_aspect_ratio(mut self, v: f64) -> Self {
        self.font_aspect_ratio = v;
        self
    }
}

/// HTML template for console export.
/// Placeholders: {stylesheet}, {foreground}, {background}, {code}
pub const CONSOLE_HTML_FORMAT: &str = r##"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<style>
{stylesheet}
body {
    color: {foreground};
    background-color: {background};
}
</style>
</head>
<body>
    <pre style="font-family:Menlo,'DejaVu Sans Mono',consolas,'Courier New',monospace"><code style="font-family:inherit">{code}</code></pre>
</body>
</html>
"##;

/// SVG template for console export.
/// See the _export_format.py for full variable list.
pub const CONSOLE_SVG_FORMAT: &str = r##"<svg class="gilt-terminal" viewBox="0 0 {width} {height}" xmlns="http://www.w3.org/2000/svg">
    <!-- Generated with gilt https://github.com/gilt-rs -->
    <style>

    @font-face {
        font-family: "Fira Code";
        src: local("FiraCode-Regular"),
                url("https://cdnjs.cloudflare.com/ajax/libs/firacode/6.2.0/woff2/FiraCode-Regular.woff2") format("woff2"),
                url("https://cdnjs.cloudflare.com/ajax/libs/firacode/6.2.0/woff/FiraCode-Regular.woff") format("woff");
        font-style: normal;
        font-weight: 400;
    }
    @font-face {
        font-family: "Fira Code";
        src: local("FiraCode-Bold"),
                url("https://cdnjs.cloudflare.com/ajax/libs/firacode/6.2.0/woff2/FiraCode-Bold.woff2") format("woff2"),
                url("https://cdnjs.cloudflare.com/ajax/libs/firacode/6.2.0/woff/FiraCode-Bold.woff") format("woff");
        font-style: bold;
        font-weight: 700;
    }

    .{unique_id}-matrix {
        font-family: Fira Code, monospace;
        font-size: {char_height}px;
        line-height: {line_height}px;
        font-variant-east-asian: full-width;
    }

    .{unique_id}-title {
        font-size: 18px;
        font-weight: bold;
        font-family: arial;
    }

    {styles}
    </style>

    <defs>
    <clipPath id="{unique_id}-clip-terminal">
      <rect x="0" y="0" width="{terminal_width}" height="{terminal_height}" />
    </clipPath>
    {lines}
    </defs>

    {chrome}
    <g transform="translate({terminal_x}, {terminal_y})" clip-path="url(#{unique_id}-clip-terminal)">
    {backgrounds}
    <g class="{unique_id}-matrix">
    {matrix}
    </g>
    </g>
</svg>
"##;
