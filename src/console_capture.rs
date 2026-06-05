//! Capture-mode methods for [`Console`]. Split out of console.rs in v1.3
//! Phase 4. The methods stay attached to `Console` via a separate
//! `impl Console` block; callers still write `console.begin_capture()`.

use crate::console::{Console, Renderable};
use crate::text::Text;

impl Console {
    /// Render a [`Renderable`] widget into a [`Text`] by capturing its
    /// ANSI output through this console. Used by `Live::from_renderable`,
    /// `Panel::from_renderable`, and the `Columns` widget render path to
    /// bridge non-Text renderables into Text-only consumers without each
    /// caller re-implementing the capture roundtrip.
    pub fn render_widget_to_text<R: Renderable + ?Sized>(&mut self, renderable: &R) -> Text {
        self.begin_capture();
        self.print(renderable);
        Text::from_ansi(&self.end_capture())
    }

    /// Begin capturing output. Subsequent writes go to the capture buffer
    /// instead of the terminal.
    ///
    /// Call [`end_capture`](Console::end_capture) to retrieve the captured output
    /// as a string and resume normal output.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let mut console = Console::builder().width(80).no_color(true).markup(false).build();
    /// console.begin_capture();
    /// console.print_text("captured");
    /// let output = console.end_capture();
    /// assert!(output.contains("captured"));
    /// ```
    pub fn begin_capture(&mut self) {
        self.capture_buffer = Some(Vec::new());
    }

    /// End capturing and return the captured output as a rendered string.
    ///
    /// Returns all output written since [`begin_capture`](Console::begin_capture)
    /// was called, rendered through the console's color system.
    pub fn end_capture(&mut self) -> String {
        let segments = self.capture_buffer.take().unwrap_or_default();
        self.render_buffer(&segments)
    }

    /// Whether the console is currently in capture mode.
    pub fn is_capturing(&self) -> bool {
        self.capture_buffer.is_some()
    }

    /// Begin capturing output and return a RAII guard.
    ///
    /// The guard holds a `&mut Console` and calls [`end_capture`] on [`Drop`],
    /// so capture state never leaks under `?`/panic.
    ///
    /// Use [`CaptureGuard::end`] to end capture early and retrieve the
    /// captured text without waiting for the guard to drop.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let mut console = Console::builder()
    ///     .width(80)
    ///     .no_color(true)
    ///     .markup(false)
    ///     .build();
    ///
    /// let text = {
    ///     let mut guard = console.capture_guard();
    ///     guard.print_text("hello from guard");
    ///     guard.end() // ends capture and returns the captured text
    /// };
    /// assert!(text.contains("hello from guard"));
    ///
    /// // After the guard is consumed, capture is no longer active.
    /// assert!(!console.is_capturing());
    /// ```
    ///
    /// Panic-safe: if the block panics the guard's `Drop` calls `end_capture`
    /// so the console is always left in a consistent state.
    pub fn capture_guard(&mut self) -> CaptureGuard<'_> {
        self.begin_capture();
        CaptureGuard { console: self }
    }

    /// Enter the alternate screen and return a RAII guard.
    ///
    /// On creation the guard:
    /// 1. Enters the alternate screen buffer (`EnableAltScreen`).
    /// 2. Hides the cursor.
    ///
    /// On [`Drop`] (or when the guard value is discarded) the guard:
    /// 1. Shows the cursor.
    /// 2. Exits the alternate screen buffer (`DisableAltScreen`).
    ///
    /// This mirrors the existing `enter_screen`/`exit_screen` pair but is
    /// panic-safe: the exit sequences are always emitted.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gilt::console::Console;
    ///
    /// let mut console = Console::new();
    ///
    /// {
    ///     let mut guard = console.screen_guard();
    ///     // We are now in the alternate screen with the cursor hidden.
    ///     // Use the guard's DerefMut to access Console methods.
    ///     guard.print_text("full-screen UI here");
    /// } // guard drops → cursor restored, primary screen re-activated
    /// ```
    pub fn screen_guard(&mut self) -> ScreenGuard<'_> {
        self.enter_screen(true);
        ScreenGuard { console: self }
    }
}

// ---------------------------------------------------------------------------
// CaptureGuard
// ---------------------------------------------------------------------------

/// RAII guard returned by [`Console::capture_guard`].
///
/// Holds a `&mut Console` and calls [`end_capture`](Console::end_capture) on
/// [`Drop`], ensuring capture state never leaks under early returns or panics.
///
/// Access the console's methods directly through [`std::ops::DerefMut`]:
///
/// ```
/// use gilt::console::Console;
///
/// let mut console = Console::builder().width(80).no_color(true).markup(false).build();
/// let mut guard = console.capture_guard();
/// guard.print_text("inside capture");
/// let text = guard.end();
/// assert!(text.contains("inside capture"));
/// ```
pub struct CaptureGuard<'a> {
    console: &'a mut Console,
}

