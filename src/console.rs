//! Console engine — the central orchestrator of gilt rendering output.
//!
//! The Console manages terminal capabilities, drives the rendering pipeline,
//! and handles output buffering, capture, and export.

use crate::cells::cell_len;
use crate::color::ColorSystem;
use crate::color_env::{detect_color_env, ColorEnvOverride};
use crate::console_caps::ConsoleCapabilities;
use crate::control::Control;
use crate::error::ConsoleError;
use crate::measure::Measurement;
use crate::pager::Pager;
use crate::segment::Segment;
use crate::style::Style;
use crate::style_interner::StyleInterner;
use crate::text::{JustifyMethod, OverflowMethod, Text};
use crate::theme::{Theme, ThemeStack};
use std::borrow::Cow;
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// Color-system auto-detection helper (P1 parity, finding #1)
// ---------------------------------------------------------------------------

/// Detect the color system from `COLORTERM` and `TERM` environment values,
/// following rich's detection order:
///
/// 1. `COLORTERM` contains `truecolor` or `24bit` → `TrueColor`
/// 2. `TERM` ends with `256color` → `EightBit`
/// 3. `TERM` is set (and not `dumb`) → `Standard`
/// 4. Otherwise → `TrueColor` (caller-level fallback; no-TTY callers use `None`)
///
/// This is a pure helper that takes string slices so it can be unit-tested
/// without mutating the process environment.
pub fn detect_color_system_from(colorterm: Option<&str>, term: Option<&str>) -> ColorSystem {
    // Step 1: COLORTERM truecolor / 24bit
    if let Some(ct) = colorterm {
        let ct_lower = ct.to_lowercase();
        if ct_lower.contains("truecolor") || ct_lower.contains("24bit") {
            return ColorSystem::TrueColor;
        }
    }
    // Step 2: TERM ends with 256color
    if let Some(t) = term {
        if t.ends_with("256color") {
            return ColorSystem::EightBit;
        }
        // Step 3: TERM is set and not dumb
        if !t.is_empty() && t != "dumb" {
            return ColorSystem::Standard;
        }
    }
    // Step 4: no meaningful terminal signal → fall back to TrueColor
    // (the caller decides whether to use None for no-TTY situations)
    ColorSystem::TrueColor
}

/// Detect whether the host terminal advertises itself as a modern Windows
/// terminal that supports ANSI / VT escape codes (and therefore does not
/// need the legacy Windows renderer).
///
/// The detection mirrors the existing rule in
/// [`crate::utils::diagnose`] and rich's own logic: a terminal is treated
/// as "modern" when any of these environment variables is set to a
/// non-empty value:
///
/// - `WT_SESSION`     — set by Windows Terminal when launched in the
///   default profile mode.
/// - `WT_PROFILE_ID`  — set by Windows Terminal when launched with a
///   non-default profile (some hosts set only this one).
/// - `TERM_PROGRAM`   — set by VS Code, Apple Terminal, iTerm, and many
///   other modern terminal emulators that may be hosting the process.
///
/// This is a pure helper (takes string slices) so the logic can be
/// unit-tested without mutating the process environment. Empty strings
/// are NOT treated as set — `Option<&str>` differentiates "variable
/// unset" from "variable set to empty".
pub fn detect_modern_windows_terminal(
    wt_session: Option<&str>,
    wt_profile_id: Option<&str>,
    term_program: Option<&str>,
) -> bool {
    let is_set = |v: Option<&str>| v.map(|s| !s.is_empty()).unwrap_or(false);
    is_set(wt_session) || is_set(wt_profile_id) || is_set(term_program)
}

// ---------------------------------------------------------------------------
// Terminal detection helper
// ---------------------------------------------------------------------------

/// Detect whether stdout is connected to a terminal.
///
/// On native (non-wasm) targets this uses [`std::io::IsTerminal`] for an
/// accurate answer even when stdout is piped. On wasm targets — where
/// `IsTerminal` is not available — we fall back to checking that `TERM` is
/// set and not `"dumb"`.
fn detect_is_terminal() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::io::IsTerminal as _;
        std::io::stdout().is_terminal()
    }
    #[cfg(target_arch = "wasm32")]
    {
        matches!(
            std::env::var("TERM").as_deref(),
            Ok(t) if !t.is_empty() && t != "dumb"
        )
    }
}

/// Detect whether the host is a "legacy" Windows console (one without
/// VT/ANSI escape-code support).
///
/// On Windows targets with native (non-wasm) builds this returns `true`
/// when none of the modern-terminal environment variables
/// (`WT_SESSION`, `WT_PROFILE_ID`, `TERM_PROGRAM`) are set. On every
/// other target — non-Windows OR wasm32 — it returns `false`, matching
/// gilt's WASM-safe contract.
///
/// The env reads only execute inside the Windows+native cfg-branch; on
/// every other target the function short-circuits to `false`, so wasm
/// builds never touch `std::env`.
pub(crate) fn detect_legacy_windows() -> bool {
    if cfg!(all(target_os = "windows", not(target_arch = "wasm32"))) {
        !detect_modern_windows_terminal(
            std::env::var("WT_SESSION").ok().as_deref(),
            std::env::var("WT_PROFILE_ID").ok().as_deref(),
            std::env::var("TERM_PROGRAM").ok().as_deref(),
        )
    } else {
        false
    }
}

// ---------------------------------------------------------------------------
// ConsoleDimensions
// ---------------------------------------------------------------------------

/// Terminal dimensions in columns and rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsoleDimensions {
    /// Number of columns.
    pub width: usize,
    /// Number of rows.
    pub height: usize,
}

// ---------------------------------------------------------------------------
// ConsoleOptions
// ---------------------------------------------------------------------------

/// Options that control how renderables produce segments.
#[derive(Debug, Clone)]
pub struct ConsoleOptions {
    /// Terminal dimensions used for layout.
    pub size: ConsoleDimensions,
    /// Whether to use legacy Windows console rendering.
    pub legacy_windows: bool,
    /// Minimum width in columns for renderable output.
    pub min_width: usize,
    /// Maximum width in columns for renderable output.
    pub max_width: usize,
    /// Whether the output target is an interactive terminal.
    pub is_terminal: bool,
    /// Character encoding (always `"utf-8"` in Rust; `Cow` avoids allocation
    /// per `options()` call while still letting tests set non-utf encodings
    /// for `ascii_only()` checks — finding #6).
    pub encoding: Cow<'static, str>,
    /// Maximum height in rows for renderable output.
    pub max_height: usize,
    /// Text justification override, if any.
    pub justify: Option<JustifyMethod>,
    /// Text overflow strategy override, if any.
    pub overflow: Option<OverflowMethod>,
    /// Whether to disable text wrapping.
    ///
    /// Tri-state (rich parity):
    /// - `None`        = inherit / wrap by default
    /// - `Some(false)` = force-wrap (explicit wrap)
    /// - `Some(true)`  = no-wrap (suppress wrapping)
    ///
    /// Only `Some(true)` suppresses wrapping; `None` and `Some(false)` both wrap.
    pub no_wrap: Option<bool>,
    /// Whether to enable syntax highlighting, if set.
    pub highlight: Option<bool>,
    /// Whether to enable markup parsing, if set.
    pub markup: Option<bool>,
    /// Explicit height constraint for renderables, if set.
    pub height: Option<usize>,
}

/// Builder for applying selective updates to `ConsoleOptions`.
#[derive(Debug, Clone, Default)]
pub struct ConsoleOptionsUpdates {
    /// New width in columns, if changing.
    pub width: Option<usize>,
    /// New minimum width, if changing.
    pub min_width: Option<usize>,
    /// New maximum width, if changing.
    pub max_width: Option<usize>,
    /// New justification override, if changing.
    pub justify: Option<Option<JustifyMethod>>,
    /// New overflow strategy override, if changing.
    pub overflow: Option<Option<OverflowMethod>>,
    /// New no-wrap flag, if changing.
    ///
    /// Triple-state:
    /// - `None`           = don't touch the field
    /// - `Some(None)`     = reset `ConsoleOptions::no_wrap` to `None`
    /// - `Some(Some(b))` = set `ConsoleOptions::no_wrap` to `Some(b)`
    pub no_wrap: Option<Option<bool>>,
    /// New highlight flag, if changing.
    pub highlight: Option<Option<bool>>,
    /// New markup flag, if changing.
    pub markup: Option<Option<bool>>,
    /// New height constraint, if changing.
    pub height: Option<Option<usize>>,
    /// New maximum height, if changing.
    pub max_height: Option<usize>,
}

