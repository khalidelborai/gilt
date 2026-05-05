//! Markdown rendering module -- parses CommonMark and produces styled terminal output.
//!
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
    /// Plain-text column headers (used for `Table::new` which takes `&[&str]`).
    header_cells: Vec<String>,
    /// Current row's cells as styled `Text` objects (preserves inline styles).
    current_row: Vec<Text>,
    /// Completed data rows as styled `Text` objects.
    rows: Vec<Vec<Text>>,
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
        // Accumulates styled inline content for the current table cell.
        let mut cell_text = Text::new("", Style::null());

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
                        // Inside a table cell, preserve spans from text_buffer
                        // (using append_text, not plain(), to retain styling).
                        cell_text.append_text(&text_buffer);
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
                        .unwrap_or_else(|_| Style::parse("italic"));
                    style_stack.push(em_style);
                }
                Event::End(TagEnd::Emphasis) => {
                    let _ = style_stack.pop();
                }

                // -- Strong (bold) ------------------------------------------
                Event::Start(Tag::Strong) => {
                    let strong_style = console
                        .get_style("markdown.strong")
                        .unwrap_or_else(|_| Style::parse("bold"));
                    style_stack.push(strong_style);
                }
                Event::End(TagEnd::Strong) => {
                    let _ = style_stack.pop();
                }

                // -- Strikethrough ------------------------------------------
                Event::Start(Tag::Strikethrough) => {
                    let s_style = console
                        .get_style("markdown.s")
                        .unwrap_or_else(|_| Style::parse("strike"));
                    style_stack.push(s_style);
                }
                Event::End(TagEnd::Strikethrough) => {
                    let _ = style_stack.pop();
                }

                // -- Inline code --------------------------------------------
                Event::Code(text) => {
                    let code_style = console
                        .get_style("markdown.code")
                        .unwrap_or_else(|_| Style::parse("bold cyan on black"));
                    let current = style_stack.current().clone();
                    let combined = current + code_style;
                    if in_table_cell {
                        // Redirect styled inline code directly into cell_text so
                        // it lands at the correct position (not deferred through
                        // text_buffer, which would reorder it relative to
                        // surrounding Event::Text spans).  Mirrors the Rich
                        // v15.0.0 fix (commit 7ef2d05c).
                        cell_text.append_str(&text, Some(combined));
                    } else {
                        text_buffer.append_str(&text, Some(combined));
                    }
                }

                // -- Links --------------------------------------------------
                Event::Start(Tag::Link { dest_url, .. }) => {
                    let link_style = console
                        .get_style("markdown.link")
                        .unwrap_or_else(|_| Style::parse("bright_blue"));
                    style_stack.push(link_style);
                    link_url = Some(dest_url.to_string());
                }
                Event::End(TagEnd::Link) => {
                    let _ = style_stack.pop();
                    if self.hyperlinks {
                        if let Some(ref url) = link_url {
                            let url_style = console
                                .get_style("markdown.link_url")
                                .unwrap_or_else(|_| Style::parse("underline blue"));
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
                        .unwrap_or_else(|_| Style::parse("bright_blue"));
                    style_stack.push(link_style);
                    link_url = Some(dest_url.to_string());
                }
                Event::End(TagEnd::Image) => {
                    let _ = style_stack.pop();
                    if self.hyperlinks {
                        if let Some(ref url) = link_url {
                            let url_style = console
                                .get_style("markdown.link_url")
                                .unwrap_or_else(|_| Style::parse("underline blue"));
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
                            .unwrap_or_else(|_| Style::parse("cyan on black"));

                        if needs_newline {
                            segments.push(Segment::line());
                        }

                        // Remove trailing newline from code text
                        let trimmed = code_text.trim_end_matches('\n');
                        let code_content = Text::styled_with(trimmed, code_style.clone());

                        // Wrap in a panel (like  does)
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
                                .unwrap_or_else(|_| Style::parse("cyan"));
                            let prefix = format!("{}{}. ", indent, ctx.item_number);
                            segments.push(Segment::styled(&prefix, num_style));
                            ctx.item_number += 1;
                        } else {
                            let bullet_style = console
                                .get_style("markdown.item.bullet")
                                .unwrap_or_else(|_| Style::parse("bold"));
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
                        // Extract plain text — Table::new takes &[&str].
                        if !ctx.current_row.is_empty() {
                            ctx.header_cells = ctx
                                .current_row
                                .iter()
                                .map(|t| t.plain().to_string())
                                .collect();
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
                            // Header cells stored as plain text for Table::new.
                            ctx.header_cells = row.iter().map(|t| t.plain().to_string()).collect();
                        } else {
                            ctx.rows.push(row);
                        }
                        ctx.current_row.clear();
                    }
                }

                Event::Start(Tag::TableCell) => {
                    in_table_cell = true;
                    cell_text = Text::new("", Style::null());
                    text_buffer = Text::new("", Style::null());
                }
                Event::End(TagEnd::TableCell) => {
                    // Flush any remaining text_buffer into cell_text, preserving spans.
                    if !text_buffer.is_empty() {
                        cell_text.append_text(&text_buffer);
                    }
                    if let Some(ref mut ctx) = table_ctx {
                        ctx.current_row.push(cell_text.clone());
                    }
                    in_table_cell = false;
                    cell_text = Text::new("", Style::null());
                    text_buffer = Text::new("", Style::null());
                }

                // -- Horizontal rule ----------------------------------------
                Event::Rule => {
                    if needs_newline {
                        segments.push(Segment::line());
                    }
                    let hr_style = console
                        .get_style("markdown.hr")
                        .unwrap_or_else(|_| Style::parse("dim"));
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

                    // If inside a table cell, accumulate styled text directly
                    // so surrounding style (bold, italic) in cells is preserved.
                    if in_table_cell {
                        let current_style = style_stack.current().clone();
                        if current_style.is_null() {
                            cell_text.append_str(&text, None);
                        } else {
                            cell_text.append_str(&text, Some(current_style));
                        }
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
                        cell_text.append_str(" ", None);
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
                        cell_text.append_str(" ", None);
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

    // Add data rows — use add_row_text to preserve inline styles (e.g. `code`).
    for row in &ctx.rows {
        table.add_row_text(row);
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
#[path = "markdown_tests.rs"]
mod tests;
