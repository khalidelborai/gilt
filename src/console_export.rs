//! Export-mode methods and helper functions for [`Console`]. The helper
//! functions (`html_escape`, `build_svg_chrome`, `build_svg_text`, etc.) were
//! split out of `console.rs` in v1.2 Phase 2. The `impl Console` export
//! methods (`export_text`, `export_html`, `export_html_with_theme`,
//! `export_html_opts`, `export_svg`, `export_svg_opts`) were relocated here in
//! v1.7.1 to mirror the pattern used by `console_render.rs` and
//! `console_capture.rs`. Callers are unchanged — the methods remain on
//! `Console` at the same public paths.
use std::borrow::Cow;
use std::fmt::Write as _;

use crate::cells::cell_len;
use crate::color::blend_rgb;
use crate::console::Console;
use crate::export_format::{
    FontEmbedding, HtmlExportOptions, SvgExportOptions, CONSOLE_HTML_FORMAT, CONSOLE_SVG_FORMAT,
};
use crate::segment::Segment;
use crate::style::Style;
use crate::terminal_theme::{TerminalTheme, DEFAULT_TERMINAL_THEME, SVG_EXPORT_THEME};

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Escape HTML special characters.
pub(super) fn html_escape(s: &str) -> Cow<'_, str> {
    if !s.contains(['&', '<', '>', '"']) {
        return Cow::Borrowed(s);
    }
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    Cow::Owned(out)
}

/// Find an existing CSS class for a style, or create a new one.
pub(super) fn find_or_insert_class(
    cache: &mut Vec<(Style, String)>,
    stylesheet: &mut String,
    style: &Style,
    css: &str,
) -> String {
    for (cached_style, class_name) in cache.iter() {
        if cached_style == style {
            return class_name.clone();
        }
    }
    let mut class_name = String::new();
    write!(class_name, "r{}", cache.len() + 1).unwrap();
    writeln!(stylesheet, ".{} {{ {} }}", class_name, css).unwrap();
    cache.push((style.clone(), class_name.clone()));
    class_name
}

/// Build the SVG chrome (window title bar and decorations).
pub(super) fn build_svg_chrome(
    width: f64,
    height: f64,
    theme: &TerminalTheme,
    title: &str,
    unique_id: &str,
) -> String {
    let bg = theme.background_color.hex();
    let mut chrome = String::new();

    // Background rectangle with rounded corners
    writeln!(
        chrome,
        "<rect fill=\"{}\" stroke=\"rgba(255,255,255,0.35)\" stroke-width=\"1\" \
         x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" rx=\"8\"/>",
        bg, width, height,
    )
    .unwrap();

    // Window control dots
    let dot_colors = ["#ff5f57", "#febc2e", "#28c840"];
    for (i, color) in dot_colors.iter().enumerate() {
        let cx = 16.0 + (i as f64) * 22.0;
        writeln!(
            chrome,
            "    <circle cx=\"{:.0}\" cy=\"18\" r=\"5\" fill=\"{}\"/>",
            cx, color
        )
        .unwrap();
    }

    // Title text
    if !title.is_empty() {
        writeln!(
            chrome,
            "    <text class=\"{}-title\" fill=\"{}\" x=\"{}\" y=\"23\" \
             text-anchor=\"middle\">{}</text>",
            unique_id,
            theme.foreground_color.hex(),
            width / 2.0,
            svg_escape(title),
        )
        .unwrap();
    }

    chrome
}

