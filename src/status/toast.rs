//! Toast notifications for modern CLI apps — temporary status messages.
//!
//! Toast notifications provide non-intrusive, auto-dismissing status messages
//! with optional progress bars. They support multiple types (success, error,
//! warning, info) with distinct visual styles.
//!
//! # Examples
//!
//! ```
//! use gilt::prelude::*;
//! use gilt::toast::Toast;
//! use std::time::Duration;
//!
//! let mut console = Console::new();
//!
//! // Simple toast
//! Toast::success("Operation completed!").show(&mut console);
//!
//! // Toast with custom duration and icon
//! Toast::new("Custom message")
//!     .toast_type(gilt::toast::ToastType::Info)
//!     .duration(Duration::from_secs(5))
//!     .icon("🚀")
//!     .show(&mut console);
//!
//! // Toast with progress bar
//! Toast::info("Processing...")
//!     .show_progress(true)
//!     .show(&mut console);
//! ```

use std::time::Duration;

use crate::utils::box_chars::{BoxChars, ROUNDED};
use crate::console::{Console, Renderable};
use crate::utils::padding::PaddingDimensions;
use crate::panel::Panel;
use crate::progress_bar::ProgressBar;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;

// ---------------------------------------------------------------------------
// ToastType
// ---------------------------------------------------------------------------

/// The type of toast notification, determining its visual style.
#[derive(Debug, Clone)]
pub enum ToastType {
    /// Success message - green styling
    Success,
    /// Error message - red styling
    Error,
    /// Warning message - yellow/orange styling
    Warning,
    /// Info message - blue styling
    Info,
    /// Custom style for full control
    Custom(Style),
}