impl ConsoleOptions {
    /// Returns `true` if the encoding is NOT utf-based (i.e. ASCII-only output).
    pub fn ascii_only(&self) -> bool {
        !self.encoding.to_lowercase().starts_with("utf")
    }

    /// Clone this options set.
    pub fn copy(&self) -> Self {
        self.clone()
    }

    /// Return a new `ConsoleOptions` with the width replaced.
    ///
    /// `min_width` is clamped so it never exceeds the new width (finding #3).
    pub fn update_width(&self, width: usize) -> Self {
        let mut opts = self.clone();
        opts.size.width = width;
        opts.max_width = width;
        // P1 parity: min_width must not exceed the new width.
        opts.min_width = opts.min_width.min(width);
        opts
    }

    /// Return a new `ConsoleOptions` with the height replaced.
    pub fn update_height(&self, height: usize) -> Self {
        let mut opts = self.clone();
        opts.height = Some(height);
        opts
    }

    /// Return a new `ConsoleOptions` with both width and height replaced.
    pub fn update_dimensions(&self, width: usize, height: usize) -> Self {
        let mut opts = self.clone();
        opts.size = ConsoleDimensions { width, height };
        opts.max_width = width;
        opts.height = Some(height);
        opts
    }

    /// Return a new `ConsoleOptions` with height reset to `None`.
    pub fn reset_height(&self) -> Self {
        let mut opts = self.clone();
        opts.height = None;
        opts
    }

    /// Apply a set of optional field updates, returning a new `ConsoleOptions`.
    pub fn with_updates(&self, updates: &ConsoleOptionsUpdates) -> Self {
        let mut opts = self.clone();
        if let Some(w) = updates.width {
            opts.size.width = w;
            opts.max_width = w;
        }
        if let Some(min_w) = updates.min_width {
            opts.min_width = min_w;
        }
        if let Some(max_w) = updates.max_width {
            opts.max_width = max_w;
        }
        if let Some(ref j) = updates.justify {
            opts.justify = *j;
        }
        if let Some(ref o) = updates.overflow {
            opts.overflow = *o;
        }
        if let Some(nw) = updates.no_wrap {
            // Triple-state: `Some(None)` resets to `None`; `Some(Some(b))` sets to `Some(b)`.
            opts.no_wrap = nw;
        }
        if let Some(ref h) = updates.highlight {
            opts.highlight = *h;
        }
        if let Some(ref m) = updates.markup {
            opts.markup = *m;
        }
        if let Some(ref h) = updates.height {
            opts.height = *h;
        }
        if let Some(mh) = updates.max_height {
            opts.max_height = mh;
        }
        opts
    }
}

// ---------------------------------------------------------------------------
// Renderable trait
// ---------------------------------------------------------------------------

/// Trait for objects that can produce `Segment`s for console rendering.
pub trait Renderable {
    /// Produce segments for rendering on the given console with given options.
    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment>;

    /// Return the minimum and maximum cell widths required to render this object.
    ///
    /// The default implementation performs a full render and measures the resulting
    /// segments — equivalent to the current `Console::measure` body.  Widgets with
    /// a cheaper measurement path should override this method.
    ///
    /// **Empty-output contract:** when the rendered output is empty this returns
    /// `Measurement::new(0, options.max_width)` rather than `(0, 0)`, matching
    /// rich's `__rich_measure__` protocol (an object that produces no output can
    /// still fill any available width).
    ///
    /// This method is additive and non-breaking: all existing `Renderable`
    /// implementations inherit this default without any source change.
    fn gilt_measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        let segments = self.gilt_console(console, options);
        // Collect all text, split by newlines to find line widths
        let full_text: String = segments
            .iter()
            .filter(|s| !s.is_control())
            .map(|s| s.text.as_str())
            .collect();
        if full_text.is_empty() {
            return Measurement::new(0, options.max_width);
        }
        let max_width = full_text.lines().map(cell_len).max().unwrap_or(0);
        let min_width = full_text
            .split_whitespace()
            .map(cell_len)
            .max()
            .unwrap_or(0);
        Measurement::new(min_width, max_width)
    }

    /// Return a cheap hash of this widget's content for per-region dirty tracking.
    ///
    /// `None` (the default) means "always dirty — re-render every frame".
    /// `Some(hash)` means "cache is valid as long as the hash matches the
    /// previous frame".  Only implement `Some(...)` where the hash is both
    /// cheap to compute and correct (i.e. it changes whenever the visible
    /// output would change).
    ///
    /// This method is additive and non-breaking: all existing `Renderable`
    /// implementations inherit the `None` default without any source change.
    fn content_hash(&self) -> Option<u64> {
        None
    }
}

impl Renderable for Text {
    /// Return a stable hash of this `Text`'s plain content.
    ///
    /// Uses a simple FNV-1a hash over the plain string bytes, mixed with
    /// a type-marker so `Text("x")` and `Rule` with the same display don't
    /// collide.  Cheap: O(n) in the plain-text length, no heap allocation.
    fn content_hash(&self) -> Option<u64> {
        const TEXT_MARKER: u64 = 0x0000_0001; // distinguishes Text from Rule

        let plain = self.plain();
        let mut h = crate::utils::hash::fnv1a_64(plain.as_bytes());
        // Mix in the type marker so Text("x") != Rule with display "x"
        h ^= TEXT_MARKER;
        h = h.wrapping_mul(crate::utils::hash::FNV_PRIME);
        Some(h)
    }

    fn gilt_measure(&self, _console: &Console, _options: &ConsoleOptions) -> Measurement {
        self.measure()
    }

    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let mut text = self.clone();
        if let Some(justify) = &options.justify {
            text.justify = Some(*justify);
        }
        if let Some(overflow) = &options.overflow {
            text.overflow = Some(*overflow);
        }
        if options.no_wrap == Some(true) || options.overflow == Some(OverflowMethod::Ignore) {
            text.render_themed(console)
        } else {
            let tab_size = text.tab_size.unwrap_or(8);
            let lines = text.wrap(
                console,
                options.max_width,
                text.justify,
                text.overflow,
                tab_size,
                text.no_wrap.unwrap_or(false),
            );
            let mut segments = Vec::new();
            for line in lines.iter() {
                // Each line's render_themed() already appends its `end` ("\n"),
                // so no extra Segment::line() is needed between lines.
                segments.extend(line.render_themed(console));
            }
            segments
        }
    }
}

impl Renderable for str {
    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let text = console.render_str(self, None, options.justify, options.overflow);
        text.gilt_console(console, options)
    }
}

impl Renderable for String {
    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        self.as_str().gilt_console(console, options)
    }
}

// ---------------------------------------------------------------------------
// RenderableArc — shared-ownership, type-erased renderable
// ---------------------------------------------------------------------------

/// Shared-ownership, type-erased renderable.  Cheap to clone (ref-counted).
///
/// This is the container/composition trait-object type used by [`Layout`] and
/// [`Live`] — naming it avoids repeating the full `Arc<dyn Renderable + Send +
/// Sync>` spelling throughout the codebase.
///
/// [`Layout`]: crate::layout::Layout
/// [`Live`]: crate::live
pub type RenderableArc = std::sync::Arc<dyn Renderable + Send + Sync>;

/// Wrap any [`Renderable`] into a [`RenderableArc`].
///
/// This is a convenience function that boxes any `Renderable + Send + Sync`
/// value into a reference-counted trait object.
///
/// # Example
///
/// ```rust
/// use gilt::console::{into_renderable_arc, Console};
/// use gilt::style::Style;
/// use gilt::text::Text;
///
/// let arc = into_renderable_arc(Text::new("hello", Style::null()));
/// let arc2 = arc.clone(); // cheap ref-count bump
/// let console = Console::new();
/// let opts = console.options();
/// assert!(!arc2.gilt_console(&console, &opts).is_empty());
/// ```
pub fn into_renderable_arc<R: Renderable + Send + Sync + 'static>(r: R) -> RenderableArc {
    std::sync::Arc::new(r)
}