/// Build the SVG text content from segments, cropping each line to `width` cells.
#[allow(clippy::too_many_arguments)]
pub(super) fn build_svg_text(
    buffer: &[Segment],
    width: usize,
    theme: &TerminalTheme,
    unique_id: &str,
    char_width: f64,
    line_height: f64,
    padding_top: f64,
    padding_left: f64,
) -> (String, String, String, String) {
    let mut matrix = String::new();
    let mut backgrounds = String::new();
    let mut styles = String::new();
    let lines_defs = String::new();

    let mut style_cache: Vec<(String, String)> = Vec::new();
    let mut y = padding_top + line_height;
    let mut x: f64;

    // Split and crop each line to the console width (rich parity: split_and_crop_lines).
    let line_segments = Segment::split_and_crop_lines(buffer, width, None, false, false);

    for line in &line_segments {
        x = padding_left;
        for seg in line {
            if seg.is_control() {
                continue;
            }
            let text = &seg.text;
            let escaped = svg_escape(text);
            let text_width = cell_len(text) as f64 * char_width;

            if let Some(ref style) = seg.style {
                // Compute effective fg/bg color references, applying `reverse` swap.
                // `reverse` swaps fg↔bg visually: the foreground color fills the
                // background rect, and the background color fills the text glyph.
                let (eff_fg_color, eff_bg_color) = if style.reverse() == Some(true) {
                    (style.bgcolor(), style.color())
                } else {
                    (style.color(), style.bgcolor())
                };

                // Background rect — uses the *effective* background color so that
                // `reverse` correctly paints the original fg as the rect fill.
                if let Some(bgcolor) = eff_bg_color {
                    let bg_triplet = bgcolor.get_truecolor(Some(theme), false);
                    writeln!(
                        backgrounds,
                        "    <rect fill=\"{}\" x=\"{:.1}\" y=\"{:.1}\" \
                         width=\"{:.1}\" height=\"{:.1}\"/>",
                        bg_triplet.hex(),
                        x,
                        y - line_height + 3.0,
                        text_width,
                        line_height,
                    )
                    .unwrap();
                }

                // When `dim` is set and the segment has no bgcolor (eff_bg_color is None),
                // rich blends the foreground toward the theme background at cross-fade 0.4.
                // `get_html_style` only blends when both fg and bg are present, so we handle
                // this SVG-only case here and emit a direct `fill=` rather than a CSS class.
                let dim_no_bg_override: Option<String> =
                    if style.dim() == Some(true) && eff_bg_color.is_none() {
                        if let Some(fg_color) = eff_fg_color {
                            let fg_triplet = fg_color.get_truecolor(Some(theme), true);
                            let blended = blend_rgb(fg_triplet, theme.background_color, 0.4);
                            Some(blended.hex())
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                // Foreground text with style class (or direct fill for dim-no-bg override).
                if let Some(ref blended_hex) = dim_no_bg_override {
                    writeln!(
                        matrix,
                        "    <text fill=\"{}\" x=\"{:.1}\" y=\"{:.1}\" \
                         textLength=\"{:.1}\">{}</text>",
                        blended_hex, x, y, text_width, escaped
                    )
                    .unwrap();
                } else {
                    let css = style.get_html_style(Some(theme));
                    if !css.is_empty() {
                        let class_name = find_or_insert_svg_class(
                            &mut style_cache,
                            &mut styles,
                            unique_id,
                            &css,
                        );
                        writeln!(
                            matrix,
                            "    <text class=\"{}\" x=\"{:.1}\" y=\"{:.1}\" \
                             textLength=\"{:.1}\">{}</text>",
                            class_name, x, y, text_width, escaped
                        )
                        .unwrap();
                    } else {
                        writeln!(
                            matrix,
                            "    <text fill=\"{}\" x=\"{:.1}\" y=\"{:.1}\" \
                             textLength=\"{:.1}\">{}</text>",
                            theme.foreground_color.hex(),
                            x,
                            y,
                            text_width,
                            escaped
                        )
                        .unwrap();
                    }
                }
            } else {
                writeln!(
                    matrix,
                    "    <text fill=\"{}\" x=\"{:.1}\" y=\"{:.1}\" \
                     textLength=\"{:.1}\">{}</text>",
                    theme.foreground_color.hex(),
                    x,
                    y,
                    text_width,
                    escaped
                )
                .unwrap();
            }

            x += text_width;
        }
        y += line_height;
    }

    (matrix, backgrounds, styles, lines_defs)
}

/// Find or create an SVG style class.
pub(super) fn find_or_insert_svg_class(
    cache: &mut Vec<(String, String)>,
    styles: &mut String,
    unique_id: &str,
    css: &str,
) -> String {
    for (cached_css, class_name) in cache.iter() {
        if cached_css == css {
            return class_name.clone();
        }
    }
    let mut class_name = String::new();
    write!(class_name, "{}-s{}", unique_id, cache.len() + 1).unwrap();
    // Convert HTML CSS to SVG attributes
    let svg_style = css_to_svg_style(css);
    writeln!(styles, "    .{} {{ {} }}", class_name, svg_style).unwrap();
    cache.push((css.to_string(), class_name.clone()));
    class_name
}

/// Convert CSS style properties to SVG-compatible style properties.
pub(super) fn css_to_svg_style(css: &str) -> String {
    let mut result = String::new();
    for part in css.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((key, value)) = part.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            let svg_key = match key {
                "color" => Some("fill"),
                "font-weight" => Some("font-weight"),
                "font-style" => Some("font-style"),
                "text-decoration" => Some("text-decoration"),
                _ => None, // Skip background-color and other non-SVG properties
            };
            if let Some(svg_key) = svg_key {
                if !result.is_empty() {
                    result.push_str("; ");
                }
                write!(result, "{}: {}", svg_key, value).unwrap();
            }
        }
    }
    result
}

