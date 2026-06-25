//! Traceback formatting module for terminal display.
//!
//! Provides the `Traceback` struct for rendering Rust backtraces, error chains,
//! and panic messages with syntax highlighting and source context. Adapted from
//! the `traceback.py` for Rust-specific backtrace formats.

use std::sync::LazyLock;

use regex::Regex;

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::panel::Panel;
use crate::segment::Segment;
use crate::style::Style;
#[cfg(feature = "syntax")]
use crate::syntax::Syntax;
use crate::text::{Text, TextPart};
use crate::utils::scope::Scope;

// ---------------------------------------------------------------------------
// Frame
// ---------------------------------------------------------------------------

/// A single frame in a backtrace/traceback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// File path where this frame originates.
    pub filename: String,
    /// Line number within the file (1-based), if known.
    pub lineno: Option<usize>,
    /// The function or method name.
    pub name: String,
    /// The source line at the error location, if available.
    pub source_line: Option<String>,
    /// Local variable name-value pairs for this frame.
    ///
    /// These are supplied by the caller (no runtime introspection) and rendered
    /// beneath the source context when `Traceback::show_locals` is `true`.
    pub locals: Vec<(String, String)>,
}

impl Frame {
    /// Create a new frame with the given details.
    pub fn new(filename: &str, lineno: Option<usize>, name: &str) -> Self {
        Frame {
            filename: filename.to_string(),
            lineno,
            name: name.to_string(),
            source_line: None,
            locals: Vec::new(),
        }
    }

    /// Set the source line for this frame.
    #[must_use]
    pub fn with_source_line(mut self, line: &str) -> Self {
        self.source_line = Some(line.to_string());
        self
    }

    /// Set the locals for this frame (replaces any existing locals).
    #[must_use]
    pub fn with_locals(mut self, locals: Vec<(String, String)>) -> Self {
        self.locals = locals;
        self
    }

    /// Add a single local variable name-value pair to this frame.
    #[must_use]
    pub fn with_local(mut self, name: &str, value: &str) -> Self {
        self.locals.push((name.to_string(), value.to_string()));
        self
    }

    /// Try to read the source line from the file system if we have a valid
    /// local path and line number.
    pub fn read_source_line(&mut self) {
        if self.source_line.is_some() {
            return;
        }
        if let Some(lineno) = self.lineno {
            if lineno == 0 {
                return;
            }
            let path = std::path::Path::new(&self.filename);
            if path.is_absolute() || self.filename.starts_with("./") {
                if let Ok(contents) = std::fs::read_to_string(path) {
                    if let Some(line) = contents.lines().nth(lineno - 1) {
                        self.source_line = Some(line.to_string());
                    }
                }
            }
        }
    }
}

impl std::fmt::Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.lineno {
            Some(n) => write!(f, "  {} ({}:{})", self.name, self.filename, n),
            None => write!(f, "  {} ({})", self.name, self.filename),
        }
    }
}

// ---------------------------------------------------------------------------
// Traceback
// ---------------------------------------------------------------------------

/// A formatted traceback that renders error information with syntax
/// highlighting and source context, similar to the Traceback.
#[derive(Debug, Clone)]
pub struct Traceback {
    /// Title displayed at the top (error type or custom title).
    pub title: String,
    /// Error message displayed at the bottom.
    pub message: String,
    /// Stack frames, ordered from outermost to innermost.
    pub frames: Vec<Frame>,
    /// Reserved for future use: display local variables.
    pub show_locals: bool,
    /// Optional fixed width for the output.
    pub width: Option<usize>,
    /// Number of context lines to show around the highlighted source line.
    pub extra_lines: usize,
    /// Syntax highlighting theme name (e.g. "base16-ocean.dark").
    pub theme: String,
    /// Whether to word-wrap long source lines.
    pub word_wrap: bool,
    /// Maximum number of frames to display.
    pub max_frames: usize,
    /// Path prefixes (substrings) to suppress from display.
    ///
    /// Any frame whose `filename` contains one of these strings is hidden.
    /// Mirrors Rich's `Traceback(suppress=[...])`.  Examples:
    /// - `"/.cargo/registry/src/"` hides all third-party registry frames
    /// - `"tokio-"` hides Tokio internals
    pub suppress_paths: Vec<String>,
    /// PEP 678-style notes appended after the error message.
    ///
    /// Each entry is rendered as a styled line below `message`.
    pub notes: Vec<String>,
    /// Nested sub-exceptions for exception group / multi-error display.
    ///
    /// When non-empty, each sub-exception is rendered as its own nested
    /// `Panel` appended after the outer panel.
    pub sub_exceptions: Vec<Traceback>,
    /// Override the width used for the syntax-highlighted code block.
    ///
    /// When `None` (the default), the code width is derived from `panel_width - 4`.
    pub code_width: Option<usize>,
}

