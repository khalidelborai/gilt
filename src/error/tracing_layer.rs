//! Integration with the [`tracing`](https://docs.rs/tracing) crate for structured, rich terminal logging.
//!
//! Enable with the `tracing` Cargo feature flag:
//!
//! ```toml
//! gilt = { version = "0.10", features = ["tracing"] }
//! ```
//!
//! # Example
//!
//! ```ignore
//! use gilt::tracing_layer::GiltLayer;
//! use tracing_subscriber::prelude::*;
//!
//! tracing_subscriber::registry()
//!     .with(GiltLayer::new())
//!     .init();
//!
//! tracing::info!(user = "alice", "request handled");
//! ```

use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;

use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;

use crate::console::Console;
use crate::highlighter::{Highlighter, ReprHighlighter};
use crate::markup;
use crate::style::Style;
use crate::text::{JustifyMethod, Text};
use crate::widgets::table::Table;

// ---------------------------------------------------------------------------
// Default keywords (mirrors logging_handler::DEFAULT_KEYWORDS)
// ---------------------------------------------------------------------------

const DEFAULT_KEYWORDS: &[&str] = &[
    "GET", "POST", "HEAD", "PUT", "DELETE", "OPTIONS", "TRACE", "PATCH",
];

// ---------------------------------------------------------------------------
// Field visitor — collects structured fields from a tracing event
// ---------------------------------------------------------------------------

/// Collects the `message` field and all other key=value structured fields.
struct FieldVisitor {
    message: Option<String>,
    fields: Vec<(String, String)>,
}

impl FieldVisitor {
    fn new() -> Self {
        Self {
            message: None,
            fields: Vec::new(),
        }
    }
}

impl Visit for FieldVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{:?}", value));
        } else {
            self.fields
                .push((field.name().to_string(), format!("{:?}", value)));
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields
                .push((field.name().to_string(), value.to_string()));
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }
}

// ---------------------------------------------------------------------------
// GiltLayer
// ---------------------------------------------------------------------------

/// A [`tracing_subscriber::Layer`] that formats tracing events using gilt's
/// [`Console`] for beautiful, styled terminal output.
///
/// ## Column layout
///
/// Each event is rendered as a single line with optional columns:
///
/// - **Time** (`HH:MM:SS`, dim style) — toggle with [`with_show_time`](Self::with_show_time)
/// - **Level** (color-coded, 8 chars wide) — toggle with [`with_show_level`](Self::with_show_level)
/// - **Message** and structured fields
/// - **Target** (module path, dim style) — toggle with [`with_show_target`](Self::with_show_target)
/// - **Span path** (parent spans, italic style) — toggle with [`with_show_span_path`](Self::with_show_span_path)
///
/// ## Level color mapping
///
/// | Level | Style |
/// |-------|-------|
/// | ERROR | bold red |
/// | WARN  | bold yellow |
/// | INFO  | bold blue |
/// | DEBUG | bold green |
/// | TRACE | dim |
pub struct GiltLayer {
    console: Mutex<Console>,
    show_time: bool,
    show_target: bool,
    show_level: bool,
    show_span_path: bool,
    /// When `true`, log messages are parsed as rich markup.
    markup: bool,
    /// Keywords to highlight in log messages.
    keywords: Vec<String>,
    /// Highlighter applied to log messages. Default: [`ReprHighlighter`].
    highlighter: Box<dyn Highlighter + Send + Sync>,
    /// Minimum tracing level to emit. Events less severe are suppressed.
    min_level: Level,
    /// Level style map — consulted after theme lookup.
    level_styles: HashMap<Level, Style>,
}

impl GiltLayer {
    /// Create a new `GiltLayer` with all columns enabled and a default console.
    pub fn new() -> Self {
        Self {
            console: Mutex::new(Console::new()),
            show_time: true,
            show_target: true,
            show_level: true,
            show_span_path: true,
            markup: false,
            keywords: DEFAULT_KEYWORDS.iter().map(|s| s.to_string()).collect(),
            highlighter: Box::new(ReprHighlighter),
            min_level: Level::TRACE,
            level_styles: Self::default_level_styles(),
        }
    }

