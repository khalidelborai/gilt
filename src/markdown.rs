//! Markdown rendering module -- parses CommonMark and produces styled terminal output.
//!
//! (a CommonMark-compliant markdown parser) instead of Python's `markdown_it`.

use pulldown_cmark::{Alignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

#[cfg(not(feature = "syntax"))]
use crate::box_chars::HEAVY;
use crate::box_chars::SIMPLE;
use crate::console::{Console, ConsoleOptions, Renderable};
#[cfg(not(feature = "syntax"))]
use crate::panel::Panel;
use crate::rule::Rule;
use crate::segment::Segment;
use crate::style::{Style, StyleStack};
use crate::table::Table;
use crate::text::{JustifyMethod, Text};
use crate::widgets::table::ColumnOptions;

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
    /// The digit-width to use for right-aligning ordered list numbers.
    /// Pre-computed from the total item count before the render pass begins,
    /// so all items in a list use the same field width (e.g. items 1–9 in a
    /// 10-item list render as " 1." not "1.").
    max_digits: usize,
}

// ---------------------------------------------------------------------------
// Table building context
// ---------------------------------------------------------------------------

/// Accumulates table data during parsing.
#[derive(Debug, Clone)]
struct TableContext {
    alignments: Vec<Alignment>,
    /// Styled column headers (preserves inline bold/italic from markdown).
    header_cells: Vec<Text>,
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

/// Remove trailing blockquote prefix and padding segments that were emitted
/// after the final newline of a block but have no content following them.
///
/// The blockquote prefix-wrapping loops emit a prefix after every `\n`
/// segment to prepare for the next content line.  When the segment stream
/// ends with a `\n`, the last emitted prefix (and any padding/indent
/// segments that follow it) dangles with no content after it — rich only
/// prefixes content lines, never a trailing one.  This function pops those
/// dangling non-content segments, stopping at the last `\n` or content.
fn strip_trailing_bq_prefix(segments: &mut Vec<Segment>, bq_prefix: &str) {
    while let Some(seg) = segments.last() {
        if seg.text == bq_prefix || seg.text.trim().is_empty() {
            segments.pop();
        } else {
            break;
        }
    }
}

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

        // Code block accumulator and language tag
        let mut code_block_text: Option<String> = None;
        let mut code_block_lang: Option<String> = None;

        // Table context
        let mut table_ctx: Option<TableContext> = None;
        let mut in_table_cell = false;
        // Accumulates styled inline content for the current table cell.
        let mut cell_text = Text::new("", Style::null());

        // Track if we need a newline before the next block element
        let mut needs_newline = false;

        // Image alt-text tracking: len of text_buffer after emoji prefix is appended.
        let mut image_text_start: usize = 0;
        let mut in_image = false;

        // Enable all pulldown-cmark extensions
        let mut md_options = Options::empty();
        md_options.insert(Options::ENABLE_TABLES);
        md_options.insert(Options::ENABLE_STRIKETHROUGH);
        md_options.insert(Options::ENABLE_TASKLISTS);

        // Collect all events so we can pre-compute ordered-list item counts
        // (needed for right-aligned numbering: all items use max_digits based
        // on the largest number in the list, not just the current item).
        let events: Vec<Event<'_>> = Parser::new_ext(&self.markup, md_options).collect();