// ---------------------------------------------------------------------------
// PanicHookConfig
// ---------------------------------------------------------------------------

/// Configuration for [`Traceback::install_panic_hook_with_config`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PanicHookConfig {
    /// Path-prefix substrings used to suppress matching frames.
    pub suppress_paths: Vec<String>,
    /// Fixed output width (overrides terminal width when `Some`).
    pub width: Option<usize>,
    /// Number of context lines shown around the highlighted source line.
    pub extra_lines: usize,
    /// Syntax highlighting theme name.
    pub theme: String,
    /// Whether to display local variables for each frame.
    pub show_locals: bool,
    /// Maximum number of frames to display.
    pub max_frames: usize,
    /// Whether to word-wrap long source lines.
    pub word_wrap: bool,
}

impl Default for PanicHookConfig {
    fn default() -> Self {
        PanicHookConfig {
            suppress_paths: Vec::new(),
            width: None,
            extra_lines: 3,
            theme: "base16-ocean.dark".to_string(),
            show_locals: false,
            max_frames: 100,
            word_wrap: false,
        }
    }
}

impl Traceback {
    /// Create a new empty traceback with default settings.
    pub fn new() -> Self {
        Traceback {
            title: String::new(),
            message: String::new(),
            frames: Vec::new(),
            show_locals: false,
            width: None,
            extra_lines: 3,
            theme: "base16-ocean.dark".to_string(),
            // Finding #13: rich defaults word_wrap to false; was incorrectly true.
            word_wrap: false,
            max_frames: 100,
            suppress_paths: Vec::new(),
            notes: Vec::new(),
            sub_exceptions: Vec::new(),
            code_width: None,
        }
    }

    // -- Constructors -------------------------------------------------------

    /// Parse a `std::backtrace::Backtrace` string into a `Traceback`.
    ///
    /// Expects the format produced by `Backtrace::force_capture().to_string()`:
    /// ```text
    ///    0: std::backtrace::Backtrace::force_capture
    ///              at /rustc/.../backtrace.rs:331:18
    ///    1: myapp::main
    ///              at ./src/main.rs:10:5
    /// ```
    pub fn from_backtrace(bt: &str) -> Self {
        let frames = parse_backtrace(bt);
        Traceback {
            title: "Backtrace".to_string(),
            message: String::new(),
            frames,
            ..Traceback::new()
        }
    }

    /// Create a `Traceback` from an error chain.
    ///
    /// Walks the chain via `.source()` to collect all nested errors. The
    /// outermost error becomes the title, and nested errors are appended
    /// to the message.
    pub fn from_error(error: &dyn std::error::Error) -> Self {
        let title = format!("{}", error);
        let mut chain_messages: Vec<String> = Vec::new();
        let mut current = error.source();
        while let Some(cause) = current {
            chain_messages.push(format!("{}", cause));
            current = cause.source();
        }
        let message = if chain_messages.is_empty() {
            String::new()
        } else {
            format!("Caused by:\n  {}", chain_messages.join("\n  "))
        };
        Traceback {
            title: error_type_name(error),
            message: format!(
                "{}{}{}",
                title,
                if message.is_empty() { "" } else { "\n" },
                message
            ),
            frames: Vec::new(),
            ..Traceback::new()
        }
    }

    /// Create a `Traceback` from a panic message and a backtrace string.
    pub fn from_panic(message: &str, backtrace: &str) -> Self {
        let frames = parse_backtrace(backtrace);
        Traceback {
            title: "Panic".to_string(),
            message: message.to_string(),
            frames,
            ..Traceback::new()
        }
    }

    // -- Builder methods ----------------------------------------------------

    /// Set the title.
    #[must_use]
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// Set the error message.
    #[must_use]
    pub fn with_message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    /// Set whether to show locals (reserved for future use).
    #[must_use]
    pub fn with_show_locals(mut self, show: bool) -> Self {
        self.show_locals = show;
        self
    }