impl CaptureGuard<'_> {
    /// End capture early and return the captured text.
    ///
    /// Consumes the guard (so it will not run [`Drop`] again) and calls
    /// [`end_capture`](Console::end_capture) immediately.
    pub fn end(self) -> String {
        let text = self.console.end_capture();
        // Mark the guard as already ended so Drop is a no-op.
        // We achieve this by storing a sentinel: set capture_buffer back to
        // None (end_capture already did that) — Drop calling end_capture again
        // on a None buffer just returns an empty string and is harmless, but
        // setting `finished` avoids even that call.
        //
        // We use std::mem::forget here so Drop never runs.
        // SAFETY: the guard holds no heap resources of its own; the Console
        // reference is valid for 'a. forgetting the guard simply skips the
        // Drop — no memory leak.
        let text_owned = text;
        std::mem::forget(self);
        text_owned
    }
}

impl Drop for CaptureGuard<'_> {
    fn drop(&mut self) {
        // Silently end capture so state is always cleaned up.
        // If `end()` was already called the buffer is None and this is a no-op.
        self.console.end_capture();
    }
}

impl std::ops::Deref for CaptureGuard<'_> {
    type Target = Console;
    fn deref(&self) -> &Console {
        self.console
    }
}

impl std::ops::DerefMut for CaptureGuard<'_> {
    fn deref_mut(&mut self) -> &mut Console {
        self.console
    }
}

// ---------------------------------------------------------------------------
// ScreenGuard
// ---------------------------------------------------------------------------

/// RAII guard returned by [`Console::screen_guard`].
///
/// Enters the alternate screen (and hides the cursor) on creation; restores
/// both on [`Drop`]. Panic-safe: the exit sequences are always emitted.
///
/// ```no_run
/// use gilt::console::Console;
///
/// let mut console = Console::new();
/// {
///     let _guard = console.screen_guard();
///     // alternate screen is active; use _guard.method() to call Console methods
/// }
/// // primary screen restored
/// ```
pub struct ScreenGuard<'a> {
    console: &'a mut Console,
}

impl Drop for ScreenGuard<'_> {
    fn drop(&mut self) {
        self.console.exit_screen(true);
    }
}

impl std::ops::Deref for ScreenGuard<'_> {
    type Target = Console;
    fn deref(&self) -> &Console {
        self.console
    }
}

impl std::ops::DerefMut for ScreenGuard<'_> {
    fn deref_mut(&mut self) -> &mut Console {
        self.console
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_console() -> Console {
        Console::builder()
            .width(80)
            .height(25)
            .quiet(false)
            .markup(false)
            .no_color(true)
            .force_terminal(true)
            .build()
    }

    // -- CaptureGuard -------------------------------------------------------

    /// `CaptureGuard::end` returns the captured text and stops capturing.
    #[test]
    fn capture_guard_end_returns_text() {
        let mut console = test_console();
        let mut guard = console.capture_guard();
        guard.print_text("guard_text");
        let captured = guard.end();
        assert!(
            captured.contains("guard_text"),
            "expected 'guard_text' in output, got: {:?}",
            captured
        );
    }

    /// After `CaptureGuard` drops (without calling `end`), `is_capturing` is
    /// `false` and a new `begin_capture` cycle works normally.
    #[test]
    fn capture_guard_drop_restores_state() {
        let mut console = test_console();
        // Use the guard and drop it without calling end().
        {
            let _guard = console.capture_guard();
            // Guard is alive here — just let it drop.
        } // guard drops here
        assert!(
            !console.is_capturing(),
            "capture should be inactive after guard drop"
        );

        // A fresh capture cycle should still work.
        console.begin_capture();
        console.print_text("after_guard");
        let text = console.end_capture();
        assert!(text.contains("after_guard"));
    }

    /// `CaptureGuard` allows printing through `DerefMut`.
    #[test]
    fn capture_guard_deref_mut_works() {
        let mut console = test_console();
        let mut guard = console.capture_guard();
        guard.print_text("deref_mut_text");
        let text = guard.end();
        assert!(text.contains("deref_mut_text"));
    }

    // -- ScreenGuard --------------------------------------------------------

    /// After `ScreenGuard` drops the exit-alt-screen control was emitted and
    /// `is_alt_screen` is `false`.
    #[test]
    fn screen_guard_drop_exits_alt_screen() {
        let mut console = test_console();
        // Enter alt screen via the guard, then drop it.
        {
            let _guard = console.screen_guard();
            // Guard holds &mut Console — we can't inspect console here directly.
        } // guard drops
        assert!(
            !console.is_alt_screen,
            "expected alt-screen to be inactive after guard drop"
        );
    }

    /// `ScreenGuard` correctly transitions in → out and the exit-alt-screen
    /// escape sequence appears in the captured output.
    #[test]
    fn screen_guard_exit_sequence_emitted() {
        let mut console = Console::builder()
            .width(80)
            .height(25)
            .quiet(false)
            .markup(false)
            .no_color(true)
            .force_terminal(true)
            .build();

        console.begin_capture();
        {
            let _guard = console.screen_guard();
        }
        let captured = console.end_capture();

        // The exit-alt-screen sequence (\x1b[?1049l) must have been emitted.
        assert!(
            captured.contains("\x1b[?1049l"),
            "expected exit-alt-screen sequence in output, got: {:?}",
            captured
        );
    }
}
