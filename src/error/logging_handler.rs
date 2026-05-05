//! Logging handler for the `log` crate.
//!
//! This module provides a [`RichHandler`] that implements [`log::Log`],
//! producing styled, formatted log output through gilt's [`Console`].
//!

use std::collections::HashMap;
use std::sync::Mutex;

use crate::console::Console;
use crate::error::traceback::Traceback;
use crate::markup;
use crate::style::Style;
use crate::text::Text;

// ---------------------------------------------------------------------------
// Default keywords (HTTP verbs, matching Python's RichHandler.KEYWORDS)
// ---------------------------------------------------------------------------

/// Default keywords highlighted in log messages.
const DEFAULT_KEYWORDS: &[&str] = &[
    "GET", "POST", "HEAD", "PUT", "DELETE", "OPTIONS", "TRACE", "PATCH",
];

// ---------------------------------------------------------------------------
// RichHandler
// ---------------------------------------------------------------------------

/// A [`log::Log`] implementation that produces styled, formatted log output
/// using gilt's [`Console`].
///
/// Each log record is rendered as a line with optional columns:
/// - **Time** (`HH:MM:SS`, dim style)
/// - **Level** (color-coded, 8 chars wide)
/// - **Message** (optionally parsed as markup, with keyword highlighting)
/// - **Path** (`module::path`, dim style, plus line number)
pub struct RichHandler {
    console: Mutex<Console>,
    show_time: bool,
    show_level: bool,
    show_path: bool,
    markup: bool,
    /// When `true`, suppress the timestamp on consecutive records that share
    /// the same wall-clock second, replacing it with spaces of equal width so
    /// columns remain aligned.
    omit_repeated_times: bool,
    /// When `true`, render the `file:line` location as a clickable OSC 8
    /// hyperlink using the `file://` URL scheme.
    enable_link_path: bool,
    /// When `true`, records whose message looks like an error chain are
    /// rendered via [`Traceback::from_error`] for full styled output.
    gilt_tracebacks: bool,
    keywords: Vec<String>,
    level_styles: HashMap<log::Level, Style>,
    /// Cache of the last-emitted `HH:MM:SS` string for `omit_repeated_times`.
    last_time_str: Mutex<Option<String>>,
}