/// Escape text for SVG content.
pub(super) fn svg_escape(s: &str) -> Cow<'_, str> {
    if !s.contains(['&', '<', '>', '"', '\'']) {
        return Cow::Borrowed(s);
    }
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    Cow::Owned(out)
}

// ---------------------------------------------------------------------------
// Export methods on Console (relocated from console.rs in v1.7.1)
// ---------------------------------------------------------------------------

impl Console {
    // -- Export (record mode) -----------------------------------------------

    /// Export recorded output as plain or styled text.
    ///
    /// Only works if `record` was enabled when the Console was created.
    /// Pass `clear = true` to empty the record buffer after export.
    /// Pass `styles = true` to include ANSI escape codes in the output.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// let mut console = Console::builder()
    ///     .width(80)
    ///     .no_color(true)
    ///     .record(true)
    ///     .markup(false)
    ///     .build();
    /// let text = Text::new("Export me", Style::null());
    /// console.print(&text);
    /// let exported = console.export_text(false, false);
    /// assert!(exported.contains("Export me"));
    /// ```
    pub fn export_text(&mut self, clear: bool, styles: bool) -> String {
        assert!(
            self.record,
            "export requires record mode — build the Console with .record(true)"
        );
        let buffer = self.record_buffer.clone();
        if clear {
            self.record_buffer.clear();
        }

        if styles {
            self.render_buffer(&buffer)
        } else {
            // Strip control segments and just concatenate text
            let mut output = String::new();
            for segment in &buffer {
                if !segment.is_control() {
                    output.push_str(&segment.text);
                }
            }
            output
        }
    }

    /// Export recorded output as an HTML document.
    ///
    /// Generates a complete HTML page with inline or class-based styles.
    /// Requires `record` mode to be enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// let mut console = Console::builder()
    ///     .width(80)
    ///     .record(true)
    ///     .markup(false)
    ///     .build();
    /// let text = Text::styled("Red text", "red");
    /// console.print(&text);
    /// let html = console.export_html(None, false, true);
    /// assert!(html.contains("<!DOCTYPE html>"));
    /// assert!(html.contains("Red text"));
    /// ```
    pub fn export_html(
        &mut self,
        theme: Option<&TerminalTheme>,
        clear: bool,
        inline_styles: bool,
    ) -> String {
        assert!(
            self.record,
            "export requires record mode — build the Console with .record(true)"
        );
        let theme = theme.unwrap_or(&DEFAULT_TERMINAL_THEME);
        // Finding #9: iterate by reference; only copy out when clear is needed.
        let buffer_ref: &[Segment];
        let taken: Vec<Segment>;
        if clear {
            taken = std::mem::take(&mut self.record_buffer);
            buffer_ref = &taken;
        } else {
            buffer_ref = &self.record_buffer;
        }

        // Finding #8: merge adjacent same-style segments before HTML iteration.
        let simplified = Segment::simplify(buffer_ref);

        let mut code = String::new();
        let mut stylesheet = String::new();
        let mut style_cache: Vec<(Style, String)> = Vec::new();

        for segment in &simplified {
            if segment.is_control() {
                continue;
            }
            let escaped = html_escape(&segment.text);

            if let Some(style) = segment.style() {
                // Finding #7: wrap the text in <a href> when the style has a link.
                let link_url = style.link().map(|s| s.to_string());

                if style.is_null() && link_url.is_none() {
                    code.push_str(&escaped);
                    continue;
                }

                let css = style.get_html_style(Some(theme));
                let inner: String;
                if css.is_empty() {
                    inner = escaped.into_owned();
                } else if inline_styles {
                    inner = format!("<span style=\"{}\">{}</span>", css, escaped);
                } else {
                    // Use class-based styles
                    let class_name =
                        find_or_insert_class(&mut style_cache, &mut stylesheet, style, &css);
                    inner = format!("<span class=\"{}\">{}</span>", class_name, escaped);
                }

                // Wrap in <a href> if there is a link (finding #7).
                if let Some(url) = link_url {
                    write!(code, "<a href=\"{}\">{}</a>", html_escape(&url), inner).unwrap();
                } else {
                    code.push_str(&inner);
                }
            } else {
                code.push_str(&escaped);
            }
        }

        let fg = theme.foreground_color.hex();
        let bg = theme.background_color.hex();

        CONSOLE_HTML_FORMAT
            .replace("{stylesheet}", &stylesheet)
            .replace("{foreground}", &fg)
            .replace("{background}", &bg)
            .replace("{code}", &code)
    }

