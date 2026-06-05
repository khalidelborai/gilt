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
}