    /// Set a fixed width.
    #[must_use]
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the number of extra context lines around the source line.
    #[must_use]
    pub fn with_extra_lines(mut self, lines: usize) -> Self {
        self.extra_lines = lines;
        self
    }

    /// Set the syntax highlighting theme.
    #[must_use]
    pub fn with_theme(mut self, theme: &str) -> Self {
        self.theme = theme.to_string();
        self
    }

    /// Set whether to word-wrap source code.
    #[must_use]
    pub fn with_word_wrap(mut self, wrap: bool) -> Self {
        self.word_wrap = wrap;
        self
    }

    /// Set the maximum number of frames to display.
    #[must_use]
    pub fn with_max_frames(mut self, max: usize) -> Self {
        self.max_frames = max;
        self
    }

    /// Set path-prefix substrings that should suppress matching frames.
    ///
    /// Any frame whose `filename` contains at least one of the supplied
    /// strings is hidden from the rendered output.  This mirrors Rich's
    /// `Traceback(suppress=[click, requests])`.
    ///
    /// If suppression hides *every* frame a one-line placeholder is rendered
    /// so the user knows frames were omitted.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::error::traceback::Traceback;
    ///
    /// let tb = Traceback::new()
    ///     .with_suppress(vec![
    ///         "/.cargo/registry/src/".to_string(),
    ///         "tokio-".to_string(),
    ///     ]);
    /// assert_eq!(tb.suppress_paths.len(), 2);
    /// ```
    #[must_use]
    pub fn with_suppress(mut self, paths: Vec<String>) -> Self {
        self.suppress_paths = paths;
        self
    }

    /// Set PEP 678-style notes shown after the error message.
    #[must_use]
    pub fn with_notes(mut self, notes: Vec<String>) -> Self {
        self.notes = notes;
        self
    }

    /// Set nested sub-exceptions for exception-group display.
    #[must_use]
    pub fn with_sub_exceptions(mut self, subs: Vec<Traceback>) -> Self {
        self.sub_exceptions = subs;
        self
    }

    /// Override the width used for the syntax-highlighted code block.
    #[must_use]
    pub fn with_code_width(mut self, width: usize) -> Self {
        self.code_width = Some(width);
        self
    }

    // -- Helper: apply suppress filter + truncation -------------------------

    /// Return the frames that survive the suppress-path filter.
    ///
    /// The filter is applied first; `max_frames` truncation is left to the
    /// individual renderers so they can decide how to split the window.
    ///
    /// Finding #12: suppression uses prefix `starts_with` to mirror rich
    /// behaviour (rich uses `str.startswith`).
    #[cfg(test)]
    fn visible_frames(&self) -> Vec<&Frame> {
        if self.suppress_paths.is_empty() {
            return self.frames.iter().collect();
        }
        self.frames
            .iter()
            .filter(|f| {
                !self
                    .suppress_paths
                    .iter()
                    .any(|p| f.filename.starts_with(p.as_str()))
            })
            .collect()
    }

    // -- Public: panic hook installation ------------------------------------

    /// Install a `std::panic::set_hook` that prints a formatted `Traceback`
    /// to **stderr** whenever the process panics.
    ///
    /// Uses `std::backtrace::Backtrace::force_capture()` so the backtrace is
    /// always available regardless of the `RUST_BACKTRACE` environment variable.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gilt::error::traceback::Traceback;
    ///
    /// Traceback::install_panic_hook();
    /// ```
    pub fn install_panic_hook() {
        Self::install_panic_hook_with_config(PanicHookConfig::default());
    }

    /// Like [`install_panic_hook`](Self::install_panic_hook) but also applies
    /// path-prefix suppression to the captured backtrace.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gilt::error::traceback::Traceback;
    ///
    /// Traceback::install_panic_hook_with(vec!["/.cargo/registry/src/".to_string()]);
    /// ```
    pub fn install_panic_hook_with(suppress_paths: Vec<String>) {
        Self::install_panic_hook_with_config(PanicHookConfig {
            suppress_paths,
            ..PanicHookConfig::default()
        });
    }