// ConsoleBuilder moved to console_builder.rs (v1.2 Phase 3 split).
#[path = "console_builder.rs"]
mod console_builder;
pub use console_builder::ConsoleBuilder;

// Capture-mode methods (begin_capture / end_capture / render_widget_to_text)
// moved to console_capture.rs in v1.3 Phase 4 — methods stay on Console via
// a separate impl block.
#[path = "console_capture.rs"]
mod console_capture;
pub use console_capture::{CaptureGuard, ScreenGuard};

// Render path (render/print/log/rule/line/inspect/print_json/print_error/
// print_exception/write_segments/buffer ops) moved to console_render.rs in
// v1.3 Phase 5 — same multi-impl-block pattern.
#[path = "console_render.rs"]
mod console_render;
// Re-export the measurement-protocol free functions so they are accessible
// at `crate::console::measurement_get` and subsequently re-exported from
// `crate::measure`.
pub use console_render::{measure_renderables, measurement_get};

// ---------------------------------------------------------------------------
// ThemeGuard
// ---------------------------------------------------------------------------

/// RAII guard returned by [`Console::use_theme`].
///
/// Holds a mutable reference to the `Console` and pops the theme layer when
/// dropped, restoring the previous theme state automatically.
pub struct ThemeGuard<'a> {
    console: &'a mut Console,
}

impl Drop for ThemeGuard<'_> {
    fn drop(&mut self) {
        self.console.pop_theme();
    }
}

impl std::ops::Deref for ThemeGuard<'_> {
    type Target = Console;
    fn deref(&self) -> &Console {
        self.console
    }
}

impl std::ops::DerefMut for ThemeGuard<'_> {
    fn deref_mut(&mut self) -> &mut Console {
        self.console
    }
}

// ---------------------------------------------------------------------------
// Console
// ---------------------------------------------------------------------------

/// A hook that intercepts renderables just before they are printed.
///
/// Each hook receives the current list of renderables and may return a
/// modified list. Hooks are invoked in registration order by
/// [`Console::print_with_hooks`].
///
/// # Note
///
/// Hooks do **not** fire when calling [`Console::print`] directly, because
/// `print<R: ?Sized>` cannot coerce an unsized `&R` (e.g. `&str`) to
/// `&dyn Renderable` without specialization. Use
/// [`Console::print_with_hooks`] when you need hooks to run.
///
/// # WASM safety
///
/// The `Send + Sync` bounds are required so `Console` remains `Send + Sync`.
/// Implementations that carry no state (the common case) satisfy these trivially.
pub trait RenderHook: Send + Sync {
    /// Transform the list of renderables before they are rendered.
    ///
    /// The default implementation returns the list unchanged.
    fn process_renderables<'a>(
        &self,
        renderables: Vec<&'a dyn Renderable>,
    ) -> Vec<&'a dyn Renderable>;
}

/// The central orchestrator of gilt rendering output.
///
/// Console manages terminal capabilities, drives the rendering pipeline,
/// and handles output buffering, capture, and export.
pub struct Console {
    // Configuration
    color_system: Option<ColorSystem>,
    width_override: Option<usize>,
    height_override: Option<usize>,
    force_terminal: Option<bool>,
    #[allow(dead_code)] // Reserved for future tab expansion support
    tab_size: usize,
    record: bool,
    markup_enabled: bool,
    highlight_enabled: bool,
    soft_wrap: bool,
    /// Pluggable highlighter applied by `render_str` when `highlight_enabled`.
    highlighter: Box<dyn crate::utils::highlighter::Highlighter + Send + Sync>,
    no_color: bool,
    quiet: bool,
    safe_box: bool,
    legacy_windows: bool,
    base_style: Option<Style>,
    /// When `true`, `Console::log` appends the caller's file:line to each log line.
    log_path: bool,
    /// When `true` (the default), `Console::log` and `Console::log_objects`
    /// prepend a `[HH:MM:SS]` timestamp to each line. Set to `false` via
    /// [`ConsoleBuilder::log_time`] to suppress the timestamp.
    log_time: bool,

    // Theme
    theme_stack: ThemeStack,

    // Buffers
    buffer: Vec<Segment>,
    buffer_index: usize,
    record_buffer: Vec<Segment>,

    // State
    is_alt_screen: bool,
    capture_buffer: Option<Vec<Segment>>,
    /// Stack of nested live-display IDs.
    ///
    /// The top-of-stack ID is the currently-active Live; any preceding entries
    /// represent outer Live displays that have been suspended while a nested
    /// Live was started. Mirrors rich v14.1.0's `Console._live_stack` —
    /// allows Progress + Live + Status etc. to nest without each one
    /// clobbering the others' state.
    live_stack: Vec<usize>,

    /// Registered render hooks, invoked in order by [`Console::print_with_hooks`].
    ///
    /// Note: hooks do **not** fire when calling [`Console::print`] directly —
    /// use [`Console::print_with_hooks`] to have hooks process the renderable.
    render_hooks: Vec<Box<dyn RenderHook>>,

    /// Per-console style interner (foundation for L2 — see
    /// `.review/V0_11_DESIGN.md`). **Dormant in v0.11.0-alpha.1**: no
    /// caller currently interns or resolves through it. Wired in PR1b
    /// when `Segment::style` is converted from a field to a method.
    /// `Arc<Mutex<...>>` so the handle returned by `style_interner()` can
    /// be shared across threads (e.g. by a future Live integration) and
    /// to leave room for cross-Console resegmenting in PR3.
    style_interner: Arc<Mutex<StyleInterner>>,

    /// Detected terminal capabilities (color system, truecolor, unicode version, etc.).
    /// Populated at construction time from environment variables.
    capabilities: ConsoleCapabilities,

    /// Optional output sink override. When `Some`, render output goes here
    /// instead of `std::io::stdout()`. Set via [`Console::with_writer`].
    /// Capture and record modes still take precedence.
    ///
    /// `+ Send + Sync` (not just `+ Send`) preserves `Console: Sync` so
    /// downstream code can wrap a Console in `Arc<…>` for cross-task
    /// sharing — same trait bounds Console had pre-v1.2.0.
    pub(crate) writer_override: Option<Box<dyn std::io::Write + Send + Sync>>,

    /// Opt 2 (BufWriter coalescing): nesting depth of
    /// [`begin_synchronized`](Self::begin_synchronized) calls.
    ///
    /// When `sync_depth > 0`, `write_segments` defers the final `flush()`
    /// to the matching `end_synchronized` call, allowing multiple segment
    /// writes within one synchronized frame to be coalesced into a single
    /// OS write by the `BufWriter` that wraps `writer_override`.
    pub(crate) sync_depth: usize,

    // -- Asciinema v2 export (feature-gated) --------------------------------
    /// Injected clock function (asciinema feature).
    ///
    /// `None` → use the default native/wasm clock inside `clock_now()`.
    #[cfg(feature = "asciinema")]
    pub(crate) asciinema_clock: Option<crate::console::console_asciinema::AsciinemaClock>,

    /// Whether an asciinema timed-recording session is currently active.
    ///
    /// Set by `begin_asciinema_record`; cleared implicitly when there are
    /// no events (a session is considered active when `asciinema_start`
    /// has been set).
    #[cfg(feature = "asciinema")]
    pub(crate) asciinema_active: bool,

    /// Absolute clock reading at the time `begin_asciinema_record` was called.
    ///
    /// Used to convert absolute clock readings into elapsed-seconds offsets.
    #[cfg(feature = "asciinema")]
    pub(crate) asciinema_start: f64,

    /// Accumulated timed events: `(elapsed_secs, ansi_string)`.
    ///
    /// Populated by `maybe_record_asciinema_event` during an active session.
    #[cfg(feature = "asciinema")]
    pub(crate) asciinema_events: Vec<(f64, String)>,
}

