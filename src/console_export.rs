/// Export-related helper functions used by Console::export_html and Console::export_svg.
/// Split out of console.rs in v1.2 Phase 2; pub(super) so console.rs methods can call them.
use std::borrow::Cow;
use std::fmt::Write as _;

use crate::cells::cell_len;
use crate::segment::Segment;
use crate::style::Style;
use crate::terminal_theme::TerminalTheme;

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

/// Build the SVG text content from segments.
pub(super) fn build_svg_text(
    buffer: &[Segment],
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
    let mut line_segments: Vec<Vec<(String, Option<Style>)>> = Vec::new();

    // Split buffer into lines
    let mut current_line: Vec<(String, Option<Style>)> = Vec::new();
    for seg in buffer {
        if seg.is_control() {
            continue;
        }
        let parts: Vec<&str> = seg.text.split('\n').collect();
        for (i, part) in parts.iter().enumerate() {
            if !part.is_empty() {
                current_line.push((part.to_string(), seg.style().cloned()));
            }
            if i + 1 < parts.len() {
                line_segments.push(std::mem::take(&mut current_line));
            }
        }
    }
    if !current_line.is_empty() {
        line_segments.push(current_line);
    }

    for line in &line_segments {
        x = padding_left;
        for (text, style) in line {
            let escaped = svg_escape(text);
            let text_width = cell_len(text) as f64 * char_width;

            if let Some(ref style) = style {
                // Background
                if let Some(bgcolor) = style.bgcolor() {
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

                // Foreground text with style class
                let css = style.get_html_style(Some(theme));
                if !css.is_empty() {
                    let class_name =
                        find_or_insert_svg_class(&mut style_cache, &mut styles, unique_id, &css);
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