    /// Like [`install_panic_hook`](Self::install_panic_hook) but accepts a
    /// full [`PanicHookConfig`] to control all rendering options.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gilt::error::traceback::{Traceback, PanicHookConfig};
    ///
    /// Traceback::install_panic_hook_with_config(PanicHookConfig {
    ///     suppress_paths: vec!["/.cargo/registry/src/".to_string()],
    ///     max_frames: 50,
    ///     ..PanicHookConfig::default()
    /// });
    /// ```
    pub fn install_panic_hook_with_config(config: PanicHookConfig) {
        std::panic::set_hook(Box::new(move |info| {
            // -- Extract panic message ---
            let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "Box<dyn Any>".to_string()
            };

            // -- Append location if available ---
            let full_message = if let Some(loc) = info.location() {
                format!("{} ({}:{})", message, loc.file(), loc.line())
            } else {
                message
            };

            // -- Capture the backtrace ---
            let bt = std::backtrace::Backtrace::force_capture();
            let bt_str = bt.to_string();

            // -- Build the Traceback applying all config fields ---
            let mut tb = Traceback::from_panic(&full_message, &bt_str)
                .with_suppress(config.suppress_paths.clone())
                .with_extra_lines(config.extra_lines)
                .with_theme(&config.theme)
                .with_show_locals(config.show_locals)
                .with_max_frames(config.max_frames)
                .with_word_wrap(config.word_wrap);
            if let Some(w) = config.width {
                tb = tb.with_width(w);
            }

            // -- Render to a capture buffer then write to stderr ---
            let mut console = Console::builder().no_color(false).build();
            console.begin_capture();
            console.print(&tb);
            let rendered = console.end_capture();

            use std::io::Write as _;
            let _ = std::io::stderr().write_all(rendered.as_bytes());
            let _ = std::io::stderr().flush();
        }));
    }

    // -- Internal rendering -------------------------------------------------

    /// Build the inner content `Text` that goes inside the Panel.
    ///
    /// This produces a simple text-only representation of the frames. The
    /// full `Renderable` implementation adds syntax highlighting on top.
    #[cfg(test)]
    fn render_content(&self) -> Text {
        let mut parts: Vec<TextPart> = Vec::new();

        // Apply suppress filter first, then truncate.
        let visible: Vec<&Frame> = self.visible_frames();

        // If we had frames but all were suppressed, emit a placeholder.
        if !self.frames.is_empty() && visible.is_empty() {
            let n = self.frames.len();
            let msg = format!(
                "[suppressed {} frame{}]\n",
                n,
                if n == 1 { "" } else { "s" }
            );
            parts.push(TextPart::Styled(msg, Style::parse("dim italic")));
            return Text::assemble(&parts, Style::null());
        }

        // Determine how many frames to show.
        // Finding #10: max_frames == 0 disables truncation (shows all frames).
        let frame_count = visible.len();
        let truncated = self.max_frames > 0 && frame_count > self.max_frames;
        let show_count = if truncated {
            self.max_frames
        } else {
            frame_count
        };

        // Collect frame indices to display
        let indices: Vec<usize> = if truncated {
            let half = self.max_frames / 2;
            let mut idx: Vec<usize> = (0..half).collect();
            idx.extend(frame_count - half..frame_count);
            idx
        } else {
            (0..frame_count).collect()
        };

        let mut inserted_ellipsis = false;

        for (pos, &frame_idx) in indices.iter().enumerate() {
            // Insert the ellipsis marker at the split point.
            // Finding #17: wording changed to "... N frames hidden ..." to match rich.
            if truncated && !inserted_ellipsis && frame_idx >= self.max_frames / 2 {
                inserted_ellipsis = true;
                let omitted = frame_count - show_count;
                let msg = format!("\n  ... {} frames hidden ...\n", omitted);
                parts.push(TextPart::Styled(msg, Style::parse("dim italic")));
            }

            let frame = visible[frame_idx];

            // File location line
            let location = match frame.lineno {
                Some(n) => format!("{}:{}", frame.filename, n),
                None => frame.filename.clone(),
            };

            parts.push(TextPart::Styled(
                format!("  File \"{}\"", location),
                Style::parse("green"),
            ));
            parts.push(TextPart::Styled(
                format!(", in {}", frame.name),
                Style::parse("magenta"),
            ));
            parts.push(TextPart::Raw("\n".to_string()));

            // Source line if available
            if let Some(ref source) = frame.source_line {
                let trimmed = source.trim();
                if !trimmed.is_empty() {
                    parts.push(TextPart::Raw(format!("    {}", trimmed)));
                    parts.push(TextPart::Raw("\n".to_string()));
                }
            }

            // Locals — rendered only when show_locals is true and the frame carries
            // at least one local variable pair.
            if self.show_locals && !frame.locals.is_empty() {
                let scope_pairs: Vec<(&str, &str)> = frame
                    .locals
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect();
                let scope = Scope::from_pairs(&scope_pairs).title("locals");
                // Render scope to plain text and fold it into the content.
                let scope_text = format!("{:80}", scope);
                if !scope_text.is_empty() {
                    parts.push(TextPart::Raw(scope_text));
                    parts.push(TextPart::Raw("\n".to_string()));
                }
            }

            // Add a blank line between frames (except after the last one)
            if pos + 1 < indices.len() {
                parts.push(TextPart::Raw("\n".to_string()));
            }
        }

        Text::assemble(&parts, Style::null())
    }
}

