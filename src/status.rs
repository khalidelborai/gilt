//! Status indicator with a spinner animation.
//!
//! Port of Python rich's `status.py`. Displays a status message alongside a
//! spinning animation, using a [`Live`] display for in-place terminal updates.
//!
//! # Examples
//!
//! ```
//! use gilt::status::Status;
//!
//! let mut status = Status::new("Loading...");
//! status.start();
//! status.update().status("Processing...").apply();
//! status.stop();
//! ```

use crate::console::Console;
use crate::live::{ConsoleRef, Live};
use crate::spinner::{Spinner, SpinnerError};
use crate::style::Style;
use crate::text::Text;

// ---------------------------------------------------------------------------
// StatusError
// ---------------------------------------------------------------------------

/// Error returned by Status operations.
#[derive(Debug)]
pub enum StatusError {
    /// The requested spinner name was not found.
    Spinner(SpinnerError),
}

impl std::fmt::Display for StatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusError::Spinner(e) => write!(f, "status spinner error: {}", e),
        }
    }
}

impl std::error::Error for StatusError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StatusError::Spinner(e) => Some(e),
        }
    }
}

impl From<SpinnerError> for StatusError {
    fn from(e: SpinnerError) -> Self {
        StatusError::Spinner(e)
    }
}

// ---------------------------------------------------------------------------
// StatusUpdate builder
// ---------------------------------------------------------------------------

/// A builder for applying selective updates to a [`Status`].
///
/// Obtained via [`Status::update`]. Call setter methods to stage changes,
/// then [`apply`](StatusUpdate::apply) to commit them.
pub struct StatusUpdate<'a> {
    status: &'a mut Status,
    new_status: Option<String>,
    new_spinner: Option<String>,
    new_spinner_style: Option<Style>,
    new_speed: Option<f64>,
}

impl<'a> StatusUpdate<'a> {
    /// Set a new status text.
    #[must_use]
    pub fn status(mut self, status: &str) -> Self {
        self.new_status = Some(status.to_string());
        self
    }

    /// Set a new spinner animation by name.
    #[must_use]
    pub fn spinner(mut self, name: &str) -> Self {
        self.new_spinner = Some(name.to_string());
        self
    }

    /// Set a new spinner style.
    #[must_use]
    pub fn spinner_style(mut self, style: Style) -> Self {
        self.new_spinner_style = Some(style);
        self
    }

    /// Set a new speed multiplier.
    #[must_use]
    pub fn speed(mut self, speed: f64) -> Self {
        self.new_speed = Some(speed);
        self
    }