    /// Replace the console used for output.
    #[must_use]
    pub fn with_console(mut self, console: Console) -> Self {
        self.console = Mutex::new(console);
        self
    }

    /// Set whether to show the time column.
    #[must_use]
    pub fn with_show_time(mut self, show: bool) -> Self {
        self.show_time = show;
        self
    }

    /// Set whether to show the target (module path) column.
    #[must_use]
    pub fn with_show_target(mut self, show: bool) -> Self {
        self.show_target = show;
        self
    }

    /// Set whether to show the level column.
    #[must_use]
    pub fn with_show_level(mut self, show: bool) -> Self {
        self.show_level = show;
        self
    }

    /// Set whether to show the span path (parent spans) column.
    #[must_use]
    pub fn with_show_span_path(mut self, show: bool) -> Self {
        self.show_span_path = show;
        self
    }

    /// Set whether log messages are parsed as rich markup.
    #[must_use]
    pub fn with_markup(mut self, m: bool) -> Self {
        self.markup = m;
        self
    }

    /// Set the keywords to highlight in log messages.
    #[must_use]
    pub fn with_keywords(mut self, kw: Vec<String>) -> Self {
        self.keywords = kw;
        self
    }

    /// Replace the highlighter applied to log messages.
    ///
    /// Default: [`ReprHighlighter`]. Pass a [`crate::highlighter::NullHighlighter`] to disable.
    #[must_use]
    pub fn with_highlighter<H: Highlighter + Send + Sync + 'static>(mut self, h: H) -> Self {
        self.highlighter = Box::new(h);
        self
    }

    /// Set the minimum tracing level to emit.
    ///
    /// Events with lower severity than `level` are suppressed.
    /// Default: [`Level::TRACE`] (pass everything).
    #[must_use]
    pub fn with_min_level(mut self, level: Level) -> Self {
        self.min_level = level;
        self
    }

    /// Override the level style map.
    #[must_use]
    pub fn with_level_styles(mut self, styles: HashMap<Level, Style>) -> Self {
        self.level_styles = styles;
        self
    }

    /// Return the default level style map.
    fn default_level_styles() -> HashMap<Level, Style> {
        let mut m = HashMap::new();
        m.insert(Level::ERROR, Style::parse("bold red"));
        m.insert(Level::WARN, Style::parse("bold yellow"));
        m.insert(Level::INFO, Style::parse("bold blue"));
        m.insert(Level::DEBUG, Style::parse("bold green"));
        m.insert(Level::TRACE, Style::parse("dim"));
        m
    }

    /// Return the style for a given tracing level (static helper, kept for tests).
    #[cfg(test)]
    fn level_style(level: &Level) -> Style {
        let spec = match *level {
            Level::ERROR => "bold red",
            Level::WARN => "bold yellow",
            Level::INFO => "bold blue",
            Level::DEBUG => "bold green",
            Level::TRACE => "dim",
        };
        Style::parse(spec)
    }

    /// Build the time column text (HH:MM:SS) in dim style.
    ///
    /// Finding #21: delegates to `crate::error::fmt_time_hms()` to eliminate
    /// the duplicated UTC-time computation between `logging_handler` and here.
    fn render_time() -> Text {
        let now = crate::error::fmt_time_hms();
        let dim_style = Style::parse("dim");
        Text::styled_with(&now, dim_style)
    }

    /// Build the level column text, left-padded to 8 chars.
    ///
    /// Tries theme lookup via `console.get_style("logging.level.{name}")` first,
    /// falls back to `self.level_styles`.
    #[cfg(test)]
    fn render_level(&self, level: &Level, console: &Console) -> Text {
        let name = match *level {
            Level::ERROR => "ERROR",
            Level::WARN => "WARN",
            Level::INFO => "INFO",
            Level::DEBUG => "DEBUG",
            Level::TRACE => "TRACE",
        };
        let lvl_lower = name.to_lowercase();
        let style = console
            .get_style(&format!("logging.level.{}", lvl_lower))
            .unwrap_or_else(|_| {
                self.level_styles
                    .get(level)
                    .cloned()
                    .unwrap_or_else(Style::null)
            });
        let padded = format!("{:<8}", name);
        Text::styled_with(&padded, style)
    }

    /// Build the target column (module path) in dim style.
    fn render_target(target: &str) -> Text {
        let dim_style = Style::parse("dim");
        Text::styled_with(target, dim_style)
    }

    /// Format structured fields as `key=value` pairs in dim italic style.
    fn render_fields(fields: &[(String, String)]) -> Text {
        if fields.is_empty() {
            return Text::new("", Style::null());
        }
        let parts: Vec<String> = fields.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
        let joined = parts.join(" ");
        let style = Style::parse("dim italic");
        Text::styled_with(&joined, style)
    }

    /// Compose all columns into a grid row and print via the console.
    fn emit<S: Subscriber + for<'a> LookupSpan<'a>>(
        &self,
        event: &Event<'_>,
        ctx: &Context<'_, S>,
    ) {
        // Collect fields from the event
        let mut visitor = FieldVisitor::new();
        event.record(&mut visitor);

        let metadata = event.metadata();

        // Phase 1: collect styles via a brief console lock, then drop.
        let (level_style, kw_style) = if let Ok(console) = self.console.lock() {
            let lvl_name = format!("logging.level.{}", metadata.level().as_str().to_lowercase());
            let lvl = console.get_style(&lvl_name).unwrap_or_else(|_| {
                self.level_styles
                    .get(metadata.level())
                    .cloned()
                    .unwrap_or_else(Style::null)
            });
            let kw = console
                .get_style("logging.keyword")
                .unwrap_or_else(|_| Style::parse("bold on dark_green"));
            (lvl, kw)
        } else {
            (
                self.level_styles
                    .get(metadata.level())
                    .cloned()
                    .unwrap_or_else(Style::null),
                Style::parse("bold on dark_green"),
            )
        };

        let mut cells: Vec<Text> = Vec::new();
        let mut headers: Vec<&str> = Vec::new();

        // Time column
        if self.show_time {
            cells.push(Self::render_time());
            headers.push("");
        }

        // Level column — use pre-fetched style
        if self.show_level {
            let name = match *metadata.level() {
                Level::ERROR => "ERROR",
                Level::WARN => "WARN",
                Level::INFO => "INFO",
                Level::DEBUG => "DEBUG",
                Level::TRACE => "TRACE",
            };
            let padded = format!("{:<8}", name);
            cells.push(Text::styled_with(&padded, level_style));
            headers.push("");
        }

        // Span path (if enabled and spans exist) — prepended to message cell
        let mut message_cell = Text::new("", Style::null());
        if self.show_span_path {
            if let Some(scope) = ctx.event_scope(event) {
                let span_names: Vec<&str> = scope.from_root().map(|s| s.name()).collect();
                if !span_names.is_empty() {
                    let path = span_names.join(":");
                    let span_style = Style::parse("italic cyan");
                    let span_text = Text::styled_with(&path, span_style);
                    message_cell.append_text(&span_text);
                    message_cell.append_str(" ", None);
                }
            }
        }

        // Message — apply markup, highlighter, keyword highlighting
        let message = visitor.message.unwrap_or_default();
        let mut msg_text = if self.markup {
            let base = Style::null();
            markup::render(&message, base).unwrap_or_else(|_| Text::new(&message, Style::null()))
        } else {
            Text::new(&message, Style::null())
        };
        self.highlighter.highlight(&mut msg_text);
        if !self.keywords.is_empty() {
            let words: Vec<&str> = self.keywords.iter().map(|s| s.as_str()).collect();
            msg_text.highlight_words(&words, kw_style, false);
        }
        message_cell.append_text(&msg_text);

        // Structured fields appended into message cell
        if !visitor.fields.is_empty() {
            message_cell.append_str(" ", None);
            let fields_text = Self::render_fields(&visitor.fields);
            message_cell.append_text(&fields_text);
        }

        cells.push(message_cell);
        headers.push("");

        // Target column
        let has_target = if self.show_target {
            let target = metadata.target();
            if !target.is_empty() {
                cells.push(Self::render_target(target));
                headers.push("");
                true
            } else {
                false
            }
        } else {
            false
        };

        let mut grid = Table::grid(&headers);
        // Match rich LogRender: padding=(0,1) between columns, no left pad,
        // expand to full width so target column lands at a fixed right edge.
        grid.padding = (0, 1, 0, 0);
        grid.set_expand(true);

        // Right-justify the target column so it is flush with the terminal right edge.
        if has_target && !grid.columns.is_empty() {
            let last = grid.columns.len() - 1;
            grid.columns[last].justify = JustifyMethod::Right;
        }

        grid.add_row_text(&cells);

        // Print via console
        if let Ok(mut console) = self.console.lock() {
            console.print(&grid);
        }
    }
}