impl Default for Traceback {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Traceback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.title.is_empty() {
            writeln!(f, "{}", self.title)?;
        }
        // Honour suppress_paths so plain Display matches Renderable behaviour.
        // Finding #12: use starts_with (prefix match) to mirror rich behaviour.
        let suppressed: Vec<&Frame> = if self.suppress_paths.is_empty() {
            self.frames.iter().collect()
        } else {
            self.frames
                .iter()
                .filter(|fr| {
                    !self
                        .suppress_paths
                        .iter()
                        .any(|p| fr.filename.starts_with(p.as_str()))
                })
                .collect()
        };
        if !self.frames.is_empty() && suppressed.is_empty() {
            let n = self.frames.len();
            writeln!(
                f,
                "[suppressed {} frame{}]",
                n,
                if n == 1 { "" } else { "s" }
            )?;
        } else {
            for frame in &suppressed {
                writeln!(f, "{}", frame)?;
            }
        }
        if !self.message.is_empty() {
            write!(f, "{}", self.message)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Renderable
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Per-render style statics (finding #20: hoist Style::parse out of per-frame loop)
// ---------------------------------------------------------------------------

static FRAME_FILE_STYLE: LazyLock<Style> = LazyLock::new(|| Style::parse("green"));
static FRAME_FUNC_STYLE: LazyLock<Style> = LazyLock::new(|| Style::parse("magenta"));
static FRAME_DIM_STYLE: LazyLock<Style> = LazyLock::new(|| Style::parse("dim italic"));
static FRAME_BOLD_STYLE: LazyLock<Style> = LazyLock::new(|| Style::parse("bold"));

impl Renderable for Traceback {
    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        #[cfg(feature = "syntax")]
        let panel_width = self.width.unwrap_or(options.max_width);

        // Build the inner content
        let mut content_parts: Vec<TextPart> = Vec::new();

        // Apply suppress filter before counting & truncating.
        // Finding #12: use starts_with (prefix match) to mirror rich behaviour.
        let visible_frames: Vec<&Frame> = if self.suppress_paths.is_empty() {
            self.frames.iter().collect()
        } else {
            self.frames
                .iter()
                .filter(|f| {
                    !self
                        .suppress_paths
                        .iter()
                        .any(|p| f.filename.starts_with(p.as_str()))
                })
                .collect()
        };

        // Special-case: had frames but suppress hid them all → emit a placeholder.
        if !self.frames.is_empty() && visible_frames.is_empty() {
            let n = self.frames.len();
            let msg = format!(
                "[suppressed {} frame{}]\n",
                n,
                if n == 1 { "" } else { "s" }
            );
            content_parts.push(TextPart::Styled(msg, FRAME_DIM_STYLE.clone()));
            let content = Text::assemble(&content_parts, Style::null());
            let panel = Panel::new(content).with_title(self.title.clone());
            return panel.gilt_console(console, options);
        }

        let frame_count = visible_frames.len();
        // Finding #10: max_frames == 0 disables truncation (show all frames).
        let truncated = self.max_frames > 0 && frame_count > self.max_frames;
        let show_count = if truncated {
            self.max_frames
        } else {
            frame_count
        };

        let frames_to_show: Vec<&Frame> = if truncated {
            let half = self.max_frames / 2;
            let mut combined: Vec<&Frame> = visible_frames.iter().take(half).copied().collect();
            combined.extend(visible_frames.iter().skip(frame_count - half).copied());
            combined
        } else {
            visible_frames.clone()
        };

        let actual_show = frames_to_show.len();
        let half_mark = if truncated {
            self.max_frames / 2
        } else {
            actual_show + 1
        };

        // Finding #21: cache source files so a path referenced by multiple
        // frames (recursion, repeated module) is read at most once per render.
        #[cfg(feature = "syntax")]
        let mut file_cache: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        for (i, frame) in frames_to_show.iter().enumerate() {
            // Insert ellipsis marker at the halfway point for truncated traces.
            // Finding #17: wording changed to "... N frames hidden ..." to match rich.
            if truncated && i == half_mark {
                let omitted = frame_count - show_count;
                let msg = format!("\n... {} frames hidden ...\n\n", omitted);
                content_parts.push(TextPart::Styled(msg, FRAME_DIM_STYLE.clone()));
            }

            // File location line — always shown (finding #11: header always emitted).
            let location = match frame.lineno {
                Some(n) => format!("{}:{}", frame.filename, n),
                None => frame.filename.clone(),
            };

            content_parts.push(TextPart::Styled(
                format!("File \"{}\"", location),
                FRAME_FILE_STYLE.clone(),
            ));
            content_parts.push(TextPart::Styled(
                format!(", in {}", frame.name),
                FRAME_FUNC_STYLE.clone(),
            ));
            content_parts.push(TextPart::Raw("\n".to_string()));

            // Finding #11: for suppressed paths, emit the header (above) but
            // skip the source snippet block (below). The frame header is already
            // pushed unconditionally; we only skip source for suppressed paths.
            let is_suppressed = !self.suppress_paths.is_empty()
                && self
                    .suppress_paths
                    .iter()
                    .any(|p| frame.filename.starts_with(p.as_str()));

            if !is_suppressed {
                // Source context: try to read the file and show context lines.
                // Finding #9: carry seg.style so highlighted source is colored.
                #[allow(unused_mut)]
                let mut showed_syntax = false;

                // Save syntax segments separately so Item 5 can use them for
                // side-by-side Columns layout when show_locals is also true.
                #[cfg(feature = "syntax")]
                let mut saved_syntax_segs: Vec<Segment> = Vec::new();

                #[cfg(feature = "syntax")]
                if let Some(lineno) = frame.lineno {
                    if lineno > 0 {
                        let path = std::path::Path::new(&frame.filename);
                        if (path.is_absolute() || frame.filename.starts_with("./")) && path.exists()
                        {
                            let file_contents =
                                file_cache.entry(frame.filename.clone()).or_insert_with(|| {
                                    std::fs::read_to_string(path).unwrap_or_default()
                                });
                            {
                                let total_lines = file_contents.lines().count();
                                if lineno <= total_lines {
                                    let start = lineno.saturating_sub(self.extra_lines).max(1);
                                    let end = (lineno + self.extra_lines).min(total_lines);

                                    let context: String = file_contents
                                        .lines()
                                        .enumerate()
                                        .filter(|(idx, _)| {
                                            let n = idx + 1;
                                            n >= start && n <= end
                                        })
                                        .map(|(_, l)| l)
                                        .collect::<Vec<_>>()
                                        .join("\n");

                                    let ext =
                                        path.extension().and_then(|e| e.to_str()).unwrap_or("txt");

                                    let syntax = Syntax::new(&context, ext)
                                        .with_theme(&self.theme)
                                        .with_line_numbers(true)
                                        .with_start_line(start)
                                        .with_highlight_lines(vec![lineno])
                                        .with_word_wrap(self.word_wrap);

                                    // Item 4: use code_width override when set.
                                    let syntax_segments = syntax.gilt_console(
                                        console,
                                        &options.update_width(
                                            self.code_width
                                                .unwrap_or(panel_width.saturating_sub(4)),
                                        ),
                                    );
                                    if !syntax_segments.is_empty() {
                                        saved_syntax_segs = syntax_segments;
                                        showed_syntax = true;
                                    }
                                }
                            }
                        }
                    }
                }

                // Item 5: side-by-side Columns when syntax was shown AND locals exist.
                #[cfg(feature = "syntax")]
                let did_side_by_side = if showed_syntax
                    && self.show_locals
                    && !frame.locals.is_empty()
                {
                    use crate::columns::Columns;

                    // Build a Text from the syntax segments.
                    let mut syn_parts: Vec<TextPart> = Vec::new();
                    for seg in &saved_syntax_segs {
                        match seg.style() {
                            Some(s) if !s.is_null() => {
                                syn_parts.push(TextPart::Styled(seg.text.to_string(), s.clone()));
                            }
                            _ => {
                                syn_parts.push(TextPart::Raw(seg.text.to_string()));
                            }
                        }
                    }
                    let syntax_text = Text::assemble(&syn_parts, Style::null());

                    // Build a Text from the locals Scope.
                    let scope_pairs: Vec<(&str, &str)> = frame
                        .locals
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_str()))
                        .collect();
                    let scope = Scope::from_pairs(&scope_pairs).title("locals");
                    let scope_segments = scope.gilt_console(console, options);
                    let mut loc_parts: Vec<TextPart> = Vec::new();
                    for seg in &scope_segments {
                        match seg.style() {
                            Some(s) if !s.is_null() => {
                                loc_parts.push(TextPart::Styled(seg.text.to_string(), s.clone()));
                            }
                            _ => {
                                loc_parts.push(TextPart::Raw(seg.text.to_string()));
                            }
                        }
                    }
                    let locals_text = Text::assemble(&loc_parts, Style::null());

                    // Render the two columns side by side.
                    let cols = Columns::from_renderables([syntax_text, locals_text]);
                    let col_segs = cols.gilt_console(console, options);
                    for seg in &col_segs {
                        match seg.style() {
                            Some(s) if !s.is_null() => {
                                content_parts
                                    .push(TextPart::Styled(seg.text.to_string(), s.clone()));
                            }
                            _ => {
                                content_parts.push(TextPart::Raw(seg.text.to_string()));
                            }
                        }
                    }
                    true
                } else {
                    // Not side-by-side: append syntax normally.
                    if showed_syntax {
                        // Finding #9: carry seg.style alongside text so
                        // syntax-highlighted output is colored correctly.
                        for seg in &saved_syntax_segs {
                            match seg.style() {
                                Some(s) if !s.is_null() => {
                                    content_parts
                                        .push(TextPart::Styled(seg.text.to_string(), s.clone()));
                                }
                                _ => {
                                    content_parts.push(TextPart::Raw(seg.text.to_string()));
                                }
                            }
                        }
                    }
                    false
                };

                // Non-syntax build: no side-by-side tracking needed.
                #[cfg(not(feature = "syntax"))]
                let did_side_by_side = false;

                // Fallback: show the single source line if we didn't render syntax
                if !showed_syntax {
                    if let Some(ref source) = frame.source_line {
                        let trimmed = source.trim();
                        if !trimmed.is_empty() {
                            content_parts.push(TextPart::Raw(format!("    {}\n", trimmed)));
                        }
                    }
                }

                // Locals — rendered when show_locals is true and the frame has locals,
                // but only when NOT already rendered side-by-side above.
                if self.show_locals && !frame.locals.is_empty() && !did_side_by_side {
                    let scope_pairs: Vec<(&str, &str)> = frame
                        .locals
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_str()))
                        .collect();
                    let scope = Scope::from_pairs(&scope_pairs).title("locals");
                    let scope_segments = scope.gilt_console(console, options);
                    for seg in &scope_segments {
                        match seg.style() {
                            Some(s) if !s.is_null() => {
                                content_parts
                                    .push(TextPart::Styled(seg.text.to_string(), s.clone()));
                            }
                            _ => {
                                content_parts.push(TextPart::Raw(seg.text.to_string()));
                            }
                        }
                    }
                }
            }