    /// Apply the staged updates. Returns `Ok(())` on success, or an error
    /// if a new spinner name was invalid.
    pub fn apply(self) -> Result<(), StatusError> {
        // Apply simple property changes first.
        if let Some(ref text) = self.new_status {
            self.status.status_text = text.clone();
        }
        if let Some(style) = self.new_spinner_style {
            self.status.spinner_style = style;
        }
        if let Some(speed) = self.new_speed {
            self.status.speed = speed;
        }

        if let Some(ref spinner_name) = self.new_spinner {
            // Create a brand new spinner with the current properties.
            let mut spinner = Spinner::new(spinner_name)?;
            spinner = spinner
                .with_text(Text::new(&self.status.status_text, Style::null()))
                .with_style(self.status.spinner_style.clone())
                .with_speed(self.status.speed);
            self.status.spinner = spinner;

            // Push the new renderable to the live display.
            let text = render_spinner_snapshot(&self.status.spinner);
            self.status.live.update_renderable(text, true);
        } else {
            // Update the existing spinner in place.
            self.status.spinner.update(
                Some(Text::new(&self.status.status_text, Style::null())),
                Some(self.status.spinner_style.clone()),
                Some(self.status.speed),
            );
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Status
// ---------------------------------------------------------------------------

/// Displays a status indicator with a spinner animation.
///
/// `Status` combines a [`Spinner`] with a [`Live`] display to show an
/// animated status message in the terminal. The spinner and status text
/// can be updated at any time.
///
/// # RAII
///
/// `Status` implements [`Drop`], which calls [`stop`](Status::stop) to
/// ensure the live display is cleanly shut down when the value goes out
/// of scope.
///
/// # Examples
///
/// ```
/// use gilt::status::Status;
///
/// let mut status = Status::new("Downloading...");
/// status.start();
/// // ... do work ...
/// status.update().status("Compiling...").apply().unwrap();
/// // ... do more work ...
/// status.stop();
/// ```
pub struct Status {
    /// The current status text.
    pub status_text: String,
    /// The style applied to the spinner frame.
    pub spinner_style: Style,
    /// The speed multiplier for the spinner.
    pub speed: f64,
    /// The spinner animation.
    spinner: Spinner,
    /// The live display that handles in-place terminal rendering.
    live: Live,
}

/// Render a spinner at time 0 to produce a `Text` snapshot for the live display.
fn render_spinner_snapshot(spinner: &Spinner) -> Text {
    let mut spinner_clone = Spinner::new(&spinner.name).unwrap();
    spinner_clone = spinner_clone.with_speed(spinner.speed);
    if let Some(ref text) = spinner.text {
        spinner_clone = spinner_clone.with_text(text.clone());
    }
    if let Some(ref style) = spinner.style {
        spinner_clone = spinner_clone.with_style(style.clone());
    }
    spinner_clone.render(0.0)
}

impl Status {
    /// Create a new `Status` with default settings.
    ///
    /// Defaults:
    /// - Spinner: `"dots"`
    /// - Speed: `1.0`
    /// - Refresh per second: `12.5`
    /// - Spinner style: `Style::null()` (no special styling)
    ///
    /// # Panics
    ///
    /// Panics if the default spinner `"dots"` is not found in the spinner
    /// registry (this should never happen).
    pub fn new(status: &str) -> Self {
        Self::try_new(status, "dots", Style::null(), 1.0, 12.5)
            .expect("default spinner 'dots' must exist")
    }

    /// Try to create a new `Status`, returning an error if the spinner
    /// name is invalid.
    fn try_new(
        status: &str,
        spinner_name: &str,
        spinner_style: Style,
        speed: f64,
        refresh_per_second: f64,
    ) -> Result<Self, StatusError> {
        let spinner = Spinner::new(spinner_name)?
            .with_text(Text::new(status, Style::null()))
            .with_style(spinner_style.clone())
            .with_speed(speed);

        let renderable_text = render_spinner_snapshot(&spinner);
        let live = Live::new(renderable_text)
            .with_refresh_per_second(refresh_per_second)
            .with_transient(true);

        Ok(Status {
            status_text: status.to_string(),
            spinner_style,
            speed,
            spinner,
            live,
        })
    }

    /// Builder method: set the spinner animation by name.
    ///
    /// # Errors
    ///
    /// Returns `StatusError::Spinner` if the name is not found.
    pub fn with_spinner(mut self, name: &str) -> Result<Self, StatusError> {
        let spinner = Spinner::new(name)?
            .with_text(Text::new(&self.status_text, Style::null()))
            .with_style(self.spinner_style.clone())
            .with_speed(self.speed);
        self.spinner = spinner;
        let text = render_spinner_snapshot(&self.spinner);
        self.live.update_renderable(text, false);
        Ok(self)
    }

    /// Builder method: set the spinner style.
    #[must_use]
    pub fn with_spinner_style(mut self, style: Style) -> Self {
        self.spinner_style = style.clone();
        self.spinner =
            std::mem::replace(&mut self.spinner, Spinner::new("dots").unwrap()).with_style(style);
        self
    }

    /// Builder method: set the speed multiplier.
    #[must_use]
    pub fn with_speed(mut self, speed: f64) -> Self {
        self.speed = speed;
        self.spinner =
            std::mem::replace(&mut self.spinner, Spinner::new("dots").unwrap()).with_speed(speed);
        self
    }

    /// Builder method: set a custom console for the live display.
    #[must_use]
    pub fn with_console(mut self, console: Console) -> Self {
        // Rebuild live with the new console, preserving other settings.
        let renderable_text = render_spinner_snapshot(&self.spinner);
        self.live = Live::new(renderable_text)
            .with_console(console)
            .with_refresh_per_second(self.live.refresh_per_second)
            .with_transient(self.live.transient);
        self
    }

    /// Builder method: set the refresh rate (refreshes per second).
    #[must_use]
    pub fn with_refresh_per_second(mut self, rate: f64) -> Self {
        let renderable_text = render_spinner_snapshot(&self.spinner);
        self.live = Live::new(renderable_text)
            .with_refresh_per_second(rate)
            .with_transient(self.live.transient);
        self
    }

    /// Get a reference to the spinner.
    pub fn renderable(&self) -> &Spinner {
        &self.spinner
    }

    /// Get a reference to the console (from the live display).
    pub fn console(&self) -> ConsoleRef<'_> {
        self.live.console()
    }

    /// Begin an update to the status. Returns a builder that can change
    /// the status text, spinner, style, and speed.
    ///
    /// Call `.apply()` on the returned builder to commit the changes.
    pub fn update(&mut self) -> StatusUpdate<'_> {
        StatusUpdate {
            status: self,
            new_status: None,
            new_spinner: None,
            new_spinner_style: None,
            new_speed: None,
        }
    }

    /// Start the live display.
    pub fn start(&mut self) {
        self.live.start();
    }

    /// Stop the live display.
    pub fn stop(&mut self) {
        self.live.stop();
    }

    /// Check if the live display has been started.
    pub fn is_started(&self) -> bool {
        self.live.is_started()
    }
}

impl Drop for Status {
    fn drop(&mut self) {
        self.stop();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Default construction -----------------------------------------------

    #[test]
    fn test_default_construction() {
        let status = Status::new("Loading...");
        assert_eq!(status.status_text, "Loading...");
        assert!(status.spinner_style.is_null());
        assert_eq!(status.speed, 1.0);
        assert!(!status.is_started());
    }

    #[test]
    fn test_default_spinner_is_dots() {
        let status = Status::new("test");
        assert_eq!(status.spinner.name, "dots");
    }

    #[test]
    fn test_default_spinner_has_text() {
        let status = Status::new("Working");
        assert!(status.spinner.text.is_some());
        assert_eq!(status.spinner.text.as_ref().unwrap().plain(), "Working");
    }

    // -- Builder methods ----------------------------------------------------

    #[test]
    fn test_with_spinner() {
        let status = Status::new("test").with_spinner("line").unwrap();
        assert_eq!(status.spinner.name, "line");
    }

    #[test]
    fn test_with_spinner_invalid() {
        let result = Status::new("test").with_spinner("nonexistent_xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_with_spinner_preserves_text() {
        let status = Status::new("my status").with_spinner("line").unwrap();
        assert!(status.spinner.text.is_some());
        assert_eq!(status.spinner.text.as_ref().unwrap().plain(), "my status");
    }

    #[test]
    fn test_with_spinner_style() {
        let style = Style::parse("bold red").unwrap();
        let status = Status::new("test").with_spinner_style(style.clone());
        assert_eq!(status.spinner_style, style);
        assert_eq!(status.spinner.style, Some(style));
    }

    #[test]
    fn test_with_speed() {
        let status = Status::new("test").with_speed(2.0);
        assert_eq!(status.speed, 2.0);
        assert_eq!(status.spinner.speed, 2.0);
    }

    #[test]
    fn test_with_console() {
        let console = Console::builder().width(120).build();
        let status = Status::new("test").with_console(console);
        assert_eq!(status.console().width(), 120);
    }

    #[test]
    fn test_with_refresh_per_second() {
        let status = Status::new("test").with_refresh_per_second(30.0);
        assert_eq!(status.live.refresh_per_second, 30.0);
    }

    #[test]
    fn test_builder_chaining() {
        let style = Style::parse("bold").unwrap();
        let status = Status::new("test")
            .with_spinner_style(style.clone())
            .with_speed(3.0)
            .with_spinner("line")
            .unwrap();

        assert_eq!(status.spinner.name, "line");
        assert_eq!(status.spinner_style, style);
        assert_eq!(status.speed, 3.0);
    }

    // -- Update with new status text ----------------------------------------

    #[test]
    fn test_update_status_text() {
        let mut status = Status::new("old text");
        status.update().status("new text").apply().unwrap();
        assert_eq!(status.status_text, "new text");
        // Spinner text should also be updated
        assert_eq!(status.spinner.text.as_ref().unwrap().plain(), "new text");
    }

    #[test]
    fn test_update_status_text_preserves_spinner() {
        let mut status = Status::new("test").with_spinner("line").unwrap();
        status.update().status("changed").apply().unwrap();
        assert_eq!(status.spinner.name, "line");
        assert_eq!(status.status_text, "changed");
    }

    // -- Update with new spinner name ---------------------------------------

    #[test]
    fn test_update_spinner_name() {
        let mut status = Status::new("test");
        assert_eq!(status.spinner.name, "dots");
        status.update().spinner("line").apply().unwrap();
        assert_eq!(status.spinner.name, "line");
    }

    #[test]
    fn test_update_spinner_name_preserves_text() {
        let mut status = Status::new("keep me");
        status.update().spinner("line").apply().unwrap();
        assert!(status.spinner.text.is_some());
        assert_eq!(status.spinner.text.as_ref().unwrap().plain(), "keep me");
    }

    #[test]
    fn test_update_spinner_invalid_name() {
        let mut status = Status::new("test");
        let result = status.update().spinner("nonexistent_xyz").apply();
        assert!(result.is_err());
        // Spinner should remain unchanged on error
        assert_eq!(status.spinner.name, "dots");
    }

    // -- Update with new style ---------------------------------------------

    #[test]
    fn test_update_style() {
        let mut status = Status::new("test");
        let style = Style::parse("bold green").unwrap();
        status
            .update()
            .spinner_style(style.clone())
            .apply()
            .unwrap();
        assert_eq!(status.spinner_style, style);
    }

    #[test]
    fn test_update_style_applied_to_spinner_update() {
        let mut status = Status::new("test");
        let style = Style::parse("italic").unwrap();
        status
            .update()
            .spinner_style(style.clone())
            .apply()
            .unwrap();
        // The spinner's update method was called with the new style
        assert_eq!(status.spinner_style, style);
    }

    // -- Update with new speed ---------------------------------------------

    #[test]
    fn test_update_speed() {
        let mut status = Status::new("test");
        assert_eq!(status.speed, 1.0);
        status.update().speed(5.0).apply().unwrap();
        assert_eq!(status.speed, 5.0);
    }

    // -- Combined updates --------------------------------------------------

    #[test]
    fn test_update_multiple_fields() {
        let mut status = Status::new("original");
        let style = Style::parse("bold").unwrap();
        status
            .update()
            .status("changed")
            .speed(2.5)
            .spinner_style(style.clone())
            .apply()
            .unwrap();

        assert_eq!(status.status_text, "changed");
        assert_eq!(status.speed, 2.5);
        assert_eq!(status.spinner_style, style);
    }

    #[test]
    fn test_update_all_with_new_spinner() {
        let mut status = Status::new("original");
        let style = Style::parse("underline").unwrap();
        status
            .update()
            .status("new status")
            .spinner("line")
            .spinner_style(style.clone())
            .speed(4.0)
            .apply()
            .unwrap();

        assert_eq!(status.status_text, "new status");
        assert_eq!(status.spinner.name, "line");
        assert_eq!(status.spinner_style, style);
        assert_eq!(status.speed, 4.0);
        assert_eq!(status.spinner.text.as_ref().unwrap().plain(), "new status");
    }

    // -- Start/stop lifecycle ----------------------------------------------

    #[test]
    fn test_start_stop() {
        let mut status = Status::new("test");
        assert!(!status.is_started());
        status.start();
        assert!(status.is_started());
        status.stop();
        assert!(!status.is_started());
    }

    #[test]
    fn test_start_idempotent() {
        let mut status = Status::new("test");
        status.start();
        status.start(); // should not panic
        assert!(status.is_started());
        status.stop();
    }

    #[test]
    fn test_stop_idempotent() {
        let mut status = Status::new("test");
        status.stop(); // not started, should not panic
        assert!(!status.is_started());
    }

    #[test]
    fn test_stop_after_start() {
        let mut status = Status::new("test");
        status.start();
        assert!(status.is_started());
        status.stop();
        assert!(!status.is_started());
    }

    // -- Drop calls stop ---------------------------------------------------

    #[test]
    fn test_drop_calls_stop() {
        let mut status = Status::new("test");
        status.start();
        assert!(status.is_started());
        // Drop triggers stop via the Drop impl
        drop(status);
        // If we get here without panicking, drop worked correctly.
    }

    #[test]
    fn test_drop_when_not_started() {
        let status = Status::new("test");
        // Should not panic on drop when not started
        drop(status);
    }

    // -- Renderable accessor -----------------------------------------------

    #[test]
    fn test_renderable_returns_spinner() {
        let status = Status::new("test");
        let spinner = status.renderable();
        assert_eq!(spinner.name, "dots");
    }

    // -- Console accessor --------------------------------------------------

    #[test]
    fn test_console_accessor() {
        let status = Status::new("test");
        let _console = status.console();
    }

    #[test]
    fn test_console_from_builder() {
        let console = Console::builder().width(100).build();
        let status = Status::new("test").with_console(console);
        assert_eq!(status.console().width(), 100);
    }

    // -- Error display -----------------------------------------------------

    #[test]
    fn test_status_error_display() {
        let err = StatusError::Spinner(SpinnerError("test error".to_string()));
        let msg = format!("{}", err);
        assert!(msg.contains("test error"));
    }

    #[test]
    fn test_status_error_source() {
        let inner = SpinnerError("inner".to_string());
        let err = StatusError::Spinner(inner);
        let source = std::error::Error::source(&err);
        assert!(source.is_some());
    }

    // -- try_new -----------------------------------------------------------

    #[test]
    fn test_try_new_invalid_spinner() {
        let result = Status::try_new("test", "nonexistent_xyz", Style::null(), 1.0, 12.5);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_new_valid_spinner() {
        let result = Status::try_new("test", "line", Style::null(), 1.0, 12.5);
        assert!(result.is_ok());
        let status = result.unwrap();
        assert_eq!(status.spinner.name, "line");
    }

    // -- Update does not crash when started --------------------------------

    #[test]
    fn test_update_while_started() {
        let mut status = Status::new("running");
        status.start();
        status.update().status("still running").apply().unwrap();
        assert_eq!(status.status_text, "still running");
        status.stop();
    }

    #[test]
    fn test_update_spinner_while_started() {
        let mut status = Status::new("running");
        status.start();
        status.update().spinner("line").apply().unwrap();
        assert_eq!(status.spinner.name, "line");
        status.stop();
    }
}