        // Pre-pass: compute the total item count (and therefore max_digits)
        // for every ordered list.  We push max_digits in list-OPEN order
        // (i.e. when Start(List(Some(..))) fires) so the Vec index matches
        // the render pass, which also consumes values in open-order.
        //
        // When a Start(List(Some(start))) is encountered we scan forward
        // through the remaining events to count how many direct-child items
        // this list has (using a nesting-depth counter so we skip items that
        // belong to inner lists), compute max_digits, and push immediately.
        //
        // This guarantees ordered_list_max_digits[0] = outermost list's
        // max_digits, [1] = next list opened, etc., matching the order the
        // render pass increments ordered_list_idx.
        //
        // (The old approach pushed on End, which is DFS-close order: inner
        // lists close before outer ones, inverting the index relative to the
        // render pass's open-order consumption — causing outer lists to use
        // the inner list's max_digits and vice versa.)
        let mut ordered_list_max_digits: Vec<usize> = Vec::new();
        {
            for (ev_idx, ev) in events.iter().enumerate() {
                if let Event::Start(Tag::List(Some(start))) = ev {
                    // Scan ahead to count direct-child items of this list.
                    // Track nesting depth: we are at depth 0 for this list.
                    // depth 0 = inside this list (not nested deeper).
                    let mut depth: usize = 0;
                    let mut item_count: u64 = 0;
                    for future_ev in events.iter().skip(ev_idx + 1) {
                        match future_ev {
                            Event::Start(Tag::List(_)) => {
                                // Entering a nested list — increase depth.
                                depth += 1;
                            }
                            Event::End(TagEnd::List(_)) => {
                                if depth == 0 {
                                    // This is the End for *this* list — stop.
                                    break;
                                }
                                depth -= 1;
                            }
                            Event::Start(Tag::Item) if depth == 0 => {
                                // Direct child item of this list.
                                item_count += 1;
                            }
                            _ => {}
                        }
                    }
                    let last_num = start + item_count.saturating_sub(1);
                    ordered_list_max_digits.push(count_digits(last_num));
                }
            }
        }
        // We'll consume max_digits values as ordered lists are opened.
        // Use an iterator index tracked by a separate counter.
        let mut ordered_list_idx = 0usize;

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
                    if blockquote_depth > 0 {
                        let bq_style = console
                            .get_style("markdown.block_quote")
                            .unwrap_or_else(|_| Style::null());
                        let indent: String =
                            std::iter::repeat_n(' ', blockquote_depth.saturating_sub(1) * 4)
                                .collect();
                        let bq_prefix = format!("{}\u{258C} ", indent);
                        segments.push(Segment::styled(&bq_prefix, bq_style.clone()));
                        for seg in &heading_segs {
                            if seg.text == "\n" {
                                segments.push(Segment::line());
                                segments.push(Segment::styled(&bq_prefix, bq_style.clone()));
                            } else {
                                segments.push(seg.clone());
                            }
                        }
                        // Remove trailing blockquote prefix that was emitted after
                        // the final newline but has no content following it.
                        strip_trailing_bq_prefix(&mut segments, &bq_prefix);
                    } else {
                        segments.extend(heading_segs);
                    }
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
                    // P2 parity: push paragraph style on entry
                    let para_style = console
                        .get_style("markdown.paragraph")
                        .unwrap_or_else(|_| Style::null());
                    style_stack.push(para_style);
                }
                Event::End(TagEnd::Paragraph) => {
                    // P2 parity: pop paragraph style
                    let _ = style_stack.pop();

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
                        // P2 parity: rich uses ▌ (U+258C left half block) not │ (U+2502)
                        let bq_prefix = format!("{}\u{258C} ", indent);

                        // P1 parity: preserve inline styles by working per-segment.
                        // Render the paragraph first, then split at newline segments
                        // and prepend the blockquote prefix to each logical line,
                        // keeping the styled segments intact.
                        let text_segs = text_buffer.gilt_console(console, &para_opts);
                        if text_segs.is_empty()
                            || text_segs.iter().all(|s| s.text.trim().is_empty())
                        {
                            segments.push(Segment::styled(&bq_prefix, bq_style.clone()));
                            segments.push(Segment::line());
                        } else {
                            // Walk segs, emitting prefix at start-of-line
                            segments.push(Segment::styled(&bq_prefix, bq_style.clone()));
                            for seg in &text_segs {
                                if seg.text == "\n" {
                                    segments.push(Segment::line());
                                    segments.push(Segment::styled(&bq_prefix, bq_style.clone()));
                                } else {
                                    segments.push(seg.clone());
                                }
                            }
                            // Remove trailing blockquote prefix that was emitted after
                            // the final newline but has no content following it.
                            strip_trailing_bq_prefix(&mut segments, &bq_prefix);
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
                    #[cfg(feature = "syntax")]
                    {
                        if let Some(ref lexer_name) = self.inline_code_lexer {
                            let theme = self
                                .inline_code_theme
                                .as_deref()
                                .unwrap_or(&self.code_theme);
                            let highlighted =
                                crate::syntax::Syntax::highlight_inline(&text, lexer_name, theme);
                            if in_table_cell {
                                cell_text.append_text(&highlighted);
                            } else {
                                text_buffer.append_text(&highlighted);
                            }
                            continue;
                        }
                    }
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
                    // P1 parity: rich shows URL inline as "(url)" when hyperlinks==false;
                    // when hyperlinks==true, the link text itself is the clickable hyperlink
                    // (no extra URL appended).
                    if !self.hyperlinks {
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
                    style_stack.push(link_style.clone());
                    link_url = Some(dest_url.to_string());
                    // P2 parity: prepend 🌆 emoji prefix for images.
                    // Deep-review fix: when hyperlinks=true, apply the OSC-8
                    // link to the prefix glyph too — rich wraps the whole
                    // image representation (prefix + alt) in the link.
                    let prefix_style = if self.hyperlinks {
                        Some(link_style + Style::with_link(&dest_url))
                    } else {
                        Some(link_style)
                    };
                    text_buffer.append_str("\u{1F306} ", prefix_style);
                    // Track where alt text starts so we can detect empty alt.
                    image_text_start = text_buffer.len();
                    in_image = true;
                }
                Event::End(TagEnd::Image) => {
                    let _ = style_stack.pop();
                    // Item 7: if no alt text was accumulated, fall back to the filename.
                    if in_image && text_buffer.len() == image_text_start {
                        if let Some(ref url) = link_url {
                            let fallback = url_to_filename(url);
                            let link_style = console
                                .get_style("markdown.link")
                                .unwrap_or_else(|_| Style::parse("bright_blue"));
                            text_buffer.append_str(&fallback, Some(link_style));
                        }
                    }
                    in_image = false;
                    // P1 parity: same as links — show URL inline only when hyperlinks==false
                    if !self.hyperlinks {
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
                Event::Start(Tag::CodeBlock(kind)) => {
                    code_block_text = Some(String::new());
                    // P1 parity: capture language tag for syntax highlighting
                    code_block_lang = match kind {
                        CodeBlockKind::Fenced(lang) if !lang.is_empty() => Some(lang.to_string()),
                        _ => None,
                    };
                }
                Event::End(TagEnd::CodeBlock) => {
                    if let Some(code_text) = code_block_text.take() {
                        let _lang = code_block_lang.take();
                        #[cfg(feature = "syntax")]
                        let lang = _lang;

                        if needs_newline {
                            segments.push(Segment::line());
                        }

                        // Remove trailing newline from code text
                        let trimmed = code_text.trim_end_matches('\n');

                        // P1 parity: use Syntax renderable when feature is enabled and
                        // language is known; fall back to plain Panel otherwise.
                        #[cfg(feature = "syntax")]
                        let block_segs = {
                            let used_lang = lang.as_deref().unwrap_or("text");
                            let syn = crate::syntax::Syntax::new(trimmed, used_lang)
                                .with_theme(&self.code_theme)
                                .with_word_wrap(true)
                                .with_padding(crate::syntax::PaddingSpec::Uniform(1));
                            syn.gilt_console(console, options)
                        };
                        #[cfg(not(feature = "syntax"))]
                        let block_segs = {
                            let code_style = console
                                .get_style("markdown.code_block")
                                .unwrap_or_else(|_| Style::parse("cyan on black"));
                            let code_content = Text::styled_with(trimmed, code_style.clone());
                            let panel = Panel::new(code_content)
                                .with_box_chars(&HEAVY)
                                .with_style(code_style)
                                .with_expand(true);
                            panel.gilt_console(console, options)
                        };

                        if blockquote_depth > 0 {
                            let bq_style = console
                                .get_style("markdown.block_quote")
                                .unwrap_or_else(|_| Style::null());
                            let indent: String =
                                std::iter::repeat_n(' ', blockquote_depth.saturating_sub(1) * 4)
                                    .collect();
                            let bq_prefix = format!("{}\u{258C} ", indent);
                            segments.push(Segment::styled(&bq_prefix, bq_style.clone()));
                            for seg in &block_segs {
                                if seg.text == "\n" {
                                    segments.push(Segment::line());
                                    segments.push(Segment::styled(&bq_prefix, bq_style.clone()));
                                } else {
                                    segments.push(seg.clone());
                                }
                            }
                            // Remove trailing blockquote prefix that was emitted after
                            // the final newline but has no content following it.
                            strip_trailing_bq_prefix(&mut segments, &bq_prefix);
                        } else {
                            segments.extend(block_segs);
                        }

                        needs_newline = true;
                    }
                }

                // -- Lists --------------------------------------------------
                Event::Start(Tag::List(first_item)) => match first_item {
                    Some(start_num) => {
                        // Pull the pre-computed max_digits for this ordered list.
                        let max_digits = ordered_list_max_digits
                            .get(ordered_list_idx)
                            .copied()
                            .unwrap_or(1);
                        ordered_list_idx += 1;
                        list_stack.push(ListContext {
                            ordered: true,
                            item_number: start_num,
                            max_digits,
                        });
                    }
                    None => {
                        list_stack.push(ListContext {
                            ordered: false,
                            item_number: 0,
                            max_digits: 1,
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
                    // P3 perf: use static slices for the most common indent levels
                    // to avoid a per-item heap allocation.
                    let indent_owned: String;
                    let indent: &str = match indent_level {
                        0 => "",
                        1 => "    ",
                        2 => "        ",
                        _ => {
                            indent_owned = std::iter::repeat_n(' ', indent_level * 4).collect();
                            &indent_owned
                        }
                    };

                    // Accumulate item segments separately so we can wrap them
                    // with the blockquote prefix when inside a blockquote.
                    let mut item_buf: Vec<Segment> = Vec::new();

                    if let Some(ctx) = list_stack.last_mut() {
                        if ctx.ordered {
                            let num_style = console
                                .get_style("markdown.item.number")
                                .unwrap_or_else(|_| Style::parse("cyan"));
                            // Use the pre-computed max_digits so all items in a
                            // list use the same field width (e.g. items 1–9 in a
                            // 10-item list render as " 1." not "1.").
                            let prefix = format!(
                                "{}{:>width$}. ",
                                indent,
                                ctx.item_number,
                                width = ctx.max_digits
                            );
                            item_buf.push(Segment::styled(&prefix, num_style));
                            ctx.item_number += 1;
                        } else {
                            let bullet_style = console
                                .get_style("markdown.item.bullet")
                                .unwrap_or_else(|_| Style::parse("bold"));
                            // P3 parity: rich uses " • " (leading space, 3 cells)
                            let prefix = format!("{} \u{2022} ", indent);
                            item_buf.push(Segment::styled(&prefix, bullet_style));
                        }
                    }

                    // Render item text
                    // P2 parity: account for 3-cell " • " prefix in width calculation
                    let item_width = width
                        .saturating_sub(blockquote_depth * 4)
                        .saturating_sub((list_stack.len().saturating_sub(1)) * 4 + 3);
                    let item_opts = options.update_width(item_width);
                    let item_segs = text_buffer.gilt_console(console, &item_opts);
                    // P2 parity: prepend the item indent to continuation lines
                    let cont_indent: String =
                        std::iter::repeat_n(' ', indent_level * 4 + 3).collect();
                    let n_item_segs = item_segs.len();
                    let mut first_line = true;
                    for (i, seg) in item_segs.into_iter().enumerate() {
                        if !first_line && seg.text == "\n" {
                            item_buf.push(seg);
                            // Indent the NEXT wrapped continuation line — but NOT
                            // after the item's TRAILING newline, or the indent
                            // leaks onto the following sibling item's bullet,
                            // making flat lists look progressively nested.
                            if i + 1 < n_item_segs {
                                item_buf.push(Segment::text(&cont_indent));
                            }
                            continue;
                        }
                        first_line = false;
                        item_buf.push(seg);
                    }

                    // Emit item segments, wrapping with blockquote prefix when needed.
                    // P3 parity: list items inside blockquotes get the ▌ prefix on
                    // each logical line, matching rich's behaviour for other block
                    // elements (headings, code blocks, horizontal rules).
                    if blockquote_depth > 0 {
                        let bq_style = console
                            .get_style("markdown.block_quote")
                            .unwrap_or_else(|_| Style::null());
                        let bq_indent: String =
                            std::iter::repeat_n(' ', blockquote_depth.saturating_sub(1) * 4)
                                .collect();
                        let bq_prefix = format!("{}\u{258C} ", bq_indent);
                        segments.push(Segment::styled(&bq_prefix, bq_style.clone()));
                        for seg in item_buf {
                            if seg.text == "\n" {
                                segments.push(Segment::line());
                                segments.push(Segment::styled(&bq_prefix, bq_style.clone()));
                            } else {
                                segments.push(seg);
                            }
                        }
                        // Remove trailing blockquote prefix that was emitted after
                        // the final newline but has no content following it.
                        strip_trailing_bq_prefix(&mut segments, &bq_prefix);
                    } else {
                        segments.extend(item_buf);
                    }

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
                    if blockquote_depth > 0 {
                        let bq_style = console
                            .get_style("markdown.block_quote")
                            .unwrap_or_else(|_| Style::null());
                        let indent: String =
                            std::iter::repeat_n(' ', blockquote_depth.saturating_sub(1) * 4)
                                .collect();
                        let bq_prefix = format!("{}\u{258C} ", indent);
                        segments.push(Segment::styled(&bq_prefix, bq_style.clone()));
                        for seg in &rule_segs {
                            if seg.text == "\n" {
                                segments.push(Segment::line());
                                segments.push(Segment::styled(&bq_prefix, bq_style.clone()));
                            } else {
                                segments.push(seg.clone());
                            }
                        }
                        // Remove trailing blockquote prefix that was emitted after
                        // the final newline but has no content following it.
                        strip_trailing_bq_prefix(&mut segments, &bq_prefix);
                    } else {
                        segments.extend(rule_segs);
                    }
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

                    // Apply current style stack; add OSC-8 link when hyperlinks=true.
                    let current_style = style_stack.current().clone();
                    let effective_style = if self.hyperlinks {
                        if let Some(ref url) = link_url {
                            current_style + Style::with_link(url)
                        } else {
                            current_style
                        }
                    } else {
                        current_style
                    };
                    if effective_style.is_null() {
                        text_buffer.append_str(&text, None);
                    } else {
                        text_buffer.append_str(&text, Some(effective_style));
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

                // -- GFM task-list markers ------------------------------------
                // pulldown-cmark emits TaskListMarker *inside* the list item,
                // before the item's text.  We prepend the checkbox as the
                // item's bullet/prefix inline with the text buffer.
                Event::TaskListMarker(checked) => {
                    // ☑ (U+2611) for checked, ☐ (U+2610) for unchecked
                    let marker = if checked { "\u{2611} " } else { "\u{2610} " };
                    let bullet_style = console
                        .get_style("markdown.item.bullet")
                        .unwrap_or_else(|_| Style::parse("bold"));
                    if in_table_cell {
                        cell_text.append_str(marker, Some(bullet_style));
                    } else {
                        text_buffer.append_str(marker, Some(bullet_style));
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
    // Build table with no headers — we add styled header columns individually.
    let mut table = Table::new(&[]);

    // P2 parity: rich uses box.SIMPLE (no outer border, header separator only)
    table = table.with_box_chars(Some(&SIMPLE));

    // Item 4 parity: rich markdown tables use no edge padding and collapse inter-column padding.
    table = table.with_pad_edge(false).with_collapse_padding(true);

    // Apply markdown table styles
    let border_style_name = "markdown.table.border";
    table.border_style = border_style_name.to_string();

    let header_style_name = "markdown.table.header";
    table.header_style = header_style_name.to_string();

    // Add columns with styled Text headers (Item 5 parity).
    for (i, header_text) in ctx.header_cells.iter().enumerate() {
        let justify = ctx.alignments.get(i).map(|a| match a {
            Alignment::None | Alignment::Left => JustifyMethod::Left,
            Alignment::Center => JustifyMethod::Center,
            Alignment::Right => JustifyMethod::Right,
        });
        table.add_column_text(
            header_text.clone(),
            "",
            ColumnOptions {
                justify,
                ..Default::default()
            },
        );
    }

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
// Helpers
// ---------------------------------------------------------------------------

/// Count the number of decimal digits in `n` (minimum 1).
fn count_digits(n: u64) -> usize {
    if n == 0 {
        return 1;
    }
    let mut d = 0usize;
    let mut x = n;
    while x > 0 {
        d += 1;
        x /= 10;
    }
    d
}

/// Extract filename stem from a URL for use as image alt-text fallback.
fn url_to_filename(url: &str) -> String {
    let path = url.split('?').next().unwrap_or(url);
    let path = path.split('#').next().unwrap_or(path);
    let last = path.rsplit('/').next().unwrap_or(path);
    if let Some(dot_pos) = last.rfind('.') {
        last[..dot_pos].to_string()
    } else {
        last.to_string()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "markdown_tests.rs"]
mod tests;