            // Blank line between frames
            if i + 1 < actual_show {
                content_parts.push(TextPart::Raw("\n".to_string()));
            }
        }

        // Error message at the bottom
        if !self.message.is_empty() {
            content_parts.push(TextPart::Raw("\n".to_string()));
            content_parts.push(TextPart::Styled(
                self.message.clone(),
                FRAME_BOLD_STYLE.clone(),
            ));
        }

        // Item 1 (PEP 678): render notes after the message.
        if !self.notes.is_empty() {
            let note_style = console
                .get_style("traceback.note")
                .unwrap_or_else(|_| Style::parse("italic"));
            for note in &self.notes {
                content_parts.push(TextPart::Raw("\n".to_string()));
                content_parts.push(TextPart::Styled(note.clone(), note_style.clone()));
            }
        }

        let content_text = Text::assemble(&content_parts, Style::null());

        // Wrap in a Panel
        let title_text = if self.title.is_empty() {
            Text::styled("Traceback", "bold red")
        } else {
            Text::styled(&self.title, "bold red")
        };

        let panel = Panel::new(content_text)
            .with_title(title_text)
            .with_border_style(Style::parse("red"))
            .with_expand(true);

        let panel_opts = if let Some(w) = self.width {
            options.update_width(w)
        } else {
            options.clone()
        };

        let mut result = panel.gilt_console(console, &panel_opts);

        // Item 2 (ExceptionGroup): render each sub-exception appended after the main panel.
        // Each sub already renders as its own red-bordered Panel (via its own gilt_console call),
        // so we only prepend a yellow "Exception N of M:" label — no extra Panel wrapper.
        if !self.sub_exceptions.is_empty() {
            let total = self.sub_exceptions.len();
            for (idx, sub) in self.sub_exceptions.iter().enumerate() {
                // Prepend a styled label line: "Exception N of M:"
                let label = format!("Exception {} of {}:", idx + 1, total);
                let label_text = Text::styled(label, "bold yellow");
                let label_segs = label_text.gilt_console(console, &panel_opts);
                result.extend(label_segs);
                // Append a newline segment after the label.
                result.push(Segment::new("\n", None, None));
                // Render the sub-exception directly (it produces its own Panel).
                let sub_segs = sub.gilt_console(console, &panel_opts);
                result.extend(sub_segs);
            }
        }

        result
    }
}