impl Console {
    /// Create a Console with sensible defaults.
    ///
    /// Prefer [`Console::default()`] for the common one-line case — it
    /// calls this method. Reach for [`Console::builder`] only when you
    /// need explicit overrides (custom width, recording for export,
    /// forcing a specific color system).
    ///
    /// Defaults:
    /// - **Color**: TrueColor, auto-disabled by `NO_COLOR`,
    ///   auto-forced by `FORCE_COLOR` / `CLICOLOR`.
    /// - **Width**: auto-detected from terminal; falls back to 80.
    /// - **Markup**: enabled (`[bold]hi[/]`-style tags parsed by
    ///   [`print_text`](Self::print_text)).
    /// - **Recording**: off — enable via the builder for
    ///   [`export_html`](Self::export_html) / [`export_svg`](Self::export_svg).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let mut c = Console::default();           // recommended
    /// c.print_text("[bold green]ready[/]");
    /// ```
    pub fn new() -> Self {
        ConsoleBuilder::default().build()
    }

    /// Create a Console using the builder pattern. Use when you need
    /// explicit overrides on top of [`Console::default()`].
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let console = Console::builder()
    ///     .width(120)
    ///     .record(true)       // for export_html / export_svg
    ///     .build();
    /// assert_eq!(console.width(), 120);
    /// ```
    pub fn builder() -> ConsoleBuilder {
        ConsoleBuilder::default()
    }

    /// Create a Console whose output goes to **stderr** with terminal state
    /// detected from `stderr` itself (not `stdout`).
    ///
    /// This is the correct console to use for diagnostic output, prompts, and
    /// error messages that should always be visible even when `stdout` is
    /// redirected to a file or pipe.
    ///
    /// Terminal detection uses [`std::io::IsTerminal`] on `stderr` on native
    /// targets; on wasm the `TERM` env-var fallback is used. When `stderr` is
    /// a tty the color system is auto-detected; when it is piped the console
    /// is plain (same policy as the stdout console).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let mut c = Console::stderr();
    /// c.print_text("[bold red]error:[/] something went wrong");
    /// ```
    pub fn stderr() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let is_tty = {
            use std::io::IsTerminal as _;
            std::io::stderr().is_terminal()
        };
        #[cfg(target_arch = "wasm32")]
        let is_tty = matches!(
            std::env::var("TERM").as_deref(),
            Ok(t) if !t.is_empty() && t != "dumb"
        );