impl Default for GiltLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for GiltLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        self.emit(event, &ctx);
    }

    fn event_enabled(&self, event: &Event<'_>, _ctx: Context<'_, S>) -> bool {
        event.metadata().level() <= &self.min_level
    }
}

// ---------------------------------------------------------------------------
// install() convenience
// ---------------------------------------------------------------------------

/// Create a default [`GiltLayer`] and install it as the global tracing subscriber.
///
/// Sets up a `tracing_subscriber::Registry` with a `GiltLayer` and installs it
/// globally.
///
/// # Errors
///
/// Returns an error if a global subscriber has already been set.
pub fn install() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tracing_subscriber::prelude::*;
    tracing_subscriber::registry()
        .with(GiltLayer::new())
        .try_init()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::Console;
    use crate::highlighter::NullHighlighter;
    use tracing_subscriber::prelude::*;

    fn test_console() -> Console {
        Console::builder().width(80).no_color(true).build()
    }

    // -- Construction and defaults -------------------------------------------

    #[test]
    fn test_default_construction() {
        let layer = GiltLayer::new();
        assert!(layer.show_time);
        assert!(layer.show_target);
        assert!(layer.show_level);
        assert!(layer.show_span_path);
        assert!(!layer.markup);
        assert!(!layer.keywords.is_empty());
        assert_eq!(layer.min_level, Level::TRACE);
    }

    #[test]
    fn test_default_trait() {
        let layer = GiltLayer::default();
        assert!(layer.show_time);
        assert!(layer.show_target);
        assert!(layer.show_level);
        assert!(layer.show_span_path);
    }

    // -- Builder methods -----------------------------------------------------

    #[test]
    fn test_builder_show_time() {
        let layer = GiltLayer::new().with_show_time(false);
        assert!(!layer.show_time);
    }

    #[test]
    fn test_builder_show_target() {
        let layer = GiltLayer::new().with_show_target(false);
        assert!(!layer.show_target);
    }

    #[test]
    fn test_builder_show_level() {
        let layer = GiltLayer::new().with_show_level(false);
        assert!(!layer.show_level);
    }

    #[test]
    fn test_builder_show_span_path() {
        let layer = GiltLayer::new().with_show_span_path(false);
        assert!(!layer.show_span_path);
    }

    #[test]
    fn test_builder_console() {
        let console = Console::builder().width(120).build();
        let _layer = GiltLayer::new().with_console(console);
    }

    #[test]
    fn test_markup_builder() {
        let layer = GiltLayer::new().with_markup(true);
        assert!(layer.markup);
    }

    #[test]
    fn test_keywords_builder() {
        let layer = GiltLayer::new().with_keywords(vec!["FOO".to_string(), "BAR".to_string()]);
        assert_eq!(layer.keywords, vec!["FOO", "BAR"]);
    }

    #[test]
    fn test_with_highlighter_builder() {
        let layer = GiltLayer::new().with_highlighter(NullHighlighter);
        // No easy way to inspect the box type, but construction should succeed.
        drop(layer);
    }

    #[test]
    fn test_min_level_builder() {
        let layer = GiltLayer::new().with_min_level(Level::WARN);
        assert_eq!(layer.min_level, Level::WARN);
    }

    #[test]
    fn test_level_styles_builder() {
        let mut styles = HashMap::new();
        styles.insert(Level::ERROR, Style::parse("italic magenta"));
        let layer = GiltLayer::new().with_level_styles(styles);
        let s = layer.level_styles.get(&Level::ERROR).unwrap();
        assert_eq!(s.italic(), Some(true));
    }

    // -- Level styles --------------------------------------------------------

    #[test]
    fn test_error_style_is_bold_red() {
        let style = GiltLayer::level_style(&Level::ERROR);
        assert_eq!(style.bold(), Some(true));
        assert!(style.color().is_some());
        assert_eq!(style.color().unwrap().name(), "red");
    }

    #[test]
    fn test_warn_style_is_bold_yellow() {
        let style = GiltLayer::level_style(&Level::WARN);
        assert_eq!(style.bold(), Some(true));
        assert_eq!(style.color().unwrap().name(), "yellow");
    }

    #[test]
    fn test_info_style_is_bold_blue() {
        let style = GiltLayer::level_style(&Level::INFO);
        assert_eq!(style.bold(), Some(true));
        assert_eq!(style.color().unwrap().name(), "blue");
    }

    #[test]
    fn test_debug_style_is_bold_green() {
        let style = GiltLayer::level_style(&Level::DEBUG);
        assert_eq!(style.bold(), Some(true));
        assert_eq!(style.color().unwrap().name(), "green");
    }

    #[test]
    fn test_trace_style_is_dim() {
        let style = GiltLayer::level_style(&Level::TRACE);
        assert_eq!(style.dim(), Some(true));
    }

    // -- Render helpers ------------------------------------------------------

    #[test]
    fn test_render_time_format() {
        let time_text = GiltLayer::render_time();
        let plain = time_text.plain().to_string();
        assert_eq!(plain.len(), 8);
        assert_eq!(plain.as_bytes()[2], b':');
        assert_eq!(plain.as_bytes()[5], b':');
    }

    #[test]
    fn test_render_time_has_dim_style() {
        let time_text = GiltLayer::render_time();
        assert!(!time_text.spans().is_empty());
    }

    #[test]
    fn test_render_level_error() {
        let layer = GiltLayer::new();
        let console = test_console();
        let text = layer.render_level(&Level::ERROR, &console);
        assert_eq!(text.plain(), "ERROR   ");
    }

    #[test]
    fn test_render_level_warn() {
        let layer = GiltLayer::new();
        let console = test_console();
        let text = layer.render_level(&Level::WARN, &console);
        assert_eq!(text.plain(), "WARN    ");
    }

    #[test]
    fn test_render_level_info() {
        let layer = GiltLayer::new();
        let console = test_console();
        let text = layer.render_level(&Level::INFO, &console);
        assert_eq!(text.plain(), "INFO    ");
    }

    #[test]
    fn test_render_level_debug() {
        let layer = GiltLayer::new();
        let console = test_console();
        let text = layer.render_level(&Level::DEBUG, &console);
        assert_eq!(text.plain(), "DEBUG   ");
    }

    #[test]
    fn test_render_level_trace() {
        let layer = GiltLayer::new();
        let console = test_console();
        let text = layer.render_level(&Level::TRACE, &console);
        assert_eq!(text.plain(), "TRACE   ");
    }

    #[test]
    fn test_render_level_has_style() {
        let layer = GiltLayer::new();
        let console = test_console();
        for level in &[
            Level::ERROR,
            Level::WARN,
            Level::INFO,
            Level::DEBUG,
            Level::TRACE,
        ] {
            let text = layer.render_level(level, &console);
            assert!(
                !text.spans().is_empty(),
                "level {:?} should have a styled span",
                level
            );
        }
    }

    #[test]
    fn test_render_target() {
        let text = GiltLayer::render_target("my_crate::module");
        assert_eq!(text.plain(), "my_crate::module");
        assert!(!text.spans().is_empty());
    }

    #[test]
    fn test_render_target_empty() {
        let text = GiltLayer::render_target("");
        assert_eq!(text.plain(), "");
    }

    #[test]
    fn test_render_fields_empty() {
        let text = GiltLayer::render_fields(&[]);
        assert_eq!(text.plain(), "");
    }

    #[test]
    fn test_render_fields_single() {
        let fields = vec![("user".to_string(), "alice".to_string())];
        let text = GiltLayer::render_fields(&fields);
        assert_eq!(text.plain(), "user=alice");
        assert!(!text.spans().is_empty());
    }

    #[test]
    fn test_render_fields_multiple() {
        let fields = vec![
            ("user".to_string(), "alice".to_string()),
            ("status".to_string(), "200".to_string()),
        ];
        let text = GiltLayer::render_fields(&fields);
        assert_eq!(text.plain(), "user=alice status=200");
    }

    // -- Field visitor -------------------------------------------------------

    #[test]
    fn test_field_visitor_record_str_message() {
        let mut visitor = FieldVisitor::new();
        let field = tracing::field::FieldSet::new(
            &["message"],
            tracing::callsite::Identifier(&TestCallsite),
        )
        .field("message")
        .unwrap();
        visitor.record_str(&field, "hello world");
        assert_eq!(visitor.message.as_deref(), Some("hello world"));
        assert!(visitor.fields.is_empty());
    }

    #[test]
    fn test_field_visitor_record_str_other() {
        let mut visitor = FieldVisitor::new();
        let field_set =
            tracing::field::FieldSet::new(&["user"], tracing::callsite::Identifier(&TestCallsite));
        let field = field_set.field("user").unwrap();
        visitor.record_str(&field, "bob");
        assert!(visitor.message.is_none());
        assert_eq!(visitor.fields.len(), 1);
        assert_eq!(visitor.fields[0].0, "user");
        assert_eq!(visitor.fields[0].1, "bob");
    }

    #[test]
    fn test_field_visitor_record_i64() {
        let mut visitor = FieldVisitor::new();
        let field_set =
            tracing::field::FieldSet::new(&["count"], tracing::callsite::Identifier(&TestCallsite));
        let field = field_set.field("count").unwrap();
        visitor.record_i64(&field, -42);
        assert_eq!(visitor.fields[0], ("count".to_string(), "-42".to_string()));
    }

    #[test]
    fn test_field_visitor_record_u64() {
        let mut visitor = FieldVisitor::new();
        let field_set =
            tracing::field::FieldSet::new(&["count"], tracing::callsite::Identifier(&TestCallsite));
        let field = field_set.field("count").unwrap();
        visitor.record_u64(&field, 99);
        assert_eq!(visitor.fields[0], ("count".to_string(), "99".to_string()));
    }

    #[test]
    fn test_field_visitor_record_bool() {
        let mut visitor = FieldVisitor::new();
        let field_set = tracing::field::FieldSet::new(
            &["active"],
            tracing::callsite::Identifier(&TestCallsite),
        );
        let field = field_set.field("active").unwrap();
        visitor.record_bool(&field, true);
        assert_eq!(
            visitor.fields[0],
            ("active".to_string(), "true".to_string())
        );
    }

    #[test]
    fn test_field_visitor_record_f64() {
        let mut visitor = FieldVisitor::new();
        let field_set =
            tracing::field::FieldSet::new(&["ratio"], tracing::callsite::Identifier(&TestCallsite));
        let field = field_set.field("ratio").unwrap();
        visitor.record_f64(&field, 1.5);
        assert_eq!(visitor.fields[0], ("ratio".to_string(), "1.5".to_string()));
    }

    // -- Subscriber integration (captured output) ----------------------------

    #[test]
    fn test_subscriber_captures_info_event() {
        let console = Console::builder()
            .width(120)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();
        let layer = GiltLayer::new()
            .with_console(console)
            .with_show_time(false)
            .with_show_level(true)
            .with_show_target(false)
            .with_show_span_path(false);

        let subscriber = tracing_subscriber::registry().with(layer);
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!("hello from tracing");
        });

        // We cannot access the console after the layer is consumed, but
        // the test verifies no panics during event emission.
    }

    #[test]
    fn test_subscriber_captures_all_levels() {
        let console = Console::builder()
            .width(120)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();
        let layer = GiltLayer::new()
            .with_console(console)
            .with_show_time(false)
            .with_show_level(true)
            .with_show_target(false)
            .with_show_span_path(false);

        let subscriber = tracing_subscriber::registry().with(layer);
        tracing::subscriber::with_default(subscriber, || {
            tracing::error!("error msg");
            tracing::warn!("warn msg");
            tracing::info!("info msg");
            tracing::debug!("debug msg");
            tracing::trace!("trace msg");
        });
    }

    #[test]
    fn test_subscriber_with_structured_fields() {
        let console = Console::builder()
            .width(120)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();
        let layer = GiltLayer::new()
            .with_console(console)
            .with_show_time(false)
            .with_show_level(false)
            .with_show_target(false)
            .with_show_span_path(false);

        let subscriber = tracing_subscriber::registry().with(layer);
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(user = "alice", status = 200u64, "request handled");
        });
    }

    #[test]
    fn test_subscriber_with_span_context() {
        let console = Console::builder()
            .width(120)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();
        let layer = GiltLayer::new()
            .with_console(console)
            .with_show_time(false)
            .with_show_level(false)
            .with_show_target(false)
            .with_show_span_path(true);

        let subscriber = tracing_subscriber::registry().with(layer);
        tracing::subscriber::with_default(subscriber, || {
            let _guard = tracing::info_span!("my_span").entered();
            tracing::info!("inside span");
        });
    }

    #[test]
    fn test_emit_full_line() {
        let console = Console::builder()
            .width(120)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();
        let layer = GiltLayer::new()
            .with_console(console)
            .with_show_time(true)
            .with_show_level(true)
            .with_show_target(true)
            .with_show_span_path(true);

        let subscriber = tracing_subscriber::registry().with(layer);
        tracing::subscriber::with_default(subscriber, || {
            let _guard = tracing::info_span!("server").entered();
            tracing::info!(port = 8080u64, "listening");
        });
    }

    #[test]
    fn test_install_returns_result() {
        // Verify install function signature; do not actually call it since
        // it sets a global subscriber and would conflict with other tests.
        let _: fn() -> Result<(), Box<dyn std::error::Error + Send + Sync>> = install;
    }

    // -- Helper: dummy callsite for creating fields in tests -----------------

    struct TestCallsite;

    impl tracing::callsite::Callsite for TestCallsite {
        fn set_interest(&self, _: tracing::subscriber::Interest) {}
        fn metadata(&self) -> &tracing::Metadata<'_> {
            // Use a static metadata for test purposes
            static META: tracing::Metadata<'static> = tracing::Metadata::new(
                "test",
                "test",
                Level::INFO,
                None,
                None,
                None,
                tracing::field::FieldSet::new(&[], tracing::callsite::Identifier(&TestCallsite2)),
                tracing::metadata::Kind::EVENT,
            );
            &META
        }
    }

    // A second callsite used by the static metadata in TestCallsite
    struct TestCallsite2;

    impl tracing::callsite::Callsite for TestCallsite2 {
        fn set_interest(&self, _: tracing::subscriber::Interest) {}
        fn metadata(&self) -> &tracing::Metadata<'_> {
            TestCallsite.metadata()
        }
    }
}