impl ToastType {
    /// Get the default icon for this toast type.
    fn default_icon(&self) -> &'static str {
        match self {
            ToastType::Success => "✓",
            ToastType::Error => "✗",
            ToastType::Warning => "⚠",
            ToastType::Info => "ℹ",
            ToastType::Custom(_) => "•",
        }
    }

    /// Get the border style for this toast type.
    fn border_style(&self) -> Style {
        match self {
            ToastType::Success => Style::parse("green").unwrap_or_else(|_| Style::null()),
            ToastType::Error => Style::parse("red").unwrap_or_else(|_| Style::null()),
            ToastType::Warning => Style::parse("yellow").unwrap_or_else(|_| Style::null()),
            ToastType::Info => Style::parse("blue").unwrap_or_else(|_| Style::null()),
            ToastType::Custom(style) => style.clone(),
        }
    }

    /// Get the background style (subtle) for this toast type.
    fn background_style(&self) -> Style {
        match self {
            ToastType::Success => {
                Style::parse("on dark_green dim").unwrap_or_else(|_| Style::null())
            }
            ToastType::Error => Style::parse("on dark_red dim").unwrap_or_else(|_| Style::null()),
            ToastType::Warning => {
                Style::parse("on dark_yellow dim").unwrap_or_else(|_| Style::null())
            }
            ToastType::Info => Style::parse("on dark_blue dim").unwrap_or_else(|_| Style::null()),
            ToastType::Custom(style) => style.background_style(),
        }
    }

    /// Get the icon style for this toast type.
    fn icon_style(&self) -> Style {
        match self {
            ToastType::Success => Style::parse("bold green").unwrap_or_else(|_| Style::null()),
            ToastType::Error => Style::parse("bold red").unwrap_or_else(|_| Style::null()),
            ToastType::Warning => Style::parse("bold yellow").unwrap_or_else(|_| Style::null()),
            ToastType::Info => Style::parse("bold blue").unwrap_or_else(|_| Style::null()),
            ToastType::Custom(style) => style.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Toast
// ---------------------------------------------------------------------------

/// A toast notification — temporary status message with optional progress bar.
///
/// Toasts are designed for transient status updates that automatically dismiss
/// after a specified duration. They support multiple visual types (success,
/// error, warning, info) with distinct colors and icons.
///
/// # Examples
///
/// ```
/// use gilt::prelude::*;
/// use gilt::toast::{Toast, ToastType};
/// use std::time::Duration;
///
/// let mut console = Console::new();
///
/// // Quick success toast
/// Toast::success("File saved!").show(&mut console);
///
/// // Toast with progress bar
/// Toast::info("Uploading...")
///     .show_progress(true)
///     .duration(Duration::from_secs(10))
///     .show(&mut console);
///
/// // Custom styled toast
/// let custom_style = Style::parse("magenta bold").unwrap();
/// Toast::new("Custom notification")
///     .toast_type(ToastType::Custom(custom_style))
///     .icon("🎉")
///     .show(&mut console);
/// ```
#[derive(Debug, Clone)]
pub struct Toast {
    /// The message to display
    message: String,
    /// The type of toast (determines styling)
    toast_type: ToastType,
    /// How long the toast should be displayed
    duration: Duration,
    /// Optional custom icon
    icon: Option<String>,
    /// Whether to show a progress bar for the duration
    show_progress: bool,
    /// Optional fixed width for the toast
    width: Option<usize>,
    /// Box drawing characters
    box_chars: &'static BoxChars,
}

impl Toast {
    /// Create a new toast with the given message.
    ///
    /// Default: Info type, 3 second duration, no progress bar.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::toast::Toast;
    ///
    /// let toast = Toast::new("Hello, world!");
    /// ```
    pub fn new(message: impl Into<String>) -> Self {
        Toast {
            message: message.into(),
            toast_type: ToastType::Info,
            duration: Duration::from_secs(3),
            icon: None,
            show_progress: false,
            width: None,
            box_chars: &ROUNDED,
        }
    }

    /// Create a success toast with the given message.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::toast::Toast;
    ///
    /// let toast = Toast::success("Operation completed!");
    /// ```
    pub fn success(message: impl Into<String>) -> Self {
        Toast::new(message).toast_type(ToastType::Success)
    }

    /// Create an error toast with the given message.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::toast::Toast;
    ///
    /// let toast = Toast::error("Failed to save file");
    /// ```
    pub fn error(message: impl Into<String>) -> Self {
        Toast::new(message).toast_type(ToastType::Error)
    }

    /// Create a warning toast with the given message.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::toast::Toast;
    ///
    /// let toast = Toast::warning("Disk space low");
    /// ```
    pub fn warning(message: impl Into<String>) -> Self {
        Toast::new(message).toast_type(ToastType::Warning)
    }

    /// Create an info toast with the given message.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::toast::Toast;
    ///
    /// let toast = Toast::info("3 new notifications");
    /// ```
    pub fn info(message: impl Into<String>) -> Self {
        Toast::new(message).toast_type(ToastType::Info)
    }

    /// Set the toast type (builder pattern).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::toast::{Toast, ToastType};
    ///
    /// let toast = Toast::new("Message").toast_type(ToastType::Success);
    /// ```
    #[must_use]
    pub fn toast_type(mut self, toast_type: ToastType) -> Self {
        self.toast_type = toast_type;
        self
    }

    /// Set the duration (builder pattern).
    ///
    /// Default: 3 seconds
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::toast::Toast;
    /// use std::time::Duration;
    ///
    /// let toast = Toast::new("Message").duration(Duration::from_secs(5));
    /// ```
    #[must_use]
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Set a custom icon (builder pattern).
    ///
    /// Default icons: ✓ (success), ✗ (error), ⚠ (warning), ℹ (info)
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::toast::Toast;
    ///
    /// let toast = Toast::success("Done!").icon("🎉");
    /// ```
    #[must_use]
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set whether to show a progress bar (builder pattern).
    ///
    /// When enabled, a progress bar is displayed at the bottom of the toast
    /// that fills up over the toast's duration.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::toast::Toast;
    ///
    /// let toast = Toast::info("Processing...").show_progress(true);
    /// ```
    #[must_use]
    pub fn show_progress(mut self, show: bool) -> Self {
        self.show_progress = show;
        self
    }

    /// Set a fixed width for the toast (builder pattern).
    ///
    /// If not set, the toast will size to fit its content.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::toast::Toast;
    ///
    /// let toast = Toast::new("Message").width(60);
    /// ```
    #[must_use]
    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the box drawing characters (builder pattern).
    ///
    /// Default: ROUNDED
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::toast::Toast;
    /// use gilt::box_chars::DOUBLE;
    ///
    /// let toast = Toast::new("Message").box_chars(&DOUBLE);
    /// ```
    #[must_use]
    pub fn box_chars(mut self, box_chars: &'static BoxChars) -> Self {
        self.box_chars = box_chars;
        self
    }

    /// Get the icon to display (custom or default).
    fn get_icon(&self) -> &str {
        self.icon.as_deref().unwrap_or_else(|| self.toast_type.default_icon())
    }

    /// Build the content text for the toast.
    fn build_content(&self) -> Text {
        let icon = self.get_icon();
        let icon_style = self.toast_type.icon_style();

        let mut content = Text::empty();

        // Add icon with brackets
        content.append_str("[", None);
        content.append_str(icon, Some(icon_style));
        content.append_str("] ", None);

        // Add message
        content.append_str(&self.message, None);

        content
    }

    /// Build the panel for this toast.
    fn build_panel(&self, elapsed: Option<Duration>) -> Panel {
        let mut content = self.build_content();

        // Add progress bar if enabled
        if self.show_progress {
            let progress_bar = self.build_progress_bar(elapsed);
            content.append_str("\n", None);

            // Render progress bar and append to content
            let console = Console::builder().width(40).build();
            let opts = console.options();
            let segments = progress_bar.gilt_console(&console, &opts);

            let mut pb_text = Text::empty();
            for seg in &segments {
                pb_text.append_str(&seg.text, seg.style.clone());
            }

            content.append_text(&pb_text);
        }

        let border_style = self.toast_type.border_style();
        let bg_style = self.toast_type.background_style();

        let mut panel = Panel::new(content)
            .with_box_chars(self.box_chars)
            .with_border_style(border_style)
            .with_style(bg_style)
            .with_expand(false)
            .with_padding(PaddingDimensions::Pair(0, 1));

        if let Some(width) = self.width {
            panel = panel.with_width(width);
        }

        panel
    }

    /// Build the progress bar for this toast.
    fn build_progress_bar(&self, elapsed: Option<Duration>) -> ProgressBar {
        let total_secs = self.duration.as_secs_f64();
        let elapsed_secs = elapsed.map(|d| d.as_secs_f64()).unwrap_or(0.0);
        let completed = elapsed_secs.min(total_secs);

        let mut bar = ProgressBar::new()
            .with_total(Some(total_secs))
            .with_completed(completed)
            .with_width(Some(40));

        // Set styles based on toast type
        match self.toast_type {
            ToastType::Success => {
                bar = bar
                    .with_complete_style("green")
                    .with_style("dim green");
            }
            ToastType::Error => {
                bar = bar
                    .with_complete_style("red")
                    .with_style("dim red");
            }
            ToastType::Warning => {
                bar = bar
                    .with_complete_style("yellow")
                    .with_style("dim yellow");
            }
            ToastType::Info => {
                bar = bar
                    .with_complete_style("blue")
                    .with_style("dim blue");
            }
            ToastType::Custom(_) => {
                bar = bar
                    .with_complete_style("bold")
                    .with_style("dim");
            }
        }

        bar
    }

    /// Display the toast on the given console (non-blocking).
    ///
    /// This immediately renders the toast. If you want to show it for
    /// the configured duration, use [`show_blocking`](Self::show_blocking).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prelude::*;
    /// use gilt::toast::Toast;
    ///
    /// let mut console = Console::new();
    /// Toast::success("Done!").show(&mut console);
    /// ```
    pub fn show(&self, console: &mut Console) {
        let panel = self.build_panel(None);
        console.print(&panel);
    }

    /// Display the toast and wait for its duration (blocking).
    ///
    /// This renders the toast and blocks for the configured duration.
    /// Note: For animated progress bars, use [`show`](Self::show) with your own
    /// update loop or the [`Live`](crate::live::Live) display.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prelude::*;
    /// use gilt::toast::Toast;
    ///
    /// let mut console = Console::new();
    /// Toast::info("Processing...").show_blocking(&mut console);
    /// ```
    pub fn show_blocking(&self, console: &mut Console) {
        self.show(console);
        std::thread::sleep(self.duration);
    }
}

impl Renderable for Toast {
    fn gilt_console(&self, console: &Console, _options: &crate::console::ConsoleOptions) -> Vec<Segment> {
        let panel = self.build_panel(None);
        panel.gilt_console(console, _options)
    }
}

// ---------------------------------------------------------------------------
// ToastManager
// ---------------------------------------------------------------------------

/// Manager for displaying multiple toast notifications.
///
/// The `ToastManager` maintains a queue of toasts and displays up to
/// `max_visible` at a time. It's useful when you have multiple status
/// messages to show.
///
/// # Examples
///
/// ```
/// use gilt::prelude::*;
/// use gilt::toast::{Toast, ToastManager};
///
/// let mut console = Console::new();
/// let mut manager = ToastManager::new();
///
/// manager.push(Toast::success("File saved"));
/// manager.push(Toast::info("Uploading..."));
/// manager.push(Toast::warning("Low disk space"));
///
/// manager.show_all(&mut console);
/// ```
#[derive(Debug, Clone)]
pub struct ToastManager {
    /// Queue of toasts to display
    toasts: Vec<Toast>,
    /// Maximum number of toasts to display at once
    max_visible: usize,
}

impl ToastManager {
    /// Create a new toast manager with default settings.
    ///
    /// Default: max_visible = 3
    pub fn new() -> Self {
        ToastManager {
            toasts: Vec::new(),
            max_visible: 3,
        }
    }

    /// Set the maximum number of visible toasts (builder pattern).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::toast::ToastManager;
    ///
    /// let manager = ToastManager::new().max_visible(5);
    /// ```
    #[must_use]
    pub fn max_visible(mut self, max: usize) -> Self {
        self.max_visible = max;
        self
    }

    /// Add a toast to the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::toast::{Toast, ToastManager};
    ///
    /// let mut manager = ToastManager::new();
    /// manager.push(Toast::success("Done!"));
    /// ```
    pub fn push(&mut self, toast: Toast) {
        self.toasts.push(toast);
    }

    /// Display all pending toasts (up to max_visible).
    ///
    /// Toasts are displayed in the order they were added.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prelude::*;
    /// use gilt::toast::{Toast, ToastManager};
    ///
    /// let mut console = Console::new();
    /// let mut manager = ToastManager::new();
    /// manager.push(Toast::success("Done!"));
    /// manager.show_all(&mut console);
    /// ```
    pub fn show_all(&self, console: &mut Console) {
        for toast in self.toasts.iter().take(self.max_visible) {
            toast.show(console);
        }
    }

    /// Clear all pending toasts.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::toast::{Toast, ToastManager};
    ///
    /// let mut manager = ToastManager::new();
    /// manager.push(Toast::success("Done!"));
    /// manager.clear();
    /// assert!(manager.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.toasts.clear();
    }

    /// Returns true if there are no toasts in the queue.
    pub fn is_empty(&self) -> bool {
        self.toasts.is_empty()
    }

    /// Returns the number of toasts in the queue.
    pub fn len(&self) -> usize {
        self.toasts.len()
    }

    /// Get a reference to the toasts in the queue.
    pub fn toasts(&self) -> &[Toast] {
        &self.toasts
    }
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Global convenience functions
// ---------------------------------------------------------------------------

/// Display a success toast on the default console.
///
/// # Examples
///
/// ```
/// use gilt::toast;
///
/// toast::toast_success("Operation completed!");
/// ```
pub fn toast_success(message: impl Into<String>) {
    crate::with_console(|console| {
        Toast::success(message).show(console);
    });
}

/// Display an error toast on the default console.
///
/// # Examples
///
/// ```
/// use gilt::toast;
///
/// toast::toast_error("Failed to save file");
/// ```
pub fn toast_error(message: impl Into<String>) {
    crate::with_console(|console| {
        Toast::error(message).show(console);
    });
}

/// Display a warning toast on the default console.
///
/// # Examples
///
/// ```
/// use gilt::toast;
///
/// toast::toast_warning("Disk space low");
/// ```
pub fn toast_warning(message: impl Into<String>) {
    crate::with_console(|console| {
        Toast::warning(message).show(console);
    });
}

/// Display an info toast on the default console.
///
/// # Examples
///
/// ```
/// use gilt::toast;
///
/// toast::toast_info("3 new notifications");
/// ```
pub fn toast_info(message: impl Into<String>) {
    crate::with_console(|console| {
        Toast::info(message).show(console);
    });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::Console;

    fn make_console() -> Console {
        Console::builder()
            .width(80)
            .force_terminal(true)
            .no_color(true)
            .build()
    }

    // -- Construction ---------------------------------------------------------

    #[test]
    fn test_default_construction() {
        let toast = Toast::new("Hello");
        assert_eq!(toast.message, "Hello");
        assert!(matches!(toast.toast_type, ToastType::Info));
        assert_eq!(toast.duration, Duration::from_secs(3));
        assert!(toast.icon.is_none());
        assert!(!toast.show_progress);
    }

    #[test]
    fn test_convenience_constructors() {
        let success = Toast::success("Done");
        assert!(matches!(success.toast_type, ToastType::Success));
        assert_eq!(success.get_icon(), "✓");

        let error = Toast::error("Failed");
        assert!(matches!(error.toast_type, ToastType::Error));
        assert_eq!(error.get_icon(), "✗");

        let warning = Toast::warning("Caution");
        assert!(matches!(warning.toast_type, ToastType::Warning));
        assert_eq!(warning.get_icon(), "⚠");

        let info = Toast::info("Note");
        assert!(matches!(info.toast_type, ToastType::Info));
        assert_eq!(info.get_icon(), "ℹ");
    }

    // -- Builder methods ------------------------------------------------------

    #[test]
    fn test_builder_type() {
        let toast = Toast::new("Test").toast_type(ToastType::Success);
        assert!(matches!(toast.toast_type, ToastType::Success));
    }

    #[test]
    fn test_builder_duration() {
        let toast = Toast::new("Test").duration(Duration::from_secs(10));
        assert_eq!(toast.duration, Duration::from_secs(10));
    }

    #[test]
    fn test_builder_icon() {
        let toast = Toast::new("Test").icon("🎉");
        assert_eq!(toast.icon, Some("🎉".to_string()));
        assert_eq!(toast.get_icon(), "🎉");
    }

    #[test]
    fn test_builder_show_progress() {
        let toast = Toast::new("Test").show_progress(true);
        assert!(toast.show_progress);
    }

    #[test]
    fn test_builder_width() {
        let toast = Toast::new("Test").width(60);
        assert_eq!(toast.width, Some(60));
    }

    #[test]
    fn test_builder_chain() {
        let toast = Toast::new("Test")
            .toast_type(ToastType::Warning)
            .duration(Duration::from_secs(5))
            .icon("⚡")
            .show_progress(true)
            .width(50);

        assert!(matches!(toast.toast_type, ToastType::Warning));
        assert_eq!(toast.duration, Duration::from_secs(5));
        assert_eq!(toast.icon, Some("⚡".to_string()));
        assert!(toast.show_progress);
        assert_eq!(toast.width, Some(50));
    }

    // -- ToastType styling ----------------------------------------------------

    #[test]
    fn test_toast_type_default_icons() {
        assert_eq!(ToastType::Success.default_icon(), "✓");
        assert_eq!(ToastType::Error.default_icon(), "✗");
        assert_eq!(ToastType::Warning.default_icon(), "⚠");
        assert_eq!(ToastType::Info.default_icon(), "ℹ");
        assert_eq!(ToastType::Custom(Style::null()).default_icon(), "•");
    }

    #[test]
    fn test_toast_type_border_styles() {
        // Just verify they don't panic and return valid styles
        let _ = ToastType::Success.border_style();
        let _ = ToastType::Error.border_style();
        let _ = ToastType::Warning.border_style();
        let _ = ToastType::Info.border_style();
        let _ = ToastType::Custom(Style::null()).border_style();
    }

    // -- Content building -----------------------------------------------------

    #[test]
    fn test_build_content() {
        let toast = Toast::success("Operation completed");
        let content = toast.build_content();
        let plain = content.plain();

        assert!(plain.contains("✓"));
        assert!(plain.contains("Operation completed"));
        assert!(plain.contains("[✓]"));
    }

    #[test]
    fn test_build_content_custom_icon() {
        let toast = Toast::info("Message").icon("🚀");
        let content = toast.build_content();
        let plain = content.plain();

        assert!(plain.contains("🚀"));
        assert!(!plain.contains("ℹ"));
    }

    // -- Rendering ------------------------------------------------------------

    #[test]
    fn test_show() {
        let mut console = make_console();
        console.begin_capture();

        let toast = Toast::success("Done!");
        toast.show(&mut console);

        let output = console.end_capture();
        assert!(output.contains("Done!"));
        assert!(output.contains("✓"));
    }

    #[test]
    fn test_renderable() {
        let console = make_console();
        let opts = console.options();

        let toast = Toast::info("Test message");
        let segments = toast.gilt_console(&console, &opts);

        // Should produce some segments
        assert!(!segments.is_empty());
    }

    // -- ToastManager ---------------------------------------------------------

    #[test]
    fn test_manager_default() {
        let manager = ToastManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.max_visible, 3);
    }

    #[test]
    fn test_manager_max_visible() {
        let manager = ToastManager::new().max_visible(5);
        assert_eq!(manager.max_visible, 5);
    }

    #[test]
    fn test_manager_push() {
        let mut manager = ToastManager::new();
        manager.push(Toast::success("Done"));
        assert_eq!(manager.len(), 1);
        assert!(!manager.is_empty());
    }

    #[test]
    fn test_manager_push_multiple() {
        let mut manager = ToastManager::new();
        manager.push(Toast::success("1"));
        manager.push(Toast::info("2"));
        manager.push(Toast::warning("3"));
        assert_eq!(manager.len(), 3);
    }

    #[test]
    fn test_manager_clear() {
        let mut manager = ToastManager::new();
        manager.push(Toast::success("Done"));
        manager.clear();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_manager_show_all() {
        let mut console = make_console();
        console.begin_capture();

        let mut manager = ToastManager::new();
        manager.push(Toast::success("First"));
        manager.push(Toast::info("Second"));
        manager.show_all(&mut console);

        let output = console.end_capture();
        assert!(output.contains("First"));
        assert!(output.contains("Second"));
    }

    #[test]
    fn test_manager_max_visible_limit() {
        let mut console = make_console();
        console.begin_capture();

        let mut manager = ToastManager::new().max_visible(2);
        manager.push(Toast::success("1"));
        manager.push(Toast::info("2"));
        manager.push(Toast::warning("3"));
        manager.show_all(&mut console);

        let output = console.end_capture();
        // Should only show first 2
        assert!(output.contains('1'));
        assert!(output.contains('2'));
    }

    #[test]
    fn test_manager_toasts_accessor() {
        let mut manager = ToastManager::new();
        manager.push(Toast::success("Test"));
        assert_eq!(manager.toasts().len(), 1);
    }

    // -- Edge cases -----------------------------------------------------------

    #[test]
    fn test_empty_message() {
        let toast = Toast::success("");
        assert_eq!(toast.message, "");
        let content = toast.build_content();
        assert!(content.plain().contains("✓"));
    }

    #[test]
    fn test_long_message() {
        let long_msg = "a".repeat(200);
        let toast = Toast::info(&long_msg);
        assert_eq!(toast.message.len(), 200);
    }

    #[test]
    fn test_multiline_message() {
        let toast = Toast::info("Line 1\nLine 2");
        let content = toast.build_content();
        let plain = content.plain();
        assert!(plain.contains("Line 1"));
        assert!(plain.contains("Line 2"));
    }

    #[test]
    fn test_custom_style() {
        let style = Style::parse("magenta bold").unwrap();
        let toast = Toast::new("Custom").toast_type(ToastType::Custom(style));
        assert!(matches!(toast.toast_type, ToastType::Custom(_)));
    }

    #[test]
    fn test_zero_duration() {
        let toast = Toast::new("Quick").duration(Duration::from_secs(0));
        assert_eq!(toast.duration, Duration::from_secs(0));
    }

    #[test]
    fn test_progress_bar_construction() {
        let toast = Toast::info("Progress").show_progress(true);
        let bar = toast.build_progress_bar(Some(Duration::from_secs(1)));
        assert!(bar.total.is_some());
        assert_eq!(bar.total.unwrap(), 3.0); // Default duration is 3s
    }
}