    /// Export recorded output as HTML using a named palette from [`ThemeRegistry`].
    ///
    /// Convenience wrapper around [`export_html`](Self::export_html): looks up
    /// `theme_name` in `ThemeRegistry` and passes the result.  Equivalent to:
    ///
    /// ```rust,ignore
    /// let theme = ThemeRegistry::terminal_theme("dracula").unwrap();
    /// console.export_html(Some(theme), clear, inline_styles);
    /// ```
    ///
    /// Returns the same HTML as `export_html` with `theme = None` (the default
    /// terminal theme) when `theme_name` is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// let mut console = Console::builder()
    ///     .width(80)
    ///     .record(true)
    ///     .force_terminal(true)
    ///     .markup(false)
    ///     .build();
    /// let text = Text::styled("Dracula", "bold magenta");
    /// console.print(&text);
    /// let html = console.export_html_with_theme("dracula", false, true);
    /// assert!(html.contains("<!DOCTYPE html>"));
    /// // Background should come from the Dracula palette (#282a36)
    /// assert!(html.contains("#282a36") || html.contains("1a2a36") || html.len() > 100);
    /// ```
    pub fn export_html_with_theme(
        &mut self,
        theme_name: &str,
        clear: bool,
        inline_styles: bool,
    ) -> String {
        use crate::terminal_theme::ThemeRegistry;
        let theme = ThemeRegistry::terminal_theme(theme_name);
        self.export_html(theme, clear, inline_styles)
    }