        ConsoleBuilder::default()
            .force_terminal(is_tty)
            .build()
            .with_writer(std::io::stderr())
    }

    /// Construct a `Console` from a fully-configured `ConsoleBuilder`.
    /// Called by `ConsoleBuilder::build`; lives here so `Console`'s
    /// private fields stay private to console.rs.
    pub(crate) fn from_builder(builder: ConsoleBuilder) -> Self {
        // Color system priority (highest first):
        //   1. color_system_override (explicit ColorSystem value)
        //   2. color_system (string, e.g. "truecolor")
        //   3. no_color(true) explicitly set by caller
        //   4. Environment vars (NO_COLOR, FORCE_COLOR, CLICOLOR_FORCE, CLICOLOR)
        //   5. Non-terminal output (piped/redirected) → no color
        //   6. Auto-detect from COLORTERM / TERM
        let has_explicit_cs = matches!(
            builder.color_system.as_deref(),
            Some("standard" | "256" | "truecolor" | "windows")
        );

        // A "color forced on" condition: force_terminal(true) is an explicit
        // signal that the caller wants terminal behaviour (ANSI output).
        let force_terminal_on = builder.force_terminal == Some(true);

        let color_system = if let Some(cs) = builder.color_system_override {
            Some(cs)
        } else if has_explicit_cs {
            match builder.color_system.as_deref() {
                Some("standard") => Some(ColorSystem::Standard),
                Some("256") => Some(ColorSystem::EightBit),
                Some("truecolor") => Some(ColorSystem::TrueColor),
                Some("windows") => Some(ColorSystem::Windows),
                _ => unreachable!(),
            }
        } else if builder.no_color_explicit && builder.no_color {
            None
        } else {
            match detect_color_env() {
                ColorEnvOverride::NoColor => None,
                // FORCE_COLOR / CLICOLOR_FORCE → keep color even when piped.
                ColorEnvOverride::ForceColor => Some(ColorSystem::EightBit),
                ColorEnvOverride::ForceColorTruecolor => Some(ColorSystem::TrueColor),
                ColorEnvOverride::None => {
                    if builder.no_color {
                        None
                    } else if !force_terminal_on && !detect_is_terminal() {
                        // Piped / redirected output: disable color unless the
                        // caller explicitly forced terminal mode.
                        None
                    } else {
                        // P1 parity: use env-based detection instead of hard TrueColor default.
                        let colorterm = std::env::var("COLORTERM").ok();
                        let term = std::env::var("TERM").ok();
                        Some(detect_color_system_from(
                            colorterm.as_deref(),
                            term.as_deref(),
                        ))
                    }
                }
            }
        };

        // Mirror the no_color flag: if color_system resolved to None due to
        // non-terminal detection, also set no_color so render_buffer skips SGR.
        let effective_no_color = builder.no_color || color_system.is_none();

        let theme = builder.theme.unwrap_or_else(|| Theme::new(None, true));
        #[allow(unused_mut)]
        let mut theme_stack = ThemeStack::new(theme);

        // ---------------------------------------------------------------------------
        // GILT_THEME env var (json + native only).
        //
        // If the caller set an explicit path via ConsoleBuilder::theme_from_path,
        // use that.  Otherwise check the GILT_THEME environment variable.  Either
        // way, errors (missing file, bad JSON) are non-fatal: we just skip the
        // override and keep the default theme.
        // ---------------------------------------------------------------------------
        #[cfg(all(feature = "json", not(target_arch = "wasm32")))]
        {
            use crate::console::console_builder::load_theme_from_path;

            // Explicit builder path wins over env var.
            let path_to_load: Option<std::path::PathBuf> = if let Some(p) = builder.theme_path {
                Some(p)
            } else {
                std::env::var("GILT_THEME")
                    .ok()
                    .map(std::path::PathBuf::from)
            };

            if let Some(path) = path_to_load {
                if let Some(loaded) = load_theme_from_path(&path) {
                    // Push the loaded theme on top of the default so it overrides
                    // named styles while still inheriting defaults that weren't
                    // overridden.
                    theme_stack.push_theme(loaded, true);
                }
            }
        }

        // Determine is_terminal the same way the Console struct does it so
        // ConsoleCapabilities mirrors the Console's own `is_terminal()` method.
        let builder_is_terminal = if let Some(forced) = builder.force_terminal {
            forced
        } else {
            detect_is_terminal()
        };
        let capabilities = ConsoleCapabilities::from_env(builder_is_terminal);

        // Enable Windows VT processing (opt-in via `windows-vt` feature).
        // On non-Windows or when the feature is disabled this is a pure no-op.
        crate::windows_vt::enable_windows_vt();

        Console {
            color_system,
            width_override: builder.width,
            height_override: builder.height,
            force_terminal: builder.force_terminal,
            tab_size: builder.tab_size,
            record: builder.record,
            markup_enabled: builder.markup,
            highlight_enabled: builder.highlight,
            soft_wrap: builder.soft_wrap,
            highlighter: Box::new(crate::utils::highlighter::ReprHighlighter::new()),
            no_color: effective_no_color,
            quiet: builder.quiet,
            safe_box: builder.safe_box,
            // Explicit builder override (Some) wins; otherwise auto-detect
            // at construction time. See `detect_legacy_windows` for the
            // cfg-gated detection logic.
            legacy_windows: builder.legacy_windows.unwrap_or_else(detect_legacy_windows),
            base_style: None,
            log_path: builder.log_path,
            // log_time defaults to true (preserves the historical
            // "[HH:MM:SS] message" output).
            log_time: builder.log_time,
            theme_stack,
            buffer: Vec::new(),
            buffer_index: 0,
            record_buffer: Vec::new(),
            is_alt_screen: false,
            capture_buffer: None,
            live_stack: Vec::new(),
            render_hooks: Vec::new(),
            style_interner: Arc::new(Mutex::new(StyleInterner::new())),
            writer_override: None,
            sync_depth: 0,
            capabilities,
            #[cfg(feature = "asciinema")]
            asciinema_clock: None,
            #[cfg(feature = "asciinema")]
            asciinema_active: false,
            #[cfg(feature = "asciinema")]
            asciinema_start: 0.0,
            #[cfg(feature = "asciinema")]
            asciinema_events: Vec::new(),
        }
    }

    /// Send all output to a custom writer instead of `std::io::stdout()`.
    /// Use for log-to-file, in-memory testing, or piping into a pre-existing
    /// sink (e.g. a network socket). Capture and record modes still take
    /// precedence over the override.
    ///
    /// ```
    /// # use gilt::console::Console;
    /// let buf: Vec<u8> = Vec::new();
    /// let mut console = Console::default().with_writer(buf);
    /// console.print_text("hello");
    /// // output is in console's writer, not on stdout
    /// ```
    pub fn with_writer<W: std::io::Write + Send + Sync + 'static>(mut self, writer: W) -> Self {
        // Wrap in BufWriter so that multiple write_all calls within a
        // synchronized block are buffered and coalesced into a single OS
        // write when flush() is called at end_synchronized (Opt 2).
        self.writer_override = Some(Box::new(std::io::BufWriter::new(writer)));
        self
    }

    // -- Properties ---------------------------------------------------------

    /// Return the detected terminal capabilities for this console.
    ///
    /// Capabilities are derived from environment variables at construction time
    /// (no blocking probes).  See [`ConsoleCapabilities`] for the full list
    /// of flags.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let console = Console::builder().force_terminal(true).build();
    /// let caps = console.capabilities();
    /// // synchronized_output defaults to true (CSI ?2026 is harmless on old terms)
    /// assert!(caps.synchronized_output);
    /// ```
    pub fn capabilities(&self) -> &ConsoleCapabilities {
        &self.capabilities
    }

    /// Override the detected terminal capabilities.
    ///
    /// Primarily intended for testing, where callers need to force specific
    /// protocol flags (e.g. `kitty = true`) without spawning a real terminal.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    /// use gilt::console_caps::ConsoleCapabilities;
    ///
    /// let mut console = Console::builder().force_terminal(true).build();
    /// let caps = ConsoleCapabilities { kitty: true, ..console.capabilities().clone() };
    /// console.set_capabilities(caps);
    /// assert!(console.capabilities().kitty);
    /// ```
    pub fn set_capabilities(&mut self, caps: ConsoleCapabilities) {
        self.capabilities = caps;
    }

    /// Whether the console is in recording mode (for `export_html` / `export_svg`).
    ///
    /// When `true`, `Image` and other renderables that have terminal-specific
    /// rendering paths (Kitty graphics, Sixel) will fall back to the
    /// always-correct halfblock representation so that HTML/SVG export produces
    /// valid styled text rather than raw escape sequences.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let recording = Console::builder().record(true).build();
    /// assert!(recording.is_recording());
    ///
    /// let normal = Console::builder().build();
    /// assert!(!normal.is_recording());
    /// ```
    pub fn is_recording(&self) -> bool {
        self.record
    }

    /// The current terminal width in columns.
    pub fn width(&self) -> usize {
        if let Some(w) = self.width_override {
            return w;
        }
        let (w, _) = Self::detect_terminal_size();
        w
    }

    /// The current terminal height in rows.
    pub fn height(&self) -> usize {
        if let Some(h) = self.height_override {
            return h;
        }
        let (_, h) = Self::detect_terminal_size();
        h
    }

    /// Current terminal dimensions.
    pub fn size(&self) -> ConsoleDimensions {
        ConsoleDimensions {
            width: self.width(),
            height: self.height(),
        }
    }

    /// Build the default `ConsoleOptions` for this console.
    pub fn options(&self) -> ConsoleOptions {
        let size = self.size();
        ConsoleOptions {
            size,
            legacy_windows: self.legacy_windows,
            min_width: 1,
            max_width: size.width,
            is_terminal: self.is_terminal(),
            encoding: Cow::Borrowed("utf-8"),
            max_height: size.height,
            justify: None,
            overflow: None,
            no_wrap: None,
            highlight: Some(self.highlight_enabled),
            markup: Some(self.markup_enabled),
            height: None,
        }
    }

    /// The current color system name, or `None` if colors are disabled.
    pub fn color_system_name(&self) -> Option<&str> {
        self.color_system.as_ref().map(|cs| match cs {
            ColorSystem::Standard => "standard",
            ColorSystem::EightBit => "256",
            ColorSystem::TrueColor => "truecolor",
            ColorSystem::Windows => "windows",
        })
    }

    /// The active `ColorSystem`, if any.
    pub fn color_system(&self) -> Option<ColorSystem> {
        self.color_system
    }

    /// The character encoding (always "utf-8" in Rust).
    pub fn encoding(&self) -> &str {
        "utf-8"
    }

    /// Whether the console is connected to a terminal.
    ///
    /// Resolution order:
    /// 1. Explicit `force_terminal` set on the builder
    /// 2. `TTY_COMPATIBLE=1`/`0` environment override
    /// 3. `std::io::IsTerminal` on stdout (native targets only)
    /// 4. `TERM` is set and not `"dumb"` (wasm / fallback)
    pub fn is_terminal(&self) -> bool {
        if let Some(forced) = self.force_terminal {
            return forced;
        }
        match crate::color::color_env::detect_tty_compatible() {
            crate::color::color_env::TtyOverride::ForceTty => return true,
            crate::color::color_env::TtyOverride::ForceNotTty => return false,
            crate::color::color_env::TtyOverride::None => {}
        }
        detect_is_terminal()
    }

    /// Whether the console should treat the user as interactive (prompts,
    /// progress bars with refresh, live updates, etc.).
    ///
    /// Resolution order:
    /// 1. `TTY_INTERACTIVE=1`/`0` environment override
    /// 2. Falls back to [`is_terminal`](Self::is_terminal)
    ///
    /// This is intentionally independent of TTY status so a user can pipe
    /// output to a file but still be prompted on stdin.
    pub fn is_interactive(&self) -> bool {
        match crate::color::color_env::detect_tty_interactive() {
            crate::color::color_env::TtyOverride::ForceTty => true,
            crate::color::color_env::TtyOverride::ForceNotTty => false,
            crate::color::color_env::TtyOverride::None => self.is_terminal(),
        }
    }

    /// Whether safe-box substitution is enabled for this console.
    ///
    /// When `true`, non-ASCII box-drawing characters are substituted with
    /// simpler Unicode equivalents on legacy-Windows terminals, or with ASCII
    /// when the encoding is not UTF-based.  Widgets that have their own
    /// `safe_box: Option<bool>` field should use this as the fallback default.
    pub fn safe_box(&self) -> bool {
        self.safe_box
    }

    /// Whether this console is rendering against a legacy Windows console
    /// (one without VT/ANSI escape-code support).
    ///
    /// When `true`, gilt routes output through the legacy Win32 console
    /// API and avoids emitting ANSI escape sequences (SGR, OSC 8, OSC 11,
    /// etc.).  When `false`, gilt emits ANSI escapes as usual.
    ///
    /// Defaults to `false` on every non-Windows target. On Windows,
    /// auto-detection runs at [`Console::builder`] time and is `true`
    /// unless one of `WT_SESSION`, `WT_PROFILE_ID`, or `TERM_PROGRAM` is
    /// set in the environment (which indicates a modern terminal). The
    /// explicit [`ConsoleBuilder::with_legacy_windows`] setter overrides
    /// auto-detection.
    pub fn legacy_windows(&self) -> bool {
        self.legacy_windows
    }

    /// Whether this is a "dumb" terminal with no styling support.
    pub fn is_dumb_terminal(&self) -> bool {
        match std::env::var("TERM") {
            Ok(term) => term == "dumb",
            Err(_) => false,
        }
    }

    // -- Background detection -----------------------------------------------

    /// Detect the terminal background (dark, light, or unknown).
    ///
    /// When the `terminal-query` feature is enabled **and** the build is
    /// native (not wasm32), this probes the controlling TTY via OSC 11.
    /// On failure, timeout, or unsupported terminals the result is
    /// [`ConsoleBackground::Unknown`].
    ///
    /// Without the `terminal-query` feature (the default), this always
    /// returns [`ConsoleBackground::Unknown`].
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    /// use gilt::terminal_bg::ConsoleBackground;
    ///
    /// let console = Console::new();
    /// let bg = console.detect_background();
    /// // In a non-interactive environment this will be Unknown.
    /// assert!(matches!(
    ///     bg,
    ///     ConsoleBackground::Dark | ConsoleBackground::Light | ConsoleBackground::Unknown
    /// ));
    /// ```
    pub fn detect_background(&self) -> crate::terminal_bg::ConsoleBackground {
        #[cfg(all(feature = "terminal-query", not(target_arch = "wasm32")))]
        {
            crate::terminal_bg::query_terminal_background()
        }
        #[cfg(not(all(feature = "terminal-query", not(target_arch = "wasm32"))))]
        {
            crate::terminal_bg::ConsoleBackground::Unknown
        }
    }

    // -- Terminal detection -------------------------------------------------

    /// Detect the terminal size.
    ///
    /// Resolution order (each dimension independently):
    /// 1. `COLUMNS` / `LINES` environment variables — these win, so tests, CI,
    ///    and explicit overrides remain deterministic.
    /// 2. The real terminal dimensions via an `ioctl`, when the `terminal-size`
    ///    feature is enabled (it is by default) on a non-wasm target and a
    ///    standard stream is connected to a terminal.
    /// 3. Fallback `80 x 25` (used when piped/redirected or on wasm).
    ///
    /// Most shells do not export `COLUMNS` to child processes, so before this
    /// the width was effectively pinned to `80`; the `ioctl` query fixes that
    /// for native builds while keeping wasm and `default-features = false`
    /// builds free of terminal syscalls.
    pub fn detect_terminal_size() -> (usize, usize) {
        let env_width = std::env::var("COLUMNS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok());
        let env_height = std::env::var("LINES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok());

        let (query_width, query_height) = query_terminal_size();

        let width = env_width.or(query_width).unwrap_or(80);
        let height = env_height.or(query_height).unwrap_or(25);
        (width, height)
    }

    // -- Theme / Style ------------------------------------------------------

    /// Look up a style by name from the theme stack, or parse it as a style definition.
    pub fn get_style(&self, name: &str) -> Result<Style, ConsoleError> {
        // First try the theme stack
        if let Some(style) = self.theme_stack.get(name) {
            return Ok(style.clone());
        }
        // Then try parsing as a style definition
        Style::parse_strict(name).map_err(|e| {
            ConsoleError::RenderError(format!("Failed to get style '{}': {}", name, e))
        })
    }

    /// Per-console style interner. **Dormant in v0.11.0-alpha.1** — no
    /// internal callers route through it yet. Exposed so `Segment` (PR1b)
    /// and the eventual L2 activation (PR3) can intern/resolve through
    /// the same id space as the parent `Console`.
    ///
    /// The returned handle is `Arc<Mutex<…>>` because a `Console::copy`
    /// (e.g. `begin_capture`) shares the id space with its origin.
    pub fn style_interner(&self) -> &Arc<Mutex<StyleInterner>> {
        &self.style_interner
    }

    /// Push a new theme onto the theme stack.
    ///
    /// If `inherit` is true the new layer inherits all styles from the current
    /// top of the stack, with the pushed theme's styles overriding any that
    /// share the same name.  If `inherit` is false, only the pushed theme's
    /// styles are visible until [`pop_theme`](Self::pop_theme) is called.
    pub fn push_theme(&mut self, theme: Theme, inherit: bool) {
        self.theme_stack.push_theme(theme, inherit);
    }

    /// Pop the top theme from the theme stack.
    pub fn pop_theme(&mut self) {
        let _ = self.theme_stack.pop_theme();
    }

    /// Push `theme` and return a [`ThemeGuard`] that pops it on drop.
    ///
    /// The guard implements [`Deref`](std::ops::Deref) and
    /// [`DerefMut`](std::ops::DerefMut) targeting `Console`, so you can call
    /// any `Console` method through it without dropping the guard first.
    ///
    /// ```
    /// use gilt::console::Console;
    /// use gilt::theme::Theme;
    /// use std::collections::HashMap;
    ///
    /// let mut console = Console::new();
    /// let mut styles = HashMap::new();
    /// styles.insert("info".to_string(), gilt::style::Style::parse("bold cyan"));
    /// let theme = Theme::new(Some(styles), true);
    /// {
    ///     let mut guard = console.use_theme(theme, true);
    ///     // Call console methods directly through the guard.
    ///     assert!(guard.get_style("info").is_ok());
    /// } // guard drops → pop_theme called automatically
    /// ```
    pub fn use_theme(&mut self, theme: Theme, inherit: bool) -> ThemeGuard<'_> {
        self.push_theme(theme, inherit);
        ThemeGuard { console: self }
    }

    // -- Control ------------------------------------------------------------

    /// Send a terminal control sequence.
    ///
    /// No-ops on dumb terminals (`TERM=dumb` or non-terminal output), because
    /// escape sequences would appear as raw text on those targets (parity with
    /// Python rich's dumb-terminal guard, finding #2).
    pub fn control(&mut self, ctrl: &Control) {
        if !self.quiet && !self.is_dumb_terminal() {
            self.write_segments(std::slice::from_ref(&ctrl.segment));
        }
    }

    /// Ring the terminal bell.
    pub fn bell(&mut self) {
        self.control(&Control::bell());
    }

    /// Clear the terminal screen, moving the cursor to home first.
    ///
    /// Emits `\x1b[H\x1b[2J` (Home + Clear) — matches rich's default
    /// `Console.clear(home=True)` behaviour.  For fine-grained control use
    /// [`clear_screen`](Self::clear_screen).
    pub fn clear(&mut self) {
        self.control(&Control::clear_with_home());
    }

    /// Clear the terminal screen with an optional home-cursor move.
    ///
    /// - `home == true`  → `\x1b[H\x1b[2J`  (parity with rich `clear(home=True)`)
    /// - `home == false` → `\x1b[2J`         (parity with rich `clear(home=False)`)
    pub fn clear_screen(&mut self, home: bool) {
        if home {
            self.control(&Control::clear_with_home());
        } else {
            self.control(&Control::clear());
        }
    }

    /// Show or hide the cursor.
    ///
    /// No-ops when the output is not an interactive terminal — emitting cursor
    /// escape sequences to a pipe or file would produce raw garbage.
    pub fn show_cursor(&mut self, show: bool) {
        if !self.is_terminal() {
            return;
        }
        self.control(&Control::show_cursor(show));
    }

    /// Enable or disable the alternate screen buffer.
    ///
    /// Returns `true` if the operation was performed.
    pub fn set_alt_screen(&mut self, enable: bool) -> bool {
        if !self.is_terminal() || self.legacy_windows {
            return false;
        }
        if enable == self.is_alt_screen {
            return false;
        }
        self.is_alt_screen = enable;
        self.control(&Control::alt_screen(enable));
        true
    }

    /// Set the terminal window title.
    ///
    /// Returns `true` if the title was set (only works on terminals).
    pub fn set_window_title(&mut self, title: &str) -> bool {
        if !self.is_terminal() {
            return false;
        }
        self.control(&Control::title(title));
        true
    }

    // -- Synchronized Output ------------------------------------------------

    /// Begin synchronized output (DEC Mode 2026).
    ///
    /// The terminal buffers all subsequent output until
    /// [`end_synchronized`](Console::end_synchronized) is called, then paints
    /// atomically. This prevents flickering and tearing during rapid updates.
    ///
    /// Also increments the internal `sync_depth` counter so that
    /// `write_segments` defers the final `flush()` to the matching
    /// `end_synchronized`, allowing the `BufWriter` that wraps the
    /// underlying writer to coalesce all segment writes into a single OS
    /// write (Opt 2).
    pub fn begin_synchronized(&mut self) {
        self.sync_depth += 1;
        self.control(&Control::begin_sync());
    }

    /// End synchronized output (DEC Mode 2026).
    ///
    /// The terminal flushes all buffered content and renders it at once.
    ///
    /// Decrements the `sync_depth` counter and, when it reaches zero,
    /// explicitly flushes the underlying writer (BufWriter drain) so all
    /// bytes buffered since `begin_synchronized` are sent to the OS in a
    /// single write (Opt 2).
    pub fn end_synchronized(&mut self) {
        self.control(&Control::end_sync());
        if self.sync_depth > 0 {
            self.sync_depth -= 1;
        }
        // Flush the underlying BufWriter now that the synchronized block
        // has closed. This drains all buffered bytes in one OS write.
        if self.sync_depth == 0 && !self.quiet {
            use std::io::Write as _;
            if let Some(w) = self.writer_override.as_mut() {
                let _ = w.flush();
            }
            // stdout path does not use BufWriter — its flush is still
            // handled inside write_segments as before.
        }
    }

    /// Execute a closure with synchronized output wrapping.
    ///
    /// Emits the DEC Mode 2026 begin sequence, runs the closure, then emits
    /// the end sequence. If the closure panics the end sequence is still sent
    /// (panic-safe) via a RAII drop guard (finding #4).
    pub fn synchronized<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Console) -> R,
    {
        self.begin_synchronized();
        // RAII guard: writes the end-sync segment when dropped, whether the
        // closure returns normally or unwinds.
        struct SyncGuard {
            /// The DEC 2026 end-sync escape sequence to write on drop.
            segment: crate::segment::Segment,
            /// True once we have already emitted end-sync (normal path).
            done: bool,
        }
        impl Drop for SyncGuard {
            fn drop(&mut self) {
                // If `done` is false we are being dropped due to a panic — we
                // cannot access the Console here, so we write the escape
                // sequence directly to stderr as a best-effort recovery.
                if !self.done {
                    use std::io::Write as _;
                    let _ = std::io::stderr().write_all(self.segment.text.as_bytes());
                }
            }
        }
        let end_seg = crate::control::Control::end_sync().segment.clone();
        let mut guard = SyncGuard {
            segment: end_seg,
            done: false,
        };
        let result = f(self);
        // Normal path: emit end-sync through the Console and mark guard done.
        self.end_synchronized();
        guard.done = true;
        result
    }

    // -- Desktop notification (OSC 9) ---------------------------------------

    /// Send a desktop notification via OSC 9.
    ///
    /// If `title` is non-empty, the message is `"{title}: {body}"`;
    /// otherwise just `body` is used. No-ops on dumb terminals.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let mut c = Console::builder().force_terminal(true).no_color(true).build();
    /// c.notify("Build", "Done"); // no-op unless TERM != dumb
    /// ```
    pub fn notify(&mut self, title: &str, body: &str) {
        self.control(&Control::notify(title, body));
    }

    // -- Taskbar progress (OSC 9;4) -----------------------------------------

    /// Set the taskbar progress indicator via OSC 9;4 (ConEmu / Windows Terminal).
    ///
    /// `state` controls the indicator style and `percent` is clamped to 0–100.
    /// No-ops on dumb terminals.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    /// use gilt::segment::TaskbarState;
    ///
    /// let mut c = Console::builder().force_terminal(true).no_color(true).build();
    /// c.set_taskbar_progress(TaskbarState::Normal, 50);
    /// ```
    pub fn set_taskbar_progress(&mut self, state: crate::segment::TaskbarState, percent: u8) {
        self.control(&Control::taskbar_progress(state, percent));
    }

    // -- Clipboard (OSC 52) -------------------------------------------------

    /// Copy text to the system clipboard via OSC 52 escape sequence.
    ///
    /// This works in terminals that support OSC 52 (kitty, iTerm2, WezTerm,
    /// etc.). The text is base64-encoded in the escape sequence.
    pub fn copy_to_clipboard(&mut self, text: &str) {
        self.control(&Control::set_clipboard(text));
    }

    /// Request clipboard contents via OSC 52.
    ///
    /// Most terminals require explicit opt-in for clipboard reading.
    /// The terminal will respond with an OSC 52 sequence containing the
    /// base64-encoded clipboard contents, which must be read from stdin.
    pub fn request_clipboard(&mut self) {
        self.control(&Control::request_clipboard());
    }

    // -- Pager --------------------------------------------------------------

    /// Pipe recorded output through an external pager.
    ///
    /// Captures the current recorded output via `export_text(clear, styles)`
    /// and pipes it through a [`Pager`]. If `pager_command` is `Some`, uses
    /// the specified command; otherwise uses the default pager (`less -r`,
    /// or `$PAGER` if set — see [`Pager::from_env`]).
    ///
    /// `styles` controls whether ANSI escapes are kept (`true`) or stripped
    /// (`false`, the default — matches rich's `Console.pager(styles=False)`).
    ///
    /// Pager errors are silently ignored.
    pub fn pager(&mut self, pager_command: Option<&str>, styles: bool) {
        let text = self.export_text(true, styles);
        let pager = match pager_command {
            Some(cmd) => Pager::new().with_command(cmd),
            None => Pager::new(),
        };
        let _ = pager.show(&text);
    }

    // -- Screen helpers -----------------------------------------------------

    /// Enter alternate screen mode, optionally hiding the cursor.
    ///
    /// Call [`exit_screen`](Console::exit_screen) with the same `hide_cursor`
    /// value to restore the previous state.
    pub fn enter_screen(&mut self, hide_cursor: bool) {
        self.set_alt_screen(true);
        if hide_cursor {
            self.show_cursor(false);
        }
    }

    /// Exit alternate screen mode, restoring the cursor if it was hidden.
    ///
    /// Pass the same `hide_cursor` value that was used with
    /// [`enter_screen`](Console::enter_screen).
    pub fn exit_screen(&mut self, hide_cursor: bool) {
        if hide_cursor {
            self.show_cursor(true);
        }
        self.set_alt_screen(false);
    }

    /// Render a [`Renderable`] at an arbitrary `(x, y)` position in the active
    /// alternate screen.
    ///
    /// Moves the cursor to the absolute position (0-indexed), prints the
    /// renderable, then leaves the cursor at the position the renderable's
    /// last segment ended at. Designed for partial-update UIs (Layouts,
    /// dashboards) running inside `enter_screen`.
    ///
    /// Has no effect — and silently does nothing — when the console is not
    /// currently in alt-screen mode, since absolute positioning would
    /// scribble on the user's main scrollback otherwise.
    pub fn update_screen(&mut self, x: usize, y: usize, renderable: &dyn Renderable) {
        if !self.is_alt_screen {
            return;
        }
        let ctrl = crate::utils::control::Control::move_to(x as i32, y as i32);
        self.write_segments(&[ctrl.segment]);
        self.print(renderable);
    }

    /// Render a [`Renderable`] into a specific [`Region`] in the active
    /// alternate screen.
    ///
    /// Moves the cursor to `(region.x, region.y)`, then renders the renderable
    /// with `ConsoleOptions::max_width` clamped to `region.width`.  This lets
    /// layout widgets confine their output to a rectangular area without
    /// knowing the full terminal size.
    ///
    /// Like [`update_screen`](Self::update_screen), no-ops when the console is
    /// not currently in alt-screen mode.
    pub fn update_screen_region(
        &mut self,
        region: crate::region::Region,
        renderable: &dyn Renderable,
    ) {
        if !self.is_alt_screen {
            return;
        }
        let ctrl = crate::utils::control::Control::move_to(region.x, region.y);
        self.write_segments(&[ctrl.segment]);
        let mut opts = self.options();
        opts.max_width = region.width;
        // Clamp to the region's height so a renderable taller than the region
        // does not overflow into adjacent pane rows (parity with rich's
        // Screen.update, which renders into a bounded box).
        opts.height = Some(region.height);
        let segments = renderable.gilt_console(self, &opts);
        // Not all renderables honour opts.height (Text doesn't), so also
        // truncate at the segment level: split into lines, keep only the
        // first `region.height`, then flatten back.
        let mut lines = Segment::split_and_crop_lines(&segments, region.width, None, false, true);
        lines.truncate(region.height);
        let clamped: Vec<Segment> = lines.into_iter().flatten().collect();
        self.write_segments(&clamped);
    }

    /// Render a slice of [`Segment`] lines at successive rows starting from
    /// `(x, y)` in the active alternate screen.
    ///
    /// Each `Vec<Segment>` in `lines` is treated as one line — printed at
    /// `(x, y + i)` for the i-th entry. Useful when you've already produced
    /// per-line segments via `Console::render` and want to splat them into a
    /// known position without going through a full Renderable wrapper.
    ///
    /// Like [`update_screen`](Self::update_screen), no-ops when not in
    /// alt-screen mode.
    pub fn update_screen_lines(&mut self, x: usize, y: usize, lines: &[Vec<Segment>]) {
        if !self.is_alt_screen {
            return;
        }
        for (i, line) in lines.iter().enumerate() {
            let ctrl = crate::utils::control::Control::move_to(x as i32, (y + i) as i32);
            self.write_segments(&[ctrl.segment]);
            self.write_segments(line);
        }
    }

    // -- Live display ID ----------------------------------------------------

    /// Push a Live-display ID onto the stack, making it the active Live.
    ///
    /// Returns `true` if the ID was pushed (always true; the API returns a
    /// `bool` for parity with rich's `set_live` which returned `False` when
    /// nesting was disabled — gilt always allows nesting).
    pub fn push_live(&mut self, live_id: usize) -> bool {
        self.live_stack.push(live_id);
        true
    }

    /// Pop the top Live-display ID off the stack. Returns the popped ID, or
    /// `None` if the stack was empty.
    pub fn pop_live(&mut self) -> Option<usize> {
        self.live_stack.pop()
    }

    /// Return the currently-active Live-display ID (the top of the stack), or
    /// `None` when no Live is active.
    pub fn current_live(&self) -> Option<usize> {
        self.live_stack.last().copied()
    }

    /// Number of currently-nested Live displays. `0` means no Live is active.
    pub fn live_depth(&self) -> usize {
        self.live_stack.len()
    }

    // -- Backwards-compatible single-slot API -------------------------------

    /// Set the active Live-display ID. `Some(id)` pushes (or replaces top);
    /// `None` clears the entire stack.
    ///
    /// Provided for source compatibility with the pre-nesting API. New code
    /// should prefer [`push_live`](Self::push_live) / [`pop_live`](Self::pop_live).
    pub fn set_live(&mut self, live_id: Option<usize>) {
        match live_id {
            Some(id) => {
                if let Some(top) = self.live_stack.last_mut() {
                    *top = id;
                } else {
                    self.live_stack.push(id);
                }
            }
            None => self.live_stack.clear(),
        }
    }

    /// Clear all Live IDs. Equivalent to `set_live(None)`.
    pub fn clear_live(&mut self) {
        self.live_stack.clear();
    }

    // -- Render hooks -------------------------------------------------------

    /// Register a [`RenderHook`] that will be called by [`Console::print`]
    /// for every printed renderable.
    ///
    /// Hooks are invoked in registration order. Each hook receives a
    /// `Vec<&dyn Renderable>` and returns a (possibly different) vec.
    /// The last element of the final vec is what gets rendered.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::{Console, RenderHook, Renderable};
    ///
    /// struct NoopHook;
    /// impl RenderHook for NoopHook {
    ///     fn process_renderables<'a>(&self, r: Vec<&'a dyn Renderable>) -> Vec<&'a dyn Renderable> { r }
    /// }
    ///
    /// let mut c = Console::builder().width(80).build();
    /// c.add_render_hook(Box::new(NoopHook));
    /// ```
    pub fn add_render_hook(&mut self, hook: Box<dyn RenderHook>) {
        self.render_hooks.push(hook);
    }

    // -- Highlighter --------------------------------------------------------

    /// Replace the console's default [`ReprHighlighter`] with a custom one.
    ///
    /// The highlighter is applied by [`render_str`](Self::render_str) when
    /// `highlight` is enabled on this console.
    ///
    /// [`ReprHighlighter`]: crate::utils::highlighter::ReprHighlighter
    pub fn with_highlighter<H>(mut self, highlighter: H) -> Self
    where
        H: crate::utils::highlighter::Highlighter + Send + Sync + 'static,
    {
        self.highlighter = Box::new(highlighter);
        self
    }

    // -- Pager (closure form) -----------------------------------------------

    /// Run a closure, capturing its output, then pipe it through a pager.
    ///
    /// Unlike [`pager`](Self::pager) (which pipes an existing recording),
    /// this method enables recording, runs `f`, then pipes the captured
    /// output through the pager — matching rich's context-manager `pager()`.
    ///
    /// `pager_command` overrides the default `"less -r"`.
    /// `styles` controls whether ANSI escapes are kept (`true`) or stripped
    /// (`false`, the default — matches rich's `Console.pager(styles=False)`).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gilt::console::Console;
    ///
    /// let mut c = Console::new();
    /// c.pager_with(|c| {
    ///     c.print_text("Hello from the pager!");
    /// }, None, false);
    /// ```
    pub fn pager_with<F: FnOnce(&mut Console)>(
        &mut self,
        f: F,
        pager_command: Option<&str>,
        styles: bool,
    ) {
        let text = self.pager_text(styles, f);
        let pager = match pager_command {
            Some(cmd) => Pager::new().with_command(cmd),
            None => Pager::new(),
        };
        let _ = pager.show(&text);
    }

    /// Run a closure, capturing its output, and return it as the pager text.
    ///
    /// This is the render-only side of [`pager_with`](Self::pager_with): it
    /// does the closure, the record-mode setup, and the styled/plain text
    /// selection, but does NOT spawn the pager. The returned `String` is what
    /// `pager_with` would have piped to the pager.
    ///
    /// Useful for tests that need to assert on the formatted text without
    /// spawning a subprocess, and for callers that want to post-process the
    /// pager text (e.g. write it to a file) instead of piping it.
    ///
    /// `styles = false` (rich default) strips ANSI escapes; `styles = true`
    /// keeps the styled buffer as-is.
    pub fn pager_text<F: FnOnce(&mut Console)>(&mut self, styles: bool, f: F) -> String {
        let was_recording = self.record;
        self.record = true;
        // Panic-safety: if `f` panics, restore `record` before unwinding so
        // the console isn't left in record mode (silent buffer growth).
        // AssertUnwindSafe is sound: we only restore `record` to its prior
        // value in the error branch — we never re-enter `f` or read torn state.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(self)));
        if let Err(payload) = result {
            self.record = was_recording;
            std::panic::resume_unwind(payload);
        }
        // export_text requires self.record == true; call it before restoring.
        let text = self.export_text(true, styles);
        self.record = was_recording;
        text
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

// Export helpers and export methods live in console_export.rs (split in v1.2,
// export methods relocated in v1.7.1). The glob re-export makes the helpers
// available to the console_tests submodule via `use super::*;`.
#[path = "console_export.rs"]
mod console_export;
#[allow(unused_imports)]
use console_export::*;

// Scoped recording API (Console::scoped_record + Recording type).
// Split into console_recording.rs in v1.8.
#[path = "console_recording.rs"]
mod console_recording;
pub use console_recording::Recording;

// Asciinema v2 .cast export (opt-in: requires `asciinema` feature).
#[cfg(feature = "asciinema")]
#[path = "console_asciinema.rs"]
pub mod console_asciinema;

// ---------------------------------------------------------------------------
// Terminal size query (feature-gated, native only)
// ---------------------------------------------------------------------------

/// Query the real terminal dimensions via `ioctl` (`terminal-size` feature,
/// native targets). Returns `(None, None)` when not connected to a terminal.
#[cfg(all(feature = "terminal-size", not(target_arch = "wasm32")))]
fn query_terminal_size() -> (Option<usize>, Option<usize>) {
    match terminal_size::terminal_size() {
        Some((terminal_size::Width(w), terminal_size::Height(h))) => {
            (Some(w as usize), Some(h as usize))
        }
        None => (None, None),
    }
}

/// Fallback when the `terminal-size` feature is off or on wasm: env vars and
/// the `80x25` default are the only sources (no terminal syscalls).
#[cfg(not(all(feature = "terminal-size", not(target_arch = "wasm32"))))]
fn query_terminal_size() -> (Option<usize>, Option<usize>) {
    (None, None)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "console_tests.rs"]
mod tests;