impl RichHandler {
    /// Create a new `RichHandler` with sensible defaults.
    ///
    /// Uses a default [`Console`] and the default level styles.
    pub fn new() -> Self {
        RichHandler {
            console: Mutex::new(Console::new()),
            show_time: true,
            show_level: true,
            show_path: true,
            markup: false,
            omit_repeated_times: true,
            enable_link_path: false,
            gilt_tracebacks: false,
            keywords: DEFAULT_KEYWORDS.iter().map(|s| s.to_string()).collect(),
            level_styles: Self::default_level_styles(),
            last_time_str: Mutex::new(None),
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

    /// Set whether to show the level column.
    #[must_use]
    pub fn with_show_level(mut self, show: bool) -> Self {
        self.show_level = show;
        self
    }

    /// Set whether to show the source path column.
    #[must_use]
    pub fn with_show_path(mut self, show: bool) -> Self {
        self.show_path = show;
        self
    }

    /// Set whether log messages are parsed as rich markup.
    #[must_use]
    pub fn with_markup(mut self, markup: bool) -> Self {
        self.markup = markup;
        self
    }

    /// Set the keywords to highlight in log messages.
    #[must_use]
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    /// Suppress repeated timestamps on consecutive records sharing the same
    /// wall-clock second (default `true`).
    ///
    /// When suppressed, the time column is replaced with spaces of the same
    /// width so the path / level columns stay aligned.
    #[must_use]
    pub fn with_omit_repeated_times(mut self, omit: bool) -> Self {
        self.omit_repeated_times = omit;
        self
    }

    /// Render the `module::path:line` location as a clickable OSC 8 hyperlink
    /// using the `file://` URL scheme (default `false`).
    ///
    /// Useful in IDE-integrated terminals (VS Code, JetBrains, iTerm2) — the
    /// link opens the source file at the right line.
    #[must_use]
    pub fn with_enable_link_path(mut self, enable: bool) -> Self {
        self.enable_link_path = enable;
        self
    }

    /// Render error-bearing log records via [`Traceback`] for full styled
    /// output (default `false`).
    #[must_use]
    pub fn with_gilt_tracebacks(mut self, enable: bool) -> Self {
        self.gilt_tracebacks = enable;
        self
    }

    /// Return the default level style map.
    fn default_level_styles() -> HashMap<log::Level, Style> {
        let mut m = HashMap::new();
        m.insert(log::Level::Error, Style::parse("bold red"));
        m.insert(log::Level::Warn, Style::parse("bold yellow"));
        m.insert(log::Level::Info, Style::parse("bold green"));
        m.insert(log::Level::Debug, Style::parse("bold blue"));
        m.insert(log::Level::Trace, Style::parse("dim"));
        m
    }

    /// Build the level column text, left-padded to 8 chars.
    fn render_level(&self, level: log::Level) -> Text {
        let name = match level {
            log::Level::Error => "ERROR",
            log::Level::Warn => "WARN",
            log::Level::Info => "INFO",
            log::Level::Debug => "DEBUG",
            log::Level::Trace => "TRACE",
        };
        let padded = format!("{:<8}", name);
        let style = self
            .level_styles
            .get(&level)
            .cloned()
            .unwrap_or_else(Style::null);
        Text::styled_with(&padded, style)
    }

    /// Build the message column, optionally parsing markup and highlighting keywords.
    fn render_message(&self, record: &log::Record) -> Text {
        let msg = format!("{}", record.args());
        let mut text = if self.markup {
            let base = Style::null();
            markup::render(&msg, base).unwrap_or_else(|_| Text::new(&msg, Style::null()))
        } else {
            Text::new(&msg, Style::null())
        };

        // Keyword highlighting
        if !self.keywords.is_empty() {
            let kw_style = Style::parse("bold on dark_green");
            let words: Vec<&str> = self.keywords.iter().map(|s| s.as_str()).collect();
            text.highlight_words(&words, kw_style, false);
        }

        text
    }

    /// Build the plain path column (`module::path:line`) without link wrapping.
    /// Kept for tests that want the unwrapped form.
    #[cfg(test)]
    fn render_path(record: &log::Record) -> Text {
        let dim_style = Style::parse("dim");
        let module = record.module_path().unwrap_or("");
        let line = record.line().unwrap_or(0);
        let path_str = if !module.is_empty() {
            format!("{}:{}", module, line)
        } else {
            format!(":{}", line)
        };
        Text::styled_with(&path_str, dim_style)
    }

    /// Build the path column (`module::path:line`) optionally as an OSC 8
    /// hyperlink to a `file://` URL when `enable_link_path` is set.
    ///
    /// The link uses the record's `file()` (absolute or workspace-relative
    /// source path) for the URL when available; the visible text remains the
    /// `module::path:line` form for readability.
    fn render_path_with_link(&self, record: &log::Record) -> Text {
        let dim_style = Style::parse("dim");
        let module = record.module_path().unwrap_or("");
        let line = record.line().unwrap_or(0);
        let path_str = if !module.is_empty() {
            format!("{}:{}", module, line)
        } else {
            format!(":{}", line)
        };

        if self.enable_link_path {
            if let Some(file) = record.file() {
                // Build a file:// URL — use absolute path if available, else
                // pass through the relative path as-is (terminals/editors that
                // understand the protocol will resolve it).
                let abs = std::path::Path::new(file)
                    .canonicalize()
                    .ok()
                    .and_then(|p| p.to_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| file.to_string());
                let url = format!("file://{}", abs);
                let link_style = Style::parse_strict(&format!("dim link {}", url))
                    .unwrap_or_else(|_| dim_style.clone());
                return Text::styled_with(&path_str, link_style);
            }
        }
        Text::styled_with(&path_str, dim_style)
    }

    /// Compose all columns into a single line and print it.
    fn emit(&self, record: &log::Record) {
        let mut parts = Text::new("", Style::null());

        if self.show_time {
            let time_text = self.render_time_with_omit();
            parts.append_text(&time_text);
            parts.append_str(" ", None);
        }

        if self.show_level {
            let level_text = self.render_level(record.level());
            parts.append_text(&level_text);
            parts.append_str(" ", None);
        }

        // Traceback path: when enabled and the message is multi-line (the
        // common Display form for error chains via `{:#}`), render it through
        // a Panel-wrapped Traceback so the call/error chain is styled.
        let msg_str = format!("{}", record.args());
        if self.gilt_tracebacks && record.level() <= log::Level::Error && msg_str.contains('\n') {
            // No real backtrace available for the log record; pass the
            // multi-line message in as the panic message so the Panel-wrapped
            // Traceback renders the full chain.
            let tb = Traceback::from_panic(&msg_str, "");
            if let Ok(mut console) = self.console.lock() {
                console.print(&parts);
                console.print(&tb);
            }
            return;
        }

        let message_text = self.render_message(record);
        parts.append_text(&message_text);

        if self.show_path {
            let path_text = self.render_path_with_link(record);
            parts.append_str(" ", None);
            parts.append_text(&path_text);
        }

        if let Ok(mut console) = self.console.lock() {
            console.print(&parts);
        }
    }

    /// Compute the time string for this record, returning an equal-width
    /// blank string when `omit_repeated_times` is set and the second matches
    /// the previously-emitted time.
    fn render_time_with_omit(&self) -> Text {
        let now = Self::current_time_str();
        if self.omit_repeated_times {
            let mut last = self.last_time_str.lock().unwrap_or_else(|p| p.into_inner());
            if last.as_ref() == Some(&now) {
                let blanks = " ".repeat(now.chars().count());
                return Text::new(&blanks, Style::null());
            }
            *last = Some(now.clone());
        }
        let dim_style = Style::parse("dim");
        Text::styled_with(&now, dim_style)
    }

    /// Return the current wall-clock time as `HH:MM:SS`.
    fn current_time_str() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let dur = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let total_secs = dur.as_secs();
        let hours = (total_secs / 3600) % 24;
        let minutes = (total_secs / 60) % 60;
        let seconds = total_secs % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

impl Default for RichHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl log::Log for RichHandler {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            self.emit(record);
        }
    }

    fn flush(&self) {
        // Console output is not buffered in a way that needs explicit flushing.
    }
}

// ---------------------------------------------------------------------------
// install() convenience
// ---------------------------------------------------------------------------

/// Create a default [`RichHandler`] and install it as the global logger.
///
/// Sets the max log level to [`log::LevelFilter::Trace`] so all messages
/// are forwarded to the handler.
pub fn install() -> Result<(), log::SetLoggerError> {
    let handler = RichHandler::new();
    log::set_boxed_logger(Box::new(handler))?;
    log::set_max_level(log::LevelFilter::Trace);
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::Console;

    // -- Default construction ------------------------------------------------

    #[test]
    fn test_default_construction() {
        let handler = RichHandler::new();
        assert!(handler.show_time);
        assert!(handler.show_level);
        assert!(handler.show_path);
        assert!(!handler.markup);
        assert!(!handler.gilt_tracebacks);
        assert!(!handler.keywords.is_empty());
    }

    #[test]
    fn test_default_trait() {
        let handler = RichHandler::default();
        assert!(handler.show_time);
    }

    // -- Builder methods -----------------------------------------------------

    #[test]
    fn test_builder_show_time() {
        let handler = RichHandler::new().with_show_time(false);
        assert!(!handler.show_time);
    }

    #[test]
    fn test_builder_show_level() {
        let handler = RichHandler::new().with_show_level(false);
        assert!(!handler.show_level);
    }

    #[test]
    fn test_builder_show_path() {
        let handler = RichHandler::new().with_show_path(false);
        assert!(!handler.show_path);
    }

    #[test]
    fn test_builder_markup() {
        let handler = RichHandler::new().with_markup(true);
        assert!(handler.markup);
    }

    #[test]
    fn test_builder_keywords() {
        let handler = RichHandler::new().with_keywords(vec!["FOO".to_string(), "BAR".to_string()]);
        assert_eq!(handler.keywords, vec!["FOO", "BAR"]);
    }

    #[test]
    fn test_builder_console() {
        let console = Console::builder().width(120).build();
        let _handler = RichHandler::new().with_console(console);
        // No panic means success; we cannot inspect the inner console easily.
    }

    // -- Level style mapping -------------------------------------------------

    #[test]
    fn test_level_styles_all_present() {
        let styles = RichHandler::default_level_styles();
        assert!(styles.contains_key(&log::Level::Error));
        assert!(styles.contains_key(&log::Level::Warn));
        assert!(styles.contains_key(&log::Level::Info));
        assert!(styles.contains_key(&log::Level::Debug));
        assert!(styles.contains_key(&log::Level::Trace));
    }

    #[test]
    fn test_error_style_is_bold_red() {
        let styles = RichHandler::default_level_styles();
        let error_style = styles.get(&log::Level::Error).unwrap();
        assert_eq!(error_style.bold(), Some(true));
        assert!(error_style.color().is_some());
        assert_eq!(error_style.color().unwrap().name(), "red");
    }

    #[test]
    fn test_warn_style_is_bold_yellow() {
        let styles = RichHandler::default_level_styles();
        let warn_style = styles.get(&log::Level::Warn).unwrap();
        assert_eq!(warn_style.bold(), Some(true));
        assert_eq!(warn_style.color().unwrap().name(), "yellow");
    }

    #[test]
    fn test_info_style_is_bold_green() {
        let styles = RichHandler::default_level_styles();
        let info_style = styles.get(&log::Level::Info).unwrap();
        assert_eq!(info_style.bold(), Some(true));
        assert_eq!(info_style.color().unwrap().name(), "green");
    }

    #[test]
    fn test_debug_style_is_bold_blue() {
        let styles = RichHandler::default_level_styles();
        let debug_style = styles.get(&log::Level::Debug).unwrap();
        assert_eq!(debug_style.bold(), Some(true));
        assert_eq!(debug_style.color().unwrap().name(), "blue");
    }

    #[test]
    fn test_trace_style_is_dim() {
        let styles = RichHandler::default_level_styles();
        let trace_style = styles.get(&log::Level::Trace).unwrap();
        assert_eq!(trace_style.dim(), Some(true));
    }

    // -- Log formatting: time ------------------------------------------------

    #[test]
    fn test_render_time_format() {
        let handler = RichHandler::new();
        let time_text = handler.render_time_with_omit();
        let plain = time_text.plain().to_string();
        // HH:MM:SS pattern: 8 characters with colons at positions 2 and 5
        assert_eq!(plain.len(), 8);
        assert_eq!(plain.as_bytes()[2], b':');
        assert_eq!(plain.as_bytes()[5], b':');
    }

    #[test]
    fn test_render_time_has_dim_style() {
        let handler = RichHandler::new();
        let time_text = handler.render_time_with_omit();
        assert!(!time_text.spans().is_empty());
    }

    // -- Wave 5B: omit_repeated_times / enable_link_path / gilt_tracebacks ----

    #[test]
    fn omit_repeated_times_blanks_duplicate_timestamps() {
        let handler = RichHandler::new().with_omit_repeated_times(true);
        // First call captures and styles the timestamp.
        let first = handler.render_time_with_omit();
        // Second call within the same second returns same-width blanks.
        let second = handler.render_time_with_omit();
        assert_eq!(second.plain().len(), first.plain().len());
        assert!(
            second.plain().chars().all(|c| c == ' '),
            "expected all spaces, got {:?}",
            second.plain()
        );
    }

    #[test]
    fn omit_repeated_times_disabled_keeps_timestamp() {
        let handler = RichHandler::new().with_omit_repeated_times(false);
        let first = handler.render_time_with_omit();
        let second = handler.render_time_with_omit();
        assert_eq!(first.plain(), second.plain());
        assert!(!first.plain().chars().all(|c| c == ' '));
    }

    #[test]
    fn enable_link_path_wraps_module_in_osc8_link() {
        // Construct a synthetic record with a known file/module/line.
        let handler = RichHandler::new().with_enable_link_path(true);
        let record = log::RecordBuilder::new()
            .args(format_args!("hello"))
            .level(log::Level::Info)
            .target("test_target")
            .module_path(Some("test_target"))
            .file(Some("/tmp/some_test_source.rs"))
            .line(Some(42))
            .build();
        let text = handler.render_path_with_link(&record);
        // Either a span carries the link, or the styled output renders OSC 8.
        let spans = text.spans();
        let has_link = spans.iter().any(|s| s.style.link().is_some());
        assert!(
            has_link,
            "expected a span with a file:// link, got {:?}",
            spans
        );
    }

    #[test]
    fn enable_link_path_disabled_has_no_link() {
        let handler = RichHandler::new().with_enable_link_path(false);
        let record = log::RecordBuilder::new()
            .args(format_args!("hello"))
            .level(log::Level::Info)
            .module_path(Some("m"))
            .line(Some(1))
            .file(Some("/tmp/x.rs"))
            .build();
        let text = handler.render_path_with_link(&record);
        let has_link = text.spans().iter().any(|s| s.style.link().is_some());
        assert!(!has_link);
    }

    #[test]
    fn gilt_tracebacks_builder_sets_field() {
        let handler = RichHandler::new().with_gilt_tracebacks(true);
        assert!(handler.gilt_tracebacks);
    }

    // -- Log formatting: level -----------------------------------------------

    #[test]
    fn test_render_level_error() {
        let handler = RichHandler::new();
        let text = handler.render_level(log::Level::Error);
        assert_eq!(text.plain(), "ERROR   ");
    }

    #[test]
    fn test_render_level_warn() {
        let handler = RichHandler::new();
        let text = handler.render_level(log::Level::Warn);
        assert_eq!(text.plain(), "WARN    ");
    }

    #[test]
    fn test_render_level_info() {
        let handler = RichHandler::new();
        let text = handler.render_level(log::Level::Info);
        assert_eq!(text.plain(), "INFO    ");
    }

    #[test]
    fn test_render_level_debug() {
        let handler = RichHandler::new();
        let text = handler.render_level(log::Level::Debug);
        assert_eq!(text.plain(), "DEBUG   ");
    }

    #[test]
    fn test_render_level_trace() {
        let handler = RichHandler::new();
        let text = handler.render_level(log::Level::Trace);
        assert_eq!(text.plain(), "TRACE   ");
    }

    #[test]
    fn test_render_level_has_style() {
        let handler = RichHandler::new();
        for level in &[
            log::Level::Error,
            log::Level::Warn,
            log::Level::Info,
            log::Level::Debug,
            log::Level::Trace,
        ] {
            let text = handler.render_level(*level);
            assert!(
                !text.spans().is_empty(),
                "level {:?} should have a styled span",
                level
            );
        }
    }

    // -- Log formatting: path ------------------------------------------------

    #[test]
    fn test_render_path_with_module() {
        let record = log::Record::builder()
            .args(format_args!("test"))
            .level(log::Level::Info)
            .module_path(Some("my_crate::module"))
            .line(Some(42))
            .build();
        let text = RichHandler::render_path(&record);
        assert_eq!(text.plain(), "my_crate::module:42");
    }

    #[test]
    fn test_render_path_without_module() {
        let record = log::Record::builder()
            .args(format_args!("test"))
            .level(log::Level::Info)
            .line(Some(10))
            .build();
        let text = RichHandler::render_path(&record);
        assert_eq!(text.plain(), ":10");
    }

    #[test]
    fn test_render_path_has_dim_style() {
        let record = log::Record::builder()
            .args(format_args!("test"))
            .level(log::Level::Info)
            .module_path(Some("foo"))
            .line(Some(1))
            .build();
        let text = RichHandler::render_path(&record);
        assert!(!text.spans().is_empty());
    }

    // -- Show/hide time, level, path -----------------------------------------

    #[test]
    fn test_emit_no_time() {
        let console = Console::builder()
            .width(80)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();
        let handler = RichHandler::new()
            .with_console(console)
            .with_show_time(false)
            .with_show_level(true)
            .with_show_path(false);

        let record = log::Record::builder()
            .args(format_args!("hello world"))
            .level(log::Level::Info)
            .build();
        handler.emit(&record);

        let mut console = handler.console.lock().unwrap();
        let output = console.export_text(true, false);
        // Should NOT contain a time pattern
        assert!(
            !output.contains(':'),
            "output should not have time, got: {}",
            output
        );
        // Should contain the level and message
        assert!(output.contains("INFO"));
        assert!(output.contains("hello world"));
    }

    #[test]
    fn test_emit_no_level() {
        let console = Console::builder()
            .width(80)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();
        let handler = RichHandler::new()
            .with_console(console)
            .with_show_time(false)
            .with_show_level(false)
            .with_show_path(false);

        let record = log::Record::builder()
            .args(format_args!("hello world"))
            .level(log::Level::Warn)
            .build();
        handler.emit(&record);

        let mut console = handler.console.lock().unwrap();
        let output = console.export_text(true, false);
        assert!(!output.contains("WARN"));
        assert!(output.contains("hello world"));
    }

    #[test]
    fn test_emit_no_path() {
        let console = Console::builder()
            .width(80)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();
        let handler = RichHandler::new()
            .with_console(console)
            .with_show_time(false)
            .with_show_level(false)
            .with_show_path(false);

        let record = log::Record::builder()
            .args(format_args!("hello world"))
            .level(log::Level::Info)
            .module_path(Some("test_mod"))
            .line(Some(99))
            .build();
        handler.emit(&record);

        let mut console = handler.console.lock().unwrap();
        let output = console.export_text(true, false);
        assert!(!output.contains("test_mod"));
        assert!(output.contains("hello world"));
    }

    #[test]
    fn test_emit_with_path() {
        let console = Console::builder()
            .width(120)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();
        let handler = RichHandler::new()
            .with_console(console)
            .with_show_time(false)
            .with_show_level(false)
            .with_show_path(true);

        let record = log::Record::builder()
            .args(format_args!("hello"))
            .level(log::Level::Debug)
            .module_path(Some("mymod"))
            .line(Some(7))
            .build();
        handler.emit(&record);

        let mut console = handler.console.lock().unwrap();
        let output = console.export_text(true, false);
        assert!(output.contains("mymod:7"));
    }

    // -- Markup parsing in messages ------------------------------------------

    #[test]
    fn test_render_message_plain() {
        let handler = RichHandler::new().with_markup(false).with_keywords(vec![]);
        let record = log::Record::builder()
            .args(format_args!("simple message"))
            .level(log::Level::Info)
            .build();
        let text = handler.render_message(&record);
        assert_eq!(text.plain(), "simple message");
    }

    #[test]
    fn test_render_message_with_markup() {
        let handler = RichHandler::new().with_markup(true).with_keywords(vec![]);
        let record = log::Record::builder()
            .args(format_args!("[bold]hello[/bold] world"))
            .level(log::Level::Info)
            .build();
        let text = handler.render_message(&record);
        // Plain text should have markup stripped
        assert_eq!(text.plain(), "hello world");
        // Should have a span for the bold markup
        assert!(!text.spans().is_empty());
    }

    // -- Different log levels ------------------------------------------------

    #[test]
    fn test_emit_all_levels() {
        let levels = [
            log::Level::Error,
            log::Level::Warn,
            log::Level::Info,
            log::Level::Debug,
            log::Level::Trace,
        ];
        let names = ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"];

        for (level, name) in levels.iter().zip(names.iter()) {
            let console = Console::builder()
                .width(80)
                .no_color(true)
                .record(true)
                .markup(false)
                .build();
            let handler = RichHandler::new()
                .with_console(console)
                .with_show_time(false)
                .with_show_level(true)
                .with_show_path(false);

            let record = log::Record::builder()
                .args(format_args!("msg"))
                .level(*level)
                .build();
            handler.emit(&record);

            let mut console = handler.console.lock().unwrap();
            let output = console.export_text(true, false);
            assert!(
                output.contains(name),
                "expected '{}' in output for {:?}, got: {}",
                name,
                level,
                output
            );
        }
    }

    // -- Keyword highlighting ------------------------------------------------

    #[test]
    fn test_keyword_highlighting() {
        let handler = RichHandler::new()
            .with_markup(false)
            .with_keywords(vec!["GET".to_string(), "POST".to_string()]);

        let record = log::Record::builder()
            .args(format_args!("GET /index.html 200"))
            .level(log::Level::Info)
            .build();
        let text = handler.render_message(&record);
        assert_eq!(text.plain(), "GET /index.html 200");
        // Should have at least one span for the keyword "GET"
        assert!(!text.spans().is_empty(), "expected keyword span for GET");
    }

    #[test]
    fn test_no_keyword_highlighting_when_empty() {
        let handler = RichHandler::new().with_markup(false).with_keywords(vec![]);

        let record = log::Record::builder()
            .args(format_args!("GET /index.html 200"))
            .level(log::Level::Info)
            .build();
        let text = handler.render_message(&record);
        // No keywords to highlight, so no spans
        assert!(text.spans().is_empty());
    }

    // -- log::Log trait implementation ---------------------------------------

    #[test]
    fn test_log_trait_enabled_always_true() {
        let handler = RichHandler::new();
        let metadata = log::MetadataBuilder::new()
            .level(log::Level::Trace)
            .target("test")
            .build();
        assert!(log::Log::enabled(&handler, &metadata));
    }

    #[test]
    fn test_log_trait_log_produces_output() {
        let console = Console::builder()
            .width(80)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();
        let handler = RichHandler::new()
            .with_console(console)
            .with_show_time(false)
            .with_show_level(true)
            .with_show_path(false);

        let record = log::Record::builder()
            .args(format_args!("log trait test"))
            .level(log::Level::Info)
            .build();
        log::Log::log(&handler, &record);

        let mut console = handler.console.lock().unwrap();
        let output = console.export_text(true, false);
        assert!(output.contains("log trait test"));
    }

    #[test]
    fn test_log_trait_flush_does_not_panic() {
        let handler = RichHandler::new();
        log::Log::flush(&handler);
    }

    // -- install() function --------------------------------------------------

    // Note: log::set_logger can only be called once per process, so we test
    // the function signature rather than calling it multiple times.

    #[test]
    fn test_install_returns_result() {
        // We cannot actually call install() in tests because it's a global
        // singleton and other tests might conflict. Instead, verify the
        // function exists and returns the right type.
        let _: fn() -> Result<(), log::SetLoggerError> = install;
    }

    // -- Full integration: captured output -----------------------------------

    #[test]
    fn test_full_line_with_all_columns() {
        let console = Console::builder()
            .width(120)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();
        let handler = RichHandler::new()
            .with_console(console)
            .with_show_time(true)
            .with_show_level(true)
            .with_show_path(true);

        let record = log::Record::builder()
            .args(format_args!("Server starting"))
            .level(log::Level::Info)
            .module_path(Some("my_app::server"))
            .line(Some(42))
            .build();
        handler.emit(&record);

        let mut console = handler.console.lock().unwrap();
        let output = console.export_text(true, false);

        // Time column (HH:MM:SS pattern)
        assert!(output.contains(':'), "expected time in output");
        // Level column
        assert!(output.contains("INFO"));
        // Message
        assert!(output.contains("Server starting"));
        // Path
        assert!(output.contains("my_app::server:42"));
    }

    #[test]
    fn test_full_line_minimal_columns() {
        let console = Console::builder()
            .width(80)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();
        let handler = RichHandler::new()
            .with_console(console)
            .with_show_time(false)
            .with_show_level(false)
            .with_show_path(false);

        let record = log::Record::builder()
            .args(format_args!("bare message"))
            .level(log::Level::Error)
            .build();
        handler.emit(&record);

        let mut console = handler.console.lock().unwrap();
        let output = console.export_text(true, false);
        // Should contain only the message
        assert!(output.contains("bare message"));
        assert!(!output.contains("ERROR"));
    }

    #[test]
    fn test_default_keywords_present() {
        let handler = RichHandler::new();
        assert!(handler.keywords.contains(&"GET".to_string()));
        assert!(handler.keywords.contains(&"POST".to_string()));
        assert!(handler.keywords.contains(&"PUT".to_string()));
        assert!(handler.keywords.contains(&"DELETE".to_string()));
    }
}