    /// Export recorded output as an HTML document with full control via
    /// [`HtmlExportOptions`].
    ///
    /// This is the options-based API; [`export_html`](Self::export_html) and
    /// [`export_html_with_theme`](Self::export_html_with_theme) delegate to
    /// this method via the default theme.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    /// use gilt::export_format::HtmlExportOptions;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// let mut console = Console::builder()
    ///     .width(80)
    ///     .record(true)
    ///     .markup(false)
    ///     .build();
    /// console.print(&Text::styled("hello", "bold green"));
    /// let opts = HtmlExportOptions::default()
    ///     .inline_styles(true)
    ///     .dark_mode(true)
    ///     .copy_button(true);
    /// let html = console.export_html_opts(None, &opts);
    /// assert!(html.contains("<!DOCTYPE html>"));
    /// assert!(html.contains("hello"));
    /// ```
    pub fn export_html_opts(
        &mut self,
        theme: Option<&TerminalTheme>,
        opts: &HtmlExportOptions,
    ) -> String {
        assert!(
            self.record,
            "export requires record mode — build the Console with .record(true)"
        );
        let theme = theme.unwrap_or(&DEFAULT_TERMINAL_THEME);

        // Optionally clear (same logic as export_html)
        let buffer_ref: &[Segment];
        let taken: Vec<Segment>;
        if opts.clear {
            taken = std::mem::take(&mut self.record_buffer);
            buffer_ref = &taken;
        } else {
            buffer_ref = &self.record_buffer;
        }

        let simplified = Segment::simplify(buffer_ref);

        let mut code = String::new();
        let mut stylesheet = String::new();
        let mut style_cache: Vec<(Style, String)> = Vec::new();

        for segment in &simplified {
            if segment.is_control() {
                continue;
            }
            let escaped = html_escape(&segment.text);

            if let Some(style) = segment.style() {
                let link_url = style.link().map(|s| s.to_string());

                if style.is_null() && link_url.is_none() {
                    code.push_str(&escaped);
                    continue;
                }

                let css = style.get_html_style(Some(theme));
                let inner: String;
                if css.is_empty() {
                    inner = escaped.into_owned();
                } else if opts.inline_styles {
                    inner = format!("<span style=\"{}\">{}</span>", css, escaped);
                } else {
                    let class_name =
                        find_or_insert_class(&mut style_cache, &mut stylesheet, style, &css);
                    inner = format!("<span class=\"{}\">{}</span>", class_name, escaped);
                }

                if let Some(url) = link_url {
                    write!(code, "<a href=\"{}\">{}</a>", html_escape(&url), inner).unwrap();
                } else {
                    code.push_str(&inner);
                }
            } else {
                code.push_str(&escaped);
            }
        }

        let fg = theme.foreground_color.hex();
        let bg = theme.background_color.hex();

        // Font-family override
        let font_family = opts
            .font_family
            .as_deref()
            .unwrap_or("Menlo,'DejaVu Sans Mono',consolas,'Courier New',monospace");

        // Build @font-face if font_url is provided
        let font_face = if let Some(ref url) = opts.font_url {
            format!(
                "@font-face {{ font-family: '{}'; src: url('{}'); }}\n",
                font_family, url
            )
        } else {
            String::new()
        };

        // Dark-mode CSS block
        let dark_css = if opts.dark_mode {
            format!(
                "\n@media (prefers-color-scheme: dark) {{\n  body {{ color: {}; background-color: {}; }}\n}}\n",
                bg, fg
            )
        } else {
            String::new()
        };

        // Full stylesheet
        let full_stylesheet = format!("{}{}{}", font_face, stylesheet, dark_css);

        // Copy-button HTML + JS
        let copy_snippet = if opts.copy_button {
            r#"<button id="gilt-copy-btn" onclick="(function(){var p=document.querySelector('pre');if(p){navigator.clipboard&&navigator.clipboard.writeText(p.innerText)||window.prompt('Copy:',p.innerText)}})()">Copy</button>
<script>document.getElementById('gilt-copy-btn').style.cssText='position:absolute;top:8px;right:8px;padding:2px 8px;cursor:pointer';</script>
"#
        } else {
            ""
        };

        // Choose template
        let template = opts.code_format.as_deref().unwrap_or(CONSOLE_HTML_FORMAT);

        // Inject copy button before </body>
        let html_base = template
            .replace("{stylesheet}", &full_stylesheet)
            .replace("{foreground}", &fg)
            .replace("{background}", &bg)
            .replace("{code}", &code);

        if opts.copy_button {
            html_base.replace("</body>", &format!("{}</body>", copy_snippet))
        } else {
            html_base
        }
    }