// ---------------------------------------------------------------------------
// Backtrace parsing
// ---------------------------------------------------------------------------

/// Parse a Rust backtrace string into a Vec of Frames.
///
/// Handles the standard Rust backtrace format:
/// ```text
///    0: rust_begin_unwind
///              at /rustc/hash/library/std/src/panicking.rs:652:5
///    1: core::panicking::panic_fmt
///              at /rustc/hash/library/core/src/panicking.rs:72:14
///    2: myapp::myfunction
///              at ./src/main.rs:42:9
/// ```
fn parse_backtrace(bt: &str) -> Vec<Frame> {
    static FRAME_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?m)^\s*(\d+):\s+(.+?)$").expect("invalid frame regex"));
    static LOCATION_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?m)^\s+at\s+(.+?):(\d+)(?::(\d+))?\s*$").expect("invalid location regex")
    });

    let frame_re = &*FRAME_RE;
    let location_re = &*LOCATION_RE;
    let lines: Vec<&str> = bt.lines().collect();
    let mut frames = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        if let Some(captures) = frame_re.captures(line) {
            let name = captures
                .get(2)
                .map(|m| m.as_str())
                .unwrap_or("")
                .trim()
                .to_string();

            // Check if the next line has location info
            let mut filename = String::new();
            let mut lineno = None;

            if i + 1 < lines.len() {
                if let Some(loc_captures) = location_re.captures(lines[i + 1]) {
                    filename = loc_captures
                        .get(1)
                        .map(|m| m.as_str())
                        .unwrap_or("")
                        .to_string();
                    lineno = loc_captures
                        .get(2)
                        .and_then(|m| m.as_str().parse::<usize>().ok());
                    i += 1; // consume the location line
                }
            }

            let mut frame = Frame::new(&filename, lineno, &name);
            frame.read_source_line();
            frames.push(frame);
        }
        i += 1;
    }

    frames
}

/// Extract a short type name from an error reference.
///
/// Since Rust does not have built-in runtime type names for trait objects, we
/// use a simple heuristic based on the Debug output. For well-known error
/// types, this produces a reasonable label.
fn error_type_name(error: &dyn std::error::Error) -> String {
    let debug = format!("{:?}", error);
    // Try to extract a type name from the Debug output.
    // Many errors format as `TypeName { ... }` or `TypeName(...)`.
    if let Some(paren) = debug.find('(') {
        let brace = debug.find('{').unwrap_or(debug.len());
        let end = paren.min(brace);
        let name = debug[..end].trim();
        if !name.is_empty() && !name.contains(' ') {
            return name.to_string();
        }
    } else if let Some(brace) = debug.find('{') {
        let name = debug[..brace].trim();
        if !name.is_empty() && !name.contains(' ') {
            return name.to_string();
        }
    }
    "Error".to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "traceback_tests.rs"]
mod tests;
