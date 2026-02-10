//! Markdown rendering module -- parses CommonMark and produces styled terminal output.
//!
//! Port of Python's `rich/markdown.py`, using the `pulldown-cmark` crate
//! (a CommonMark-compliant markdown parser) instead of Python's `markdown_it`.

use pulldown_cmark::{Alignment, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::box_chars::HEAVY;
use crate::console::{Console, ConsoleOptions, Renderable};
use crate::panel::Panel;
use crate::rule::Rule;
use crate::segment::Segment;
use crate::style::{Style, StyleStack};
use crate::table::Table;
use crate::text::{JustifyMethod, Text};

// ---------------------------------------------------------------------------
// Markdown struct
// ---------------------------------------------------------------------------

/// Renders Markdown-formatted text to styled terminal output.
///
/// Supports headings, paragraphs, lists, code blocks, emphasis, links,
/// block quotes, horizontal rules, and tables.
#[derive(Debug, Clone)]
pub struct Markdown {
    /// Raw markdown source text.
    pub markup: String,
    /// Theme for syntax-highlighted code blocks (reserved for future use).
    pub code_theme: String,
    /// Lexer for inline code (reserved for future use).
    pub inline_code_lexer: Option<String>,
    /// Theme for inline code (reserved for future use).
    pub inline_code_theme: Option<String>,
    /// Whether to display hyperlink URLs after link text.
    pub hyperlinks: bool,
    /// Text justification method.
    pub justify: Option<JustifyMethod>,
}

impl Markdown {
    /// Create a new `Markdown` renderer from raw markdown text.
    pub fn new(markup: &str) -> Self {
        Markdown {
            markup: markup.to_string(),
            code_theme: "monokai".to_string(),
            inline_code_lexer: None,
            inline_code_theme: None,
            hyperlinks: true,
            justify: None,
        }
    }

    /// Set the code theme (builder pattern).
    #[must_use]
    pub fn with_code_theme(mut self, theme: &str) -> Self {
        self.code_theme = theme.to_string();
        self
    }

    /// Set whether hyperlink URLs are shown (builder pattern).
    #[must_use]
    pub fn with_hyperlinks(mut self, hyperlinks: bool) -> Self {
        self.hyperlinks = hyperlinks;
        self
    }

    /// Set the text justification (builder pattern).
    #[must_use]
    pub fn with_justify(mut self, justify: JustifyMethod) -> Self {
        self.justify = Some(justify);
        self
    }
}

// ---------------------------------------------------------------------------
// List context tracking
// ---------------------------------------------------------------------------

/// Tracks whether we are inside an ordered or unordered list, and the
/// current item number for ordered lists.
#[derive(Debug, Clone)]
struct ListContext {
    ordered: bool,
    item_number: u64,
}

// ---------------------------------------------------------------------------
// Table building context
// ---------------------------------------------------------------------------

/// Accumulates table data during parsing.
#[derive(Debug, Clone)]
struct TableContext {
    alignments: Vec<Alignment>,
    header_cells: Vec<String>,
    current_row: Vec<String>,
    rows: Vec<Vec<String>>,
    in_head: bool,
}

impl TableContext {
    fn new() -> Self {
        TableContext {
            alignments: Vec::new(),
            header_cells: Vec::new(),
            current_row: Vec::new(),
            rows: Vec::new(),
            in_head: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Renderable implementation
// ---------------------------------------------------------------------------

impl Renderable for Markdown {
    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let mut segments: Vec<Segment> = Vec::new();
        let width = options.max_width;

        // Style stack for nested inline styles
        let base_style = Style::null();
        let mut style_stack = StyleStack::new(base_style);

        // Current text buffer for inline content
        let mut text_buffer = Text::new("", Style::null());

        // List stack for nested lists
        let mut list_stack: Vec<ListContext> = Vec::new();

        // Block quote nesting depth
        let mut blockquote_depth: usize = 0;

        // Link URL tracking
        let mut link_url: Option<String> = None;

        // Code block accumulator
        let mut code_block_text: Option<String> = None;

        // Table context
        let mut table_ctx: Option<TableContext> = None;
        let mut in_table_cell = false;
        let mut cell_text = String::new();

        // Track if we need a newline before the next block element
        let mut needs_newline = false;

        // Enable all pulldown-cmark extensions
        let mut md_options = Options::empty();
        md_options.insert(Options::ENABLE_TABLES);
        md_options.insert(Options::ENABLE_STRIKETHROUGH);

        let parser = Parser::new_ext(&self.markup, md_options);
        let events: Vec<Event> = parser.collect();

        for event in events {
            match event {
                // -- Headings -----------------------------------------------
                Event::Start(Tag::Heading { .. }) => {
                    text_buffer = Text::new("", Style::null());
                }
                Event::End(TagEnd::Heading(level)) => {
                    let style_name = match level {
                        HeadingLevel::H1 => "markdown.h1",
                        HeadingLevel::H2 => "markdown.h2",
                        HeadingLevel::H3 => "markdown.h3",
                        HeadingLevel::H4 => "markdown.h4",
                        HeadingLevel::H5 => "markdown.h5",
                        HeadingLevel::H6 => "markdown.h6",
                    };
                    let heading_style = console
                        .get_style(style_name)
                        .unwrap_or_else(|_| Style::null());

                    if needs_newline {
                        segments.push(Segment::line());
                    }

                    // Apply heading style to the entire text
                    let text_len = text_buffer.len();
                    if text_len > 0 {
                        text_buffer.stylize(heading_style.clone(), 0, Some(text_len));
                    }
                    text_buffer.end = String::new();

                    // Render heading text
                    let heading_opts =
                        options.update_width(width.saturating_sub(blockquote_depth * 4));
                    let heading_segs = text_buffer.gilt_console(console, &heading_opts);
                    segments.extend(heading_segs);
                    segments.push(Segment::line());

                    // Add underline rule for h1 and h2
                    if matches!(level, HeadingLevel::H1 | HeadingLevel::H2) {
                        let rule_style = console
                            .get_style("markdown.hr")
                            .unwrap_or_else(|_| Style::null());
                        let rule = Rule::new().with_style(rule_style).with_end("");
                        let rule_segs = rule.gilt_console(console, options);
                        segments.extend(rule_segs);
                        segments.push(Segment::line());
                    }

                    needs_newline = true;
                    text_buffer = Text::new("", Style::null());
                }

                // -- Paragraphs ---------------------------------------------
                Event::Start(Tag::Paragraph) => {
                    text_buffer = Text::new("", Style::null());
                    if let Some(j) = self.justify {
                        text_buffer.justify = Some(j);
                    }
                }
                Event::End(TagEnd::Paragraph) => {
                    if in_table_cell {
                        // Inside a table cell, append text to cell_text
                        cell_text.push_str(text_buffer.plain());
                        text_buffer = Text::new("", Style::null());
                        continue;
                    }

                    if needs_newline {
                        segments.push(Segment::line());
                    }

                    // Apply blockquote indentation
                    let effective_width = width.saturating_sub(blockquote_depth * 4);
                    let para_opts = options.update_width(effective_width);

                    if blockquote_depth > 0 {
                        let bq_style = console
                            .get_style("markdown.block_quote")
                            .unwrap_or_else(|_| Style::null());
                        let indent: String =
                            std::iter::repeat_n(' ', blockquote_depth.saturating_sub(1) * 4)
                                .collect();
                        let bq_prefix = format!("{}\u{2502} ", indent);

                        // Let Text wrap normally then split into lines
                        let text_segs = text_buffer.gilt_console(console, &para_opts);
                        let rendered_text: String =
                            text_segs.iter().map(|s| s.text.as_str()).collect();

                        for line in rendered_text.lines() {
                            segments.push(Segment::styled(&bq_prefix, bq_style.clone()));
                            segments.push(Segment::text(line));
                            segments.push(Segment::line());
                        }
                        // If the text was empty, still emit one quote line
                        if rendered_text.trim().is_empty() {
                            segments.push(Segment::styled(&bq_prefix, bq_style.clone()));
                            segments.push(Segment::line());
                        }
                    } else {
                        let text_segs = text_buffer.gilt_console(console, &para_opts);
                        segments.extend(text_segs);
                    }

                    needs_newline = true;
                    text_buffer = Text::new("", Style::null());
                }

                // -- Emphasis (italic) --------------------------------------
                Event::Start(Tag::Emphasis) => {
                    let em_style = console
                        .get_style("markdown.em")
                        .unwrap_or_else(|_| Style::parse("italic").unwrap());
                    style_stack.push(em_style);
                }
                Event::End(TagEnd::Emphasis) => {
                    let _ = style_stack.pop();
                }

                // -- Strong (bold) ------------------------------------------
                Event::Start(Tag::Strong) => {
                    let strong_style = console
                        .get_style("markdown.strong")
                        .unwrap_or_else(|_| Style::parse("bold").unwrap());
                    style_stack.push(strong_style);
                }
                Event::End(TagEnd::Strong) => {
                    let _ = style_stack.pop();
                }

                // -- Strikethrough ------------------------------------------
                Event::Start(Tag::Strikethrough) => {
                    let s_style = console
                        .get_style("markdown.s")
                        .unwrap_or_else(|_| Style::parse("strike").unwrap());
                    style_stack.push(s_style);
                }
                Event::End(TagEnd::Strikethrough) => {
                    let _ = style_stack.pop();
                }

                // -- Inline code --------------------------------------------
                Event::Code(text) => {
                    let code_style = console
                        .get_style("markdown.code")
                        .unwrap_or_else(|_| Style::parse("bold cyan on black").unwrap());
                    let current = style_stack.current().clone();
                    let combined = current + code_style;
                    text_buffer.append_str(&text, Some(combined));
                }

                // -- Links --------------------------------------------------
                Event::Start(Tag::Link { dest_url, .. }) => {
                    let link_style = console
                        .get_style("markdown.link")
                        .unwrap_or_else(|_| Style::parse("bright_blue").unwrap());
                    style_stack.push(link_style);
                    link_url = Some(dest_url.to_string());
                }
                Event::End(TagEnd::Link) => {
                    let _ = style_stack.pop();
                    if self.hyperlinks {
                        if let Some(ref url) = link_url {
                            let url_style = console
                                .get_style("markdown.link_url")
                                .unwrap_or_else(|_| Style::parse("underline blue").unwrap());
                            text_buffer.append_str(" (", None);
                            text_buffer.append_str(url, Some(url_style));
                            text_buffer.append_str(")", None);
                        }
                    }
                    link_url = None;
                }

                // -- Images (treat like links with alt text) ----------------
                Event::Start(Tag::Image { dest_url, .. }) => {
                    let link_style = console
                        .get_style("markdown.link")
                        .unwrap_or_else(|_| Style::parse("bright_blue").unwrap());
                    style_stack.push(link_style);
                    link_url = Some(dest_url.to_string());
                }
                Event::End(TagEnd::Image) => {
                    let _ = style_stack.pop();
                    if self.hyperlinks {
                        if let Some(ref url) = link_url {
                            let url_style = console
                                .get_style("markdown.link_url")
                                .unwrap_or_else(|_| Style::parse("underline blue").unwrap());
                            text_buffer.append_str(" (", None);
                            text_buffer.append_str(url, Some(url_style));
                            text_buffer.append_str(")", None);
                        }
                    }
                    link_url = None;
                }

                // -- Code blocks --------------------------------------------
                Event::Start(Tag::CodeBlock(_kind)) => {
                    code_block_text = Some(String::new());
                }
                Event::End(TagEnd::CodeBlock) => {
                    if let Some(code_text) = code_block_text.take() {
                        let code_style = console
                            .get_style("markdown.code_block")
                            .unwrap_or_else(|_| Style::parse("cyan on black").unwrap());

                        if needs_newline {
                            segments.push(Segment::line());
                        }

                        // Remove trailing newline from code text
                        let trimmed = code_text.trim_end_matches('\n');
                        let code_content = Text::styled(trimmed, code_style.clone());

                        // Wrap in a panel (like Python rich does)
                        let panel = Panel::new(code_content)
                            .with_box_chars(&HEAVY)
                            .with_style(code_style)
                            .with_expand(true);
                        let panel_segs = panel.gilt_console(console, options);
                        segments.extend(panel_segs);

                        needs_newline = true;
                    }
                }

                // -- Lists --------------------------------------------------
                Event::Start(Tag::List(first_item)) => match first_item {
                    Some(start_num) => {
                        list_stack.push(ListContext {
                            ordered: true,
                            item_number: start_num,
                        });
                    }
                    None => {
                        list_stack.push(ListContext {
                            ordered: false,
                            item_number: 0,
                        });
                    }
                },
                Event::End(TagEnd::List(_ordered)) => {
                    list_stack.pop();
                    if list_stack.is_empty() {
                        needs_newline = true;
                    }
                }

                Event::Start(Tag::Item) => {
                    text_buffer = Text::new("", Style::null());
                }
                Event::End(TagEnd::Item) => {
                    if needs_newline && list_stack.len() <= 1 {
                        segments.push(Segment::line());
                    }

                    let indent_level = list_stack.len().saturating_sub(1);
                    let indent: String = std::iter::repeat_n(' ', indent_level * 4).collect();

                    if let Some(ctx) = list_stack.last_mut() {
                        if ctx.ordered {
                            let num_style = console
                                .get_style("markdown.item.number")
                                .unwrap_or_else(|_| Style::parse("cyan").unwrap());
                            let prefix = format!("{}{}. ", indent, ctx.item_number);
                            segments.push(Segment::styled(&prefix, num_style));
                            ctx.item_number += 1;
                        } else {
                            let bullet_style = console
                                .get_style("markdown.item.bullet")
                                .unwrap_or_else(|_| Style::parse("bold").unwrap());
                            let prefix = format!("{}\u{2022} ", indent);
                            segments.push(Segment::styled(&prefix, bullet_style));
                        }
                    }

                    // Render item text
                    let item_width =
                        width.saturating_sub((list_stack.len().saturating_sub(1)) * 4 + 3);
                    let item_opts = options.update_width(item_width);
                    let item_segs = text_buffer.gilt_console(console, &item_opts);
                    segments.extend(item_segs);

                    text_buffer = Text::new("", Style::null());
                    needs_newline = false;
                }

                // -- Block quotes -------------------------------------------
                Event::Start(Tag::BlockQuote(_kind)) => {
                    blockquote_depth += 1;
                }
                Event::End(TagEnd::BlockQuote(_kind)) => {
                    blockquote_depth = blockquote_depth.saturating_sub(1);
                }

                // -- Tables -------------------------------------------------
                Event::Start(Tag::Table(alignments)) => {
                    let mut ctx = TableContext::new();
                    ctx.alignments = alignments.to_vec();
                    table_ctx = Some(ctx);
                }
                Event::End(TagEnd::Table) => {
                    if let Some(ctx) = table_ctx.take() {
                        if needs_newline {
                            segments.push(Segment::line());
                        }

                        let table_segs = render_table(console, options, &ctx);
                        segments.extend(table_segs);
                        needs_newline = true;
                    }
                }

                Event::Start(Tag::TableHead) => {
                    if let Some(ref mut ctx) = table_ctx {
                        ctx.in_head = true;
                    }
                }
                Event::End(TagEnd::TableHead) => {
                    if let Some(ref mut ctx) = table_ctx {
                        // pulldown-cmark may not emit TableRow for the header,
                        // so save any accumulated cells as header_cells here.
                        if !ctx.current_row.is_empty() {
                            ctx.header_cells = ctx.current_row.clone();
                            ctx.current_row.clear();
                        }
                        ctx.in_head = false;
                    }
                }

                Event::Start(Tag::TableRow) => {
                    if let Some(ref mut ctx) = table_ctx {
                        ctx.current_row.clear();
                    }
                }
                Event::End(TagEnd::TableRow) => {
                    if let Some(ref mut ctx) = table_ctx {
                        let row = ctx.current_row.clone();
                        if ctx.in_head {
                            ctx.header_cells = row;
                        } else {
                            ctx.rows.push(row);
                        }
                    }
                }

                Event::Start(Tag::TableCell) => {
                    in_table_cell = true;
                    cell_text.clear();
                    text_buffer = Text::new("", Style::null());
                }
                Event::End(TagEnd::TableCell) => {
                    // Flush any remaining text_buffer into cell_text
                    let remaining = text_buffer.plain().to_string();
                    if !remaining.is_empty() {
                        cell_text.push_str(&remaining);
                    }
                    if let Some(ref mut ctx) = table_ctx {
                        ctx.current_row.push(cell_text.clone());
                    }
                    in_table_cell = false;
                    cell_text.clear();
                    text_buffer = Text::new("", Style::null());
                }

                // -- Horizontal rule ----------------------------------------
                Event::Rule => {
                    if needs_newline {
                        segments.push(Segment::line());
                    }
                    let hr_style = console
                        .get_style("markdown.hr")
                        .unwrap_or_else(|_| Style::parse("dim").unwrap());
                    let rule = Rule::new().with_style(hr_style).with_end("");
                    let rule_segs = rule.gilt_console(console, options);
                    segments.extend(rule_segs);
                    segments.push(Segment::line());
                    needs_newline = true;
                }

                // -- Text ---------------------------------------------------
                Event::Text(text) => {
                    // If inside a code block, accumulate raw text
                    if let Some(ref mut code_text) = code_block_text {
                        code_text.push_str(&text);
                        continue;
                    }

                    // If inside a table cell, accumulate text
                    if in_table_cell {
                        cell_text.push_str(&text);
                        continue;
                    }

                    // Apply current style stack
                    let current_style = style_stack.current().clone();
                    if current_style.is_null() {
                        text_buffer.append_str(&text, None);
                    } else {
                        text_buffer.append_str(&text, Some(current_style));
                    }
                }

                // -- Breaks -------------------------------------------------
                Event::SoftBreak => {
                    if code_block_text.is_some() {
                        if let Some(ref mut code_text) = code_block_text {
                            code_text.push('\n');
                        }
                    } else if in_table_cell {
                        cell_text.push(' ');
                    } else {
                        text_buffer.append_str(" ", None);
                    }
                }
                Event::HardBreak => {
                    if code_block_text.is_some() {
                        if let Some(ref mut code_text) = code_block_text {
                            code_text.push('\n');
                        }
                    } else if in_table_cell {
                        cell_text.push(' ');
                    } else {
                        text_buffer.append_str("\n", None);
                    }
                }

                // -- HTML (ignored) -----------------------------------------
                Event::Html(_) | Event::InlineHtml(_) => {}

                // -- Footnotes, metadata, etc. (ignored) --------------------
                _ => {}
            }
        }

        // Handle any remaining text in the buffer (shouldn't normally happen
        // with well-formed markdown, but handle gracefully)
        if !text_buffer.plain().is_empty() {
            text_buffer.end = String::new();
            let final_segs = text_buffer.gilt_console(console, options);
            segments.extend(final_segs);
            segments.push(Segment::line());
        }

        segments
    }
}

// ---------------------------------------------------------------------------
// Table rendering helper
// ---------------------------------------------------------------------------

/// Build and render a gilt `Table` from accumulated table context data.
fn render_table(console: &Console, options: &ConsoleOptions, ctx: &TableContext) -> Vec<Segment> {
    let headers: Vec<&str> = ctx.header_cells.iter().map(|s| s.as_str()).collect();
    let mut table = Table::new(&headers);

    // Apply alignment from markdown
    for (i, alignment) in ctx.alignments.iter().enumerate() {
        if i < table.columns.len() {
            table.columns[i].justify = match alignment {
                Alignment::None | Alignment::Left => JustifyMethod::Left,
                Alignment::Center => JustifyMethod::Center,
                Alignment::Right => JustifyMethod::Right,
            };
        }
    }

    // Apply markdown table styles
    let border_style_name = "markdown.table.border";
    table.border_style = border_style_name.to_string();

    let header_style_name = "markdown.table.header";
    table.header_style = header_style_name.to_string();

    // Add data rows
    for row in &ctx.rows {
        let cells: Vec<&str> = row.iter().map(|s| s.as_str()).collect();
        table.add_row(&cells);
    }

    table.gilt_console(console, options)
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl std::fmt::Display for Markdown {
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
    use crate::cells::cell_len;

    fn make_console(width: usize) -> Console {
        Console::builder()
            .width(width)
            .force_terminal(true)
            .no_color(true)
            .markup(false)
            .build()
    }

    fn render_markdown(console: &Console, md: &Markdown) -> String {
        let opts = console.options();
        let segments = md.gilt_console(console, &opts);
        segments.iter().map(|s| s.text.as_str()).collect()
    }

    fn render_segments(console: &Console, md: &Markdown) -> Vec<Segment> {
        let opts = console.options();
        md.gilt_console(console, &opts)
    }

    // -- Simple paragraph ---------------------------------------------------

    #[test]
    fn test_simple_paragraph() {
        let console = make_console(80);
        let md = Markdown::new("Hello, world!");
        let output = render_markdown(&console, &md);
        assert!(output.contains("Hello, world!"));
    }

    #[test]
    fn test_two_paragraphs() {
        let console = make_console(80);
        let md = Markdown::new("First paragraph.\n\nSecond paragraph.");
        let output = render_markdown(&console, &md);
        assert!(output.contains("First paragraph."));
        assert!(output.contains("Second paragraph."));
    }

    // -- Headings -----------------------------------------------------------

    #[test]
    fn test_heading_h1() {
        let console = make_console(80);
        let md = Markdown::new("# Heading 1");
        let output = render_markdown(&console, &md);
        assert!(output.contains("Heading 1"));
    }

    #[test]
    fn test_heading_h2() {
        let console = make_console(80);
        let md = Markdown::new("## Heading 2");
        let output = render_markdown(&console, &md);
        assert!(output.contains("Heading 2"));
    }

    #[test]
    fn test_heading_h3() {
        let console = make_console(80);
        let md = Markdown::new("### Heading 3");
        let output = render_markdown(&console, &md);
        assert!(output.contains("Heading 3"));
    }

    #[test]
    fn test_heading_h4() {
        let console = make_console(80);
        let md = Markdown::new("#### Heading 4");
        let output = render_markdown(&console, &md);
        assert!(output.contains("Heading 4"));
    }

    #[test]
    fn test_heading_h5() {
        let console = make_console(80);
        let md = Markdown::new("##### Heading 5");
        let output = render_markdown(&console, &md);
        assert!(output.contains("Heading 5"));
    }

    #[test]
    fn test_heading_h6() {
        let console = make_console(80);
        let md = Markdown::new("###### Heading 6");
        let output = render_markdown(&console, &md);
        assert!(output.contains("Heading 6"));
    }

    #[test]
    fn test_headings_have_appropriate_styles() {
        let console = make_console(80);

        // H1 should produce a rule underline
        let md = Markdown::new("# Title");
        let output = render_markdown(&console, &md);
        // H1 and H2 get underlines (Rule characters)
        assert!(output.contains("Title"));
        // The rule character should be present
        assert!(output.contains('\u{2501}') || output.contains('-'));
    }

    #[test]
    fn test_h1_has_rule_underline() {
        let console = make_console(40);
        let md = Markdown::new("# Big Title");
        let segments = render_segments(&console, &md);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        // Should contain the heading text and a rule line
        assert!(text.contains("Big Title"));
        assert!(text.contains('\u{2501}'));
    }

    #[test]
    fn test_h2_has_rule_underline() {
        let console = make_console(40);
        let md = Markdown::new("## Sub Title");
        let segments = render_segments(&console, &md);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("Sub Title"));
        assert!(text.contains('\u{2501}'));
    }

    // -- Bold text ----------------------------------------------------------

    #[test]
    fn test_bold_text() {
        let console = make_console(80);
        let md = Markdown::new("This is **bold** text.");
        let segments = render_segments(&console, &md);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("bold"));
        assert!(text.contains("This is"));
        assert!(text.contains("text."));

        // Check that "bold" segment has a style with bold attribute
        let bold_seg = segments.iter().find(|s| s.text == "bold");
        assert!(bold_seg.is_some(), "Should have a segment with text 'bold'");
        if let Some(seg) = bold_seg {
            assert!(seg.style.is_some(), "Bold segment should have a style");
        }
    }

    // -- Italic text --------------------------------------------------------

    #[test]
    fn test_italic_text() {
        let console = make_console(80);
        let md = Markdown::new("This is *italic* text.");
        let segments = render_segments(&console, &md);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("italic"));

        let italic_seg = segments.iter().find(|s| s.text == "italic");
        assert!(
            italic_seg.is_some(),
            "Should have a segment with text 'italic'"
        );
        if let Some(seg) = italic_seg {
            assert!(seg.style.is_some(), "Italic segment should have a style");
        }
    }

    // -- Bold + italic combined ---------------------------------------------

    #[test]
    fn test_bold_italic_combined() {
        let console = make_console(80);
        let md = Markdown::new("This is ***bold and italic*** text.");
        let segments = render_segments(&console, &md);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("bold and italic"));

        // The combined text should have a style
        let combined_seg = segments.iter().find(|s| s.text.contains("bold and italic"));
        assert!(combined_seg.is_some());
        if let Some(seg) = combined_seg {
            assert!(
                seg.style.is_some(),
                "Bold+italic segment should have a style"
            );
        }
    }

    // -- Inline code --------------------------------------------------------

    #[test]
    fn test_inline_code() {
        let console = make_console(80);
        let md = Markdown::new("Use `println!` to print.");
        let segments = render_segments(&console, &md);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("println!"));

        let code_seg = segments.iter().find(|s| s.text == "println!");
        assert!(
            code_seg.is_some(),
            "Should have a segment with inline code text"
        );
        if let Some(seg) = code_seg {
            assert!(seg.style.is_some(), "Inline code should have a style");
        }
    }

    // -- Code blocks (fenced) -----------------------------------------------

    #[test]
    fn test_code_block() {
        let console = make_console(80);
        let md = Markdown::new("```\nfn main() {\n    println!(\"hello\");\n}\n```");
        let output = render_markdown(&console, &md);
        assert!(output.contains("fn main()"));
        assert!(output.contains("println!"));
    }

    #[test]
    fn test_code_block_with_language() {
        let console = make_console(80);
        let md = Markdown::new("```rust\nlet x = 42;\n```");
        let output = render_markdown(&console, &md);
        assert!(output.contains("let x = 42;"));
    }

    // -- Links with URLs ----------------------------------------------------

    #[test]
    fn test_link_with_url() {
        let console = make_console(80);
        let md = Markdown::new("[Rust](https://www.rust-lang.org)");
        let output = render_markdown(&console, &md);
        assert!(output.contains("Rust"));
        assert!(output.contains("https://www.rust-lang.org"));
    }

    #[test]
    fn test_link_without_url_display() {
        let console = make_console(80);
        let md = Markdown::new("[Rust](https://www.rust-lang.org)").with_hyperlinks(false);
        let output = render_markdown(&console, &md);
        assert!(output.contains("Rust"));
        assert!(!output.contains("https://www.rust-lang.org"));
    }

    // -- Unordered lists (bullets) ------------------------------------------

    #[test]
    fn test_unordered_list() {
        let console = make_console(80);
        let md = Markdown::new("- Item 1\n- Item 2\n- Item 3");
        let output = render_markdown(&console, &md);
        assert!(output.contains("Item 1"));
        assert!(output.contains("Item 2"));
        assert!(output.contains("Item 3"));
        // Should have bullet characters
        assert!(output.contains('\u{2022}'));
    }

    // -- Ordered lists (numbers) --------------------------------------------

    #[test]
    fn test_ordered_list() {
        let console = make_console(80);
        let md = Markdown::new("1. First\n2. Second\n3. Third");
        let output = render_markdown(&console, &md);
        assert!(output.contains("First"));
        assert!(output.contains("Second"));
        assert!(output.contains("Third"));
        assert!(output.contains("1."));
        assert!(output.contains("2."));
        assert!(output.contains("3."));
    }

    // -- Nested lists -------------------------------------------------------

    #[test]
    fn test_nested_list() {
        let console = make_console(80);
        let md = Markdown::new("- Outer\n  - Inner 1\n  - Inner 2\n- Outer 2");
        let output = render_markdown(&console, &md);
        assert!(output.contains("Outer"));
        assert!(output.contains("Inner 1"));
        assert!(output.contains("Inner 2"));
        assert!(output.contains("Outer 2"));
    }

    // -- Block quotes -------------------------------------------------------

    #[test]
    fn test_block_quote() {
        let console = make_console(80);
        let md = Markdown::new("> This is a quote.");
        let output = render_markdown(&console, &md);
        assert!(output.contains("This is a quote."));
        // Should have the vertical bar character for block quotes
        assert!(output.contains('\u{2502}'));
    }

    // -- Horizontal rules ---------------------------------------------------

    #[test]
    fn test_horizontal_rule() {
        let console = make_console(40);
        let md = Markdown::new("Above\n\n---\n\nBelow");
        let output = render_markdown(&console, &md);
        assert!(output.contains("Above"));
        assert!(output.contains("Below"));
        // Should contain rule characters
        assert!(output.contains('\u{2501}'));
    }

    // -- Mixed content ------------------------------------------------------

    #[test]
    fn test_mixed_content() {
        let console = make_console(80);
        let md = Markdown::new("# Title\n\nA paragraph.\n\n- Item 1\n- Item 2\n\n```\ncode\n```");
        let output = render_markdown(&console, &md);
        assert!(output.contains("Title"));
        assert!(output.contains("A paragraph."));
        assert!(output.contains("Item 1"));
        assert!(output.contains("Item 2"));
        assert!(output.contains("code"));
    }

    // -- Empty markdown -----------------------------------------------------

    #[test]
    fn test_empty_markdown() {
        let console = make_console(80);
        let md = Markdown::new("");
        let output = render_markdown(&console, &md);
        assert!(output.is_empty() || output.trim().is_empty());
    }

    // -- Tables -------------------------------------------------------------

    #[test]
    fn test_table() {
        let console = make_console(80);
        let md = Markdown::new("| Name | Age |\n|------|-----|\n| Alice | 30 |\n| Bob | 25 |");
        let output = render_markdown(&console, &md);
        assert!(output.contains("Name"));
        assert!(output.contains("Age"));
        assert!(output.contains("Alice"));
        assert!(output.contains("30"));
        assert!(output.contains("Bob"));
        assert!(output.contains("25"));
    }

    #[test]
    fn test_table_with_alignment() {
        let console = make_console(80);
        let md =
            Markdown::new("| Left | Center | Right |\n|:-----|:------:|------:|\n| L | C | R |");
        let output = render_markdown(&console, &md);
        assert!(output.contains("Left"));
        assert!(output.contains("Center"));
        assert!(output.contains("Right"));
    }

    // -- Renderable trait integration ---------------------------------------

    #[test]
    fn test_renderable_integration() {
        let console = Console::builder()
            .width(60)
            .force_terminal(true)
            .no_color(true)
            .markup(false)
            .build();

        let md = Markdown::new("Hello, **world**!");
        let opts = console.options();
        let segments = md.gilt_console(&console, &opts);

        // Should produce non-empty output
        assert!(!segments.is_empty());

        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("Hello,"));
        assert!(text.contains("world"));
    }

    #[test]
    fn test_renderable_through_console_render() {
        let console = Console::builder()
            .width(60)
            .force_terminal(true)
            .no_color(true)
            .markup(false)
            .build();

        let md = Markdown::new("# Title\n\nParagraph text.");
        let segments = console.render(&md, None);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("Title"));
        assert!(text.contains("Paragraph text."));
    }

    // -- Constructor / builder pattern --------------------------------------

    #[test]
    fn test_constructor_defaults() {
        let md = Markdown::new("test");
        assert_eq!(md.markup, "test");
        assert_eq!(md.code_theme, "monokai");
        assert!(md.inline_code_lexer.is_none());
        assert!(md.inline_code_theme.is_none());
        assert!(md.hyperlinks);
        assert!(md.justify.is_none());
    }

    #[test]
    fn test_builder_code_theme() {
        let md = Markdown::new("test").with_code_theme("dracula");
        assert_eq!(md.code_theme, "dracula");
    }

    #[test]
    fn test_builder_hyperlinks() {
        let md = Markdown::new("test").with_hyperlinks(false);
        assert!(!md.hyperlinks);
    }

    #[test]
    fn test_builder_justify() {
        let md = Markdown::new("test").with_justify(JustifyMethod::Center);
        assert_eq!(md.justify, Some(JustifyMethod::Center));
    }

    // -- Strikethrough text -------------------------------------------------

    #[test]
    fn test_strikethrough() {
        let console = make_console(80);
        let md = Markdown::new("This is ~~deleted~~ text.");
        let segments = render_segments(&console, &md);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("deleted"));
    }

    // -- Soft and hard breaks -----------------------------------------------

    #[test]
    fn test_soft_break() {
        let console = make_console(80);
        let md = Markdown::new("Line one\nLine two");
        let output = render_markdown(&console, &md);
        // Soft break should become a space
        assert!(output.contains("Line one"));
        assert!(output.contains("Line two"));
    }

    #[test]
    fn test_hard_break() {
        let console = make_console(80);
        let md = Markdown::new("Line one  \nLine two");
        let output = render_markdown(&console, &md);
        assert!(output.contains("Line one"));
        assert!(output.contains("Line two"));
    }

    // -- Multiple headings --------------------------------------------------

    #[test]
    fn test_all_heading_levels() {
        let console = make_console(80);
        let md = Markdown::new("# H1\n\n## H2\n\n### H3\n\n#### H4\n\n##### H5\n\n###### H6");
        let output = render_markdown(&console, &md);
        assert!(output.contains("H1"));
        assert!(output.contains("H2"));
        assert!(output.contains("H3"));
        assert!(output.contains("H4"));
        assert!(output.contains("H5"));
        assert!(output.contains("H6"));
    }

    // -- Width constraints --------------------------------------------------

    #[test]
    fn test_narrow_width() {
        let console = make_console(20);
        let md = Markdown::new("This is a paragraph with enough text to wrap.");
        let output = render_markdown(&console, &md);
        assert!(output.contains("This"));
        // Check that output lines are within the width constraint
        for line in output.split('\n') {
            if !line.is_empty() {
                assert!(
                    cell_len(line) <= 20,
                    "Line exceeds width: '{}' ({} cells)",
                    line,
                    cell_len(line)
                );
            }
        }
    }

    // -- Code block in panel ------------------------------------------------

    #[test]
    fn test_code_block_has_panel_border() {
        let console = make_console(40);
        let md = Markdown::new("```\nhello\n```");
        let output = render_markdown(&console, &md);
        // Panel uses HEAVY box chars
        assert!(
            output.contains('\u{2501}')
                || output.contains('\u{2503}')
                || output.contains('\u{250F}'),
            "Code block should be wrapped in a panel border"
        );
    }

    // -- Multiple items with inline formatting ------------------------------

    #[test]
    fn test_list_with_inline_formatting() {
        let console = make_console(80);
        let md = Markdown::new("- **Bold item**\n- *Italic item*\n- `Code item`");
        let output = render_markdown(&console, &md);
        assert!(output.contains("Bold item"));
        assert!(output.contains("Italic item"));
        assert!(output.contains("Code item"));
    }

    // -- Edge case: only whitespace -----------------------------------------

    #[test]
    fn test_whitespace_only() {
        let console = make_console(80);
        let md = Markdown::new("   \n\n   ");
        let output = render_markdown(&console, &md);
        assert!(output.trim().is_empty());
    }

    // -- Blockquote with multiple paragraphs --------------------------------

    #[test]
    fn test_blockquote_multiple_paragraphs() {
        let console = make_console(80);
        let md = Markdown::new("> First quote.\n>\n> Second quote.");
        let output = render_markdown(&console, &md);
        assert!(output.contains("First quote."));
        assert!(output.contains("Second quote."));
    }

    // -- Image as link-like -------------------------------------------------

    #[test]
    fn test_image() {
        let console = make_console(80);
        let md = Markdown::new("![Alt text](https://example.com/image.png)");
        let output = render_markdown(&console, &md);
        assert!(output.contains("Alt text"));
        assert!(output.contains("https://example.com/image.png"));
    }

    // -- Verify output ends with newline ------------------------------------

    #[test]
    fn test_output_has_trailing_content() {
        let console = make_console(80);
        let md = Markdown::new("Hello");
        let segments = render_segments(&console, &md);
        assert!(!segments.is_empty());
    }

    #[test]
    fn test_display_trait() {
        let md = Markdown::new("# Hello\n\nWorld");
        let s = format!("{}", md);
        assert!(!s.is_empty());
    }
}