    /// Export recorded output as an SVG document.
    ///
    /// Generates a complete SVG image with terminal-style chrome (title bar,
    /// window controls) and styled text content. Requires `record` mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// let mut console = Console::builder()
    ///     .width(40)
    ///     .record(true)
    ///     .no_color(true)
    ///     .markup(false)
    ///     .build();
    /// let text = Text::new("SVG test", Style::null());
    /// console.print(&text);
    /// let svg = console.export_svg("Test", None, false, None, 0.61);
    /// assert!(svg.contains("<svg"));
    /// assert!(svg.contains("SVG test"));
    /// ```
    pub fn export_svg(
        &mut self,
        title: &str,
        theme: Option<&TerminalTheme>,
        clear: bool,
        unique_id: Option<&str>,
        font_aspect_ratio: f64,
    ) -> String {
        assert!(
            self.record,
            "export requires record mode — build the Console with .record(true)"
        );
        let theme = theme.unwrap_or(&SVG_EXPORT_THEME);

        // Finding #9: avoid cloning the whole buffer.
        let buffer_ref: &[Segment];
        let taken: Vec<Segment>;
        if clear {
            taken = std::mem::take(&mut self.record_buffer);
            buffer_ref = &taken;
        } else {
            buffer_ref = &self.record_buffer;
        }

        // Finding #11: derive a unique id from FNV-1a hash of segment text + title
        // when the caller leaves unique_id as None.
        let derived_id: String;
        let unique_id: &str = if let Some(id) = unique_id {
            id
        } else {
            use crate::utils::hash::{fnv1a_64_extend, FNV_OFFSET};
            let mut hash = FNV_OFFSET;
            for seg in buffer_ref {
                hash = fnv1a_64_extend(hash, seg.text.as_bytes());
            }
            hash = fnv1a_64_extend(hash, title.as_bytes());
            derived_id = format!("gilt-{:016x}", hash);
            &derived_id
        };

        // Split and crop to get accurate line count for height calculation.
        let text_lines =
            Segment::split_and_crop_lines(buffer_ref, self.width(), None, false, false);

        let char_height = 20.0_f64;
        let line_height = char_height * 1.22;
        let char_width = char_height * font_aspect_ratio;
        let margin_top = 1.0;
        let margin_right = 1.0;
        let margin_bottom = 1.0;
        let margin_left = 1.0;
        let padding_top = 40.0;
        let padding_right = 8.0;
        let padding_bottom = 8.0;
        let padding_left = 8.0;

        let console_width = self.width() as f64;
        let line_count = text_lines.len().max(1) as f64;

        let terminal_width = (console_width * char_width + padding_left + padding_right).ceil();
        let terminal_height = (line_count * line_height + padding_top + padding_bottom).ceil();
        let svg_width = (terminal_width + margin_left + margin_right).ceil();
        let svg_height = (terminal_height + margin_top + margin_bottom).ceil();

        let terminal_x = margin_left;
        let terminal_y = margin_top;

        // Build the chrome (window decorations)
        let chrome = build_svg_chrome(terminal_width, terminal_height, theme, title, unique_id);

        let (matrix, backgrounds, styles, lines_defs) = build_svg_text(
            buffer_ref,
            self.width(),
            theme,
            unique_id,
            char_width,
            line_height,
            padding_top,
            padding_left,
        );

        // Pre-format numeric values into a shared buffer to avoid per-replace allocations.
        let mut buf = String::with_capacity(16);
        macro_rules! fmt_buf {
            ($fmt:literal, $val:expr) => {{
                buf.clear();
                write!(buf, $fmt, $val).unwrap();
                &buf
            }};
        }

        // Apply replacements that use the shared buffer one at a time,
        // cloning the formatted value so `buf` can be reused.
        let mut svg = CONSOLE_SVG_FORMAT.replace("{unique_id}", unique_id);
        svg = svg.replace("{char_height}", fmt_buf!("{:.1}", char_height));
        svg = svg.replace("{line_height}", fmt_buf!("{:.1}", line_height));
        svg = svg.replace("{width}", fmt_buf!("{:.0}", svg_width));
        svg = svg.replace("{height}", fmt_buf!("{:.0}", svg_height));
        svg = svg.replace("{terminal_width}", fmt_buf!("{:.0}", terminal_width));
        svg = svg.replace("{terminal_height}", fmt_buf!("{:.0}", terminal_height));
        svg = svg.replace("{terminal_x}", fmt_buf!("{:.0}", terminal_x));
        svg = svg.replace("{terminal_y}", fmt_buf!("{:.0}", terminal_y));
        svg = svg.replace("{chrome}", &chrome);
        svg = svg.replace("{matrix}", &matrix);
        svg = svg.replace("{backgrounds}", &backgrounds);
        svg = svg.replace("{styles}", &styles);
        svg = svg.replace("{lines}", &lines_defs);
        svg
    }

    /// Export recorded output as an SVG document with full control via
    /// [`SvgExportOptions`].
    ///
    /// The key addition over [`export_svg`](Self::export_svg) is
    /// [`FontEmbedding::Base64`], which embeds raw font bytes as a base64
    /// `data:` URL so the SVG is completely self-contained offline.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    /// use gilt::export_format::{FontEmbedding, SvgExportOptions};
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// let mut console = Console::builder()
    ///     .width(40)
    ///     .record(true)
    ///     .no_color(true)
    ///     .markup(false)
    ///     .build();
    /// console.print(&Text::new("SVG opts", Style::null()));
    ///
    /// // Embed a tiny fake font for offline use
    /// let opts = SvgExportOptions::default()
    ///     .title("Demo")
    ///     .font_embedding(FontEmbedding::Base64(b"FAKE_FONT".to_vec()));
    /// let svg = console.export_svg_opts(None, &opts);
    /// assert!(svg.contains("<svg"));
    /// assert!(svg.contains("data:font/"));
    /// ```
    pub fn export_svg_opts(
        &mut self,
        theme: Option<&TerminalTheme>,
        opts: &SvgExportOptions,
    ) -> String {
        assert!(
            self.record,
            "export requires record mode — build the Console with .record(true)"
        );
        use crate::utils::control::base64_encode;

        let theme = theme.unwrap_or(&SVG_EXPORT_THEME);

        let buffer_ref: &[Segment];
        let taken: Vec<Segment>;
        if opts.clear {
            taken = std::mem::take(&mut self.record_buffer);
            buffer_ref = &taken;
        } else {
            buffer_ref = &self.record_buffer;
        }

        // Derive unique_id
        let derived_id: String;
        let unique_id: &str = if let Some(ref id) = opts.unique_id {
            id.as_str()
        } else {
            use crate::utils::hash::{fnv1a_64_extend, FNV_OFFSET};
            let mut hash = FNV_OFFSET;
            for seg in buffer_ref {
                hash = fnv1a_64_extend(hash, seg.text.as_bytes());
            }
            hash = fnv1a_64_extend(hash, opts.title.as_bytes());
            derived_id = format!("gilt-{:016x}", hash);
            &derived_id
        };

        // Split and crop to get accurate line count for height calculation.
        let text_lines =
            Segment::split_and_crop_lines(buffer_ref, self.width(), None, false, false);

        let char_height = 20.0_f64;
        let line_height = char_height * 1.22;
        let char_width = char_height * opts.font_aspect_ratio;
        let margin_top = 1.0;
        let margin_right = 1.0;
        let margin_bottom = 1.0;
        let margin_left = 1.0;
        let padding_top = 40.0;
        let padding_right = 8.0;
        let padding_bottom = 8.0;
        let padding_left = 8.0;

        let console_width = self.width() as f64;
        let line_count = text_lines.len().max(1) as f64;

        let terminal_width = (console_width * char_width + padding_left + padding_right).ceil();
        let terminal_height = (line_count * line_height + padding_top + padding_bottom).ceil();
        let svg_width = (terminal_width + margin_left + margin_right).ceil();
        let svg_height = (terminal_height + margin_top + margin_bottom).ceil();
        let terminal_x = margin_left;
        let terminal_y = margin_top;

        let chrome = build_svg_chrome(
            terminal_width,
            terminal_height,
            theme,
            &opts.title,
            unique_id,
        );

        let (matrix, backgrounds, styles, lines_defs) = build_svg_text(
            buffer_ref,
            self.width(),
            theme,
            unique_id,
            char_width,
            line_height,
            padding_top,
            padding_left,
        );

        // Build base SVG from standard template
        let mut buf = String::with_capacity(16);
        macro_rules! fmt_buf {
            ($fmt:literal, $val:expr) => {{
                buf.clear();
                write!(buf, $fmt, $val).unwrap();
                &buf
            }};
        }

        let mut svg = CONSOLE_SVG_FORMAT.replace("{unique_id}", unique_id);
        svg = svg.replace("{char_height}", fmt_buf!("{:.1}", char_height));
        svg = svg.replace("{line_height}", fmt_buf!("{:.1}", line_height));
        svg = svg.replace("{width}", fmt_buf!("{:.0}", svg_width));
        svg = svg.replace("{height}", fmt_buf!("{:.0}", svg_height));
        svg = svg.replace("{terminal_width}", fmt_buf!("{:.0}", terminal_width));
        svg = svg.replace("{terminal_height}", fmt_buf!("{:.0}", terminal_height));
        svg = svg.replace("{terminal_x}", fmt_buf!("{:.0}", terminal_x));
        svg = svg.replace("{terminal_y}", fmt_buf!("{:.0}", terminal_y));
        svg = svg.replace("{chrome}", &chrome);
        svg = svg.replace("{matrix}", &matrix);
        svg = svg.replace("{backgrounds}", &backgrounds);
        svg = svg.replace("{styles}", &styles);
        svg = svg.replace("{lines}", &lines_defs);

        // Task 2: inject @font-face with base64 data: URL when FontEmbedding::Base64
        if let FontEmbedding::Base64(ref font_bytes) = opts.font_embedding {
            let encoded = base64_encode(font_bytes);
            let font_face = format!(
                "@font-face {{\n    font-family: \"Fira Code\";\n    src: url(\"data:font/woff2;base64,{}\") format(\"woff2\");\n    font-style: normal;\n    font-weight: 400;\n}}\n",
                encoded
            );
            // Replace the existing @font-face blocks (everything from the first
            // @font-face up to the first `.{unique_id}-matrix` class).
            // Simpler: just prepend the embedded rule inside the <style> tag.
            svg = svg.replacen("<style>", &format!("<style>\n{}", font_face), 1);
        }

        svg
    }
}
