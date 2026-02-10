//! Live display module -- a terminal display that refreshes at regular intervals.
//!
//! Port of Python's `rich/live.py`. Provides a `Live` struct that can display
//! content that updates in-place using cursor movement control codes and an
//! optional background refresh thread.

pub mod live_render;
pub mod screen;

use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use crate::console::{Console, Renderable};
use crate::control::Control;
use crate::segment::Segment;
use crate::text::Text;

use self::live_render::{LiveRender, VerticalOverflowMethod};
use self::screen::Screen;

// ---------------------------------------------------------------------------
// SharedState -- data accessed by both the main thread and the refresh thread
// ---------------------------------------------------------------------------

/// Internal mutable state shared between the `Live` owner and the refresh thread.
struct SharedState {
    console: Console,
    live_render: LiveRender,
    renderable: Text,
    get_renderable: Option<Box<dyn Fn() -> Text + Send>>,
    screen: bool,
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Emit control-bearing segments to a console.
fn emit_control_segments(console: &mut Console, segments: &[Segment]) {
    for seg in segments {
        if let Some(ref codes) = seg.control {
            console.control(&Control::new(codes.clone()));
        }
    }
}

// ---------------------------------------------------------------------------
// Live
// ---------------------------------------------------------------------------

/// A live-updating terminal display that refreshes content at regular intervals.
///
/// `Live` renders a [`Text`] value to the terminal, hiding the cursor and
/// (optionally) using a background thread to repaint at a configurable rate.
/// When the display is stopped (explicitly via [`stop`](Live::stop) or
/// implicitly via [`Drop`]), the terminal state is restored.
///
/// # Examples
///
/// ```no_run
/// use gilt::live::Live;
/// use gilt::text::Text;
/// use gilt::style::Style;
///
/// let mut live = Live::new(Text::new("Loading...", Style::null()));
/// live.start();
/// live.update_renderable(Text::new("Done!", Style::null()), true);
/// live.stop();
/// ```
pub struct Live {
    state: Arc<Mutex<SharedState>>,
    auto_refresh: bool,
    /// Number of refreshes per second.
    pub refresh_per_second: f64,
    /// Whether the display clears on exit (transient mode).
    pub transient: bool,
    vertical_overflow: VerticalOverflowMethod,
    started: bool,
    refresh_thread: Option<thread::JoinHandle<()>>,
    stop_flag: Arc<(Mutex<bool>, Condvar)>,
}

impl Live {
    /// Create a new `Live` display for the given renderable.
    ///
    /// # Defaults
    /// - `auto_refresh`: `true`
    /// - `refresh_per_second`: `4.0`
    /// - `transient`: `false`
    /// - `screen`: `false`
    /// - `vertical_overflow`: [`VerticalOverflowMethod::Ellipsis`]
    pub fn new(renderable: Text) -> Self {
        let live_render = LiveRender::new(renderable.clone());
        let console = Console::new();

        let state = Arc::new(Mutex::new(SharedState {
            console,
            live_render,
            renderable,
            get_renderable: None,
            screen: false,
        }));

        Live {
            state,
            auto_refresh: true,
            refresh_per_second: 4.0,
            transient: false,
            vertical_overflow: VerticalOverflowMethod::Ellipsis,
            started: false,
            refresh_thread: None,
            stop_flag: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    // -- Builder methods ----------------------------------------------------

    /// Set the console to use for output (builder pattern).
    #[must_use]
    pub fn with_console(self, console: Console) -> Self {
        {
            let mut s = self.state.lock().unwrap();
            s.console = console;
        }
        self
    }

    /// Enable or disable auto-refresh (builder pattern).
    #[must_use]
    pub fn with_auto_refresh(mut self, auto_refresh: bool) -> Self {
        self.auto_refresh = auto_refresh;
        self
    }

    /// Set the refresh rate in refreshes per second (builder pattern).
    ///
    /// # Panics
    /// Panics if `rate` is not greater than zero.
    #[must_use]
    pub fn with_refresh_per_second(mut self, rate: f64) -> Self {
        assert!(rate > 0.0, "refresh_per_second must be > 0");
        self.refresh_per_second = rate;
        self
    }

    /// Enable or disable transient mode (builder pattern).
    ///
    /// In transient mode the last render is erased when the display stops.
    #[must_use]
    pub fn with_transient(mut self, transient: bool) -> Self {
        self.transient = transient;
        self
    }

    /// Enable or disable alternate screen mode (builder pattern).
    #[must_use]
    pub fn with_screen(self, screen: bool) -> Self {
        {
            let mut s = self.state.lock().unwrap();
            s.screen = screen;
        }
        self
    }

    /// Set the vertical overflow method (builder pattern).
    #[must_use]
    pub fn with_vertical_overflow(mut self, overflow: VerticalOverflowMethod) -> Self {
        self.vertical_overflow = overflow;
        {
            let mut s = self.state.lock().unwrap();
            s.live_render.vertical_overflow = overflow;
        }
        self
    }

    /// Set a callback that provides the renderable on each refresh (builder pattern).
    #[must_use]
    pub fn with_get_renderable<F>(self, f: F) -> Self
    where
        F: Fn() -> Text + Send + 'static,
    {
        {
            let mut s = self.state.lock().unwrap();
            s.get_renderable = Some(Box::new(f));
        }
        self
    }

    // -- Accessors ----------------------------------------------------------

    /// Get a reference to the console (locks internal state briefly and
    /// returns a value, because the console lives behind a Mutex).
    ///
    /// For simple width/height queries this clones the relevant fields.
    /// If you need prolonged access, prefer `with_console_mut`.
    pub fn console(&self) -> ConsoleRef<'_> {
        ConsoleRef {
            guard: self.state.lock().unwrap(),
        }
    }

    /// Get a mutable reference to the console.
    pub fn console_mut(&self) -> ConsoleRefMut<'_> {
        ConsoleRefMut {
            guard: self.state.lock().unwrap(),
        }
    }

    /// Whether the live display is currently running.
    pub fn is_started(&self) -> bool {
        self.started
    }

    /// Get a reference to the underlying `LiveRender` (locks internal state).
    pub fn live_render(&self) -> LiveRenderRef<'_> {
        LiveRenderRef {
            guard: self.state.lock().unwrap(),
        }
    }

    // -- Lifecycle ----------------------------------------------------------

    /// Start the live display.
    ///
    /// Hides the cursor, optionally enables the alternate screen, and spawns
    /// the background refresh thread if `auto_refresh` is enabled.
    ///
    /// Calling `start` on an already-started display is a no-op.
    pub fn start(&mut self) {
        if self.started {
            return;
        }
        self.started = true;

        // Reset stop flag for a fresh start.
        {
            let mut stopped = self.stop_flag.0.lock().unwrap();
            *stopped = false;
        }

        {
            let mut s = self.state.lock().unwrap();
            s.console.show_cursor(false);
            if s.screen {
                s.console.set_alt_screen(true);
            }
        }

        if self.auto_refresh {
            let flag = Arc::clone(&self.stop_flag);
            let state = Arc::clone(&self.state);
            let vertical_overflow = self.vertical_overflow;
            let interval = Duration::from_secs_f64(1.0 / self.refresh_per_second);

            let handle = thread::spawn(move || loop {
                let (lock, cvar) = &*flag;
                let stopped = lock.lock().unwrap();
                let result = cvar.wait_timeout(stopped, interval).unwrap();
                if *result.0 {
                    break;
                }
                drop(result);
                Self::do_refresh(&state, vertical_overflow);
            });
            self.refresh_thread = Some(handle);
        }
    }

    /// Stop the live display.
    ///
    /// Signals the refresh thread to exit and joins it, optionally erases the
    /// last render (transient mode), shows the cursor, and disables the
    /// alternate screen if it was enabled.
    ///
    /// Calling `stop` on an already-stopped display is a no-op.
    pub fn stop(&mut self) {
        if !self.started {
            return;
        }
        self.started = false;

        // Signal the refresh thread to stop.
        {
            let mut stopped = self.stop_flag.0.lock().unwrap();
            *stopped = true;
            self.stop_flag.1.notify_all();
        }

        // Join the refresh thread.
        if let Some(handle) = self.refresh_thread.take() {
            let _ = handle.join();
        }

        let mut s = self.state.lock().unwrap();

        // In transient mode, erase the last render.
        if self.transient {
            let segments = s.live_render.restore_cursor();
            emit_control_segments(&mut s.console, &segments);
        } else {
            // Move to a new line so the terminal prompt doesn't overlap
            // the last rendered content (do_refresh omits trailing newlines
            // to keep shape tracking accurate).
            s.console.write_segments(&[Segment::line()]);
        }

        // Restore terminal state.
        s.console.show_cursor(true);
        if s.screen {
            s.console.set_alt_screen(false);
        }
    }

    // -- Content management -------------------------------------------------

    /// Refresh the display with the current content.
    ///
    /// This acquires the shared state lock internally, so it is safe to call
    /// from any thread (the refresh thread calls this automatically).
    pub fn refresh(&self) {
        Self::do_refresh(&self.state, self.vertical_overflow);
    }

    /// Internal refresh implementation operating on shared state.
    fn do_refresh(state: &Arc<Mutex<SharedState>>, vertical_overflow: VerticalOverflowMethod) {
        let mut s = state.lock().unwrap();

        // Resolve the renderable: use callback if available, else stored.
        let renderable = match &s.get_renderable {
            Some(f) => f(),
            None => s.renderable.clone(),
        };

        // Update the live render with the resolved content.
        s.live_render.set_renderable(renderable.clone());
        s.live_render.vertical_overflow = vertical_overflow;

        if s.screen {
            // Screen mode: render through Screen which fills the whole alt-screen.
            let opts = s.console.options();
            let _render_segments = s.live_render.gilt_console(&s.console, &opts);
            let screen = Screen::new(renderable);
            s.console.print(&screen);
        } else {
            // Normal mode: render through LiveRender and write segments directly.
            // This ensures the shape tracking matches the actual output exactly.
            // We do NOT use console.print() because it adds a trailing newline,
            // which causes the tracked shape (N lines) to mismatch the actual
            // output (N+1 lines), leaking 1 line per refresh frame.
            let opts = s.console.options();
            
            // First render to compute shape (shape is stored in live_render)
            let render_segments = s.live_render.gilt_console(&s.console, &opts);
            
            // Now position cursor using the computed shape
            let position_segments = s.live_render.position_cursor();
            emit_control_segments(&mut s.console, &position_segments);
            
            s.console.write_segments(&render_segments);
        }
    }

    /// Update the renderable content.
    ///
    /// If `refresh` is `true`, the display is repainted immediately.
    pub fn update_renderable(&mut self, renderable: Text, refresh: bool) {
        {
            let mut s = self.state.lock().unwrap();
            s.live_render.set_renderable(renderable.clone());
            s.renderable = renderable;
        }
        if refresh {
            self.refresh();
        }
    }

    /// Alias for [`update_renderable`](Live::update_renderable).
    pub fn update(&mut self, renderable: Text, refresh: bool) {
        self.update_renderable(renderable, refresh);
    }

    /// Get a clone of the current renderable.
    pub fn renderable(&self) -> Text {
        let s = self.state.lock().unwrap();
        s.renderable.clone()
    }
}

impl Drop for Live {
    fn drop(&mut self) {
        self.stop();
    }
}

// ---------------------------------------------------------------------------
// Smart references for accessing Console and LiveRender through the Mutex
// ---------------------------------------------------------------------------

/// A guard that provides `&Console` access while the shared state is locked.
pub struct ConsoleRef<'a> {
    guard: std::sync::MutexGuard<'a, SharedState>,
}

impl std::ops::Deref for ConsoleRef<'_> {
    type Target = Console;
    fn deref(&self) -> &Console {
        &self.guard.console
    }
}

/// A guard that provides `&mut Console` access while the shared state is locked.
pub struct ConsoleRefMut<'a> {
    guard: std::sync::MutexGuard<'a, SharedState>,
}

impl std::ops::Deref for ConsoleRefMut<'_> {
    type Target = Console;
    fn deref(&self) -> &Console {
        &self.guard.console
    }
}

impl std::ops::DerefMut for ConsoleRefMut<'_> {
    fn deref_mut(&mut self) -> &mut Console {
        &mut self.guard.console
    }
}

/// A guard that provides `&LiveRender` access while the shared state is locked.
pub struct LiveRenderRef<'a> {
    guard: std::sync::MutexGuard<'a, SharedState>,
}

impl std::ops::Deref for LiveRenderRef<'_> {
    type Target = LiveRender;
    fn deref(&self) -> &LiveRender {
        &self.guard.live_render
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Style;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Helper: build a quiet console so tests don't write to stdout.
    fn test_console() -> Console {
        Console::builder()
            .width(80)
            .height(25)
            .quiet(true)
            .markup(false)
            .no_color(true)
            .force_terminal(true)
            .build()
    }

    // -- Construction -------------------------------------------------------

    #[test]
    fn test_default_construction() {
        let live = Live::new(Text::new("hello", Style::null()));
        assert!(!live.started);
        assert!(live.auto_refresh);
        assert!((live.refresh_per_second - 4.0).abs() < f64::EPSILON);
        assert!(live.refresh_thread.is_none());
        assert!(!live.transient);
        assert_eq!(live.vertical_overflow, VerticalOverflowMethod::Ellipsis);
    }

    #[test]
    fn test_construction_stores_renderable() {
        let live = Live::new(Text::new("Hello", Style::null()));
        assert_eq!(live.renderable().plain(), "Hello");
    }

    // -- Builder methods ----------------------------------------------------

    #[test]
    fn test_with_auto_refresh() {
        let live = Live::new(Text::empty()).with_auto_refresh(false);
        assert!(!live.auto_refresh);
    }

    #[test]
    fn test_with_refresh_per_second() {
        let live = Live::new(Text::empty()).with_refresh_per_second(10.0);
        assert!((live.refresh_per_second - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    #[should_panic(expected = "refresh_per_second must be > 0")]
    fn test_with_refresh_per_second_zero() {
        let _ = Live::new(Text::empty()).with_refresh_per_second(0.0);
    }

    #[test]
    #[should_panic(expected = "refresh_per_second must be > 0")]
    fn test_with_refresh_per_second_negative() {
        let _ = Live::new(Text::empty()).with_refresh_per_second(-1.0);
    }

    #[test]
    fn test_with_transient() {
        let live = Live::new(Text::empty()).with_transient(true);
        assert!(live.transient);
    }

    #[test]
    fn test_with_screen() {
        let live = Live::new(Text::empty()).with_screen(true);
        let s = live.state.lock().unwrap();
        assert!(s.screen);
    }

    #[test]
    fn test_with_vertical_overflow() {
        let live = Live::new(Text::empty()).with_vertical_overflow(VerticalOverflowMethod::Crop);
        assert_eq!(live.vertical_overflow, VerticalOverflowMethod::Crop);
        let s = live.state.lock().unwrap();
        assert_eq!(
            s.live_render.vertical_overflow,
            VerticalOverflowMethod::Crop
        );
    }

    #[test]
    fn test_with_console() {
        let console = test_console();
        let live = Live::new(Text::empty()).with_console(console);
        assert_eq!(live.console().width(), 80);
    }

    #[test]
    fn test_with_get_renderable() {
        let live =
            Live::new(Text::empty()).with_get_renderable(|| Text::new("dynamic", Style::null()));
        let s = live.state.lock().unwrap();
        assert!(s.get_renderable.is_some());
    }

    // -- Lifecycle ----------------------------------------------------------

    #[test]
    fn test_start_stop() {
        let mut live = Live::new(Text::new("test", Style::null()))
            .with_console(test_console())
            .with_auto_refresh(false);

        assert!(!live.is_started());
        live.start();
        assert!(live.is_started());
        live.stop();
        assert!(!live.is_started());
    }

    #[test]
    fn test_double_start_is_noop() {
        let mut live = Live::new(Text::empty())
            .with_console(test_console())
            .with_auto_refresh(false);

        live.start();
        assert!(live.is_started());
        live.start(); // second start should be no-op
        assert!(live.is_started());
        live.stop();
    }

    #[test]
    fn test_double_stop_is_noop() {
        let mut live = Live::new(Text::empty())
            .with_console(test_console())
            .with_auto_refresh(false);

        live.start();
        live.stop();
        assert!(!live.is_started());
        live.stop(); // second stop should be no-op
        assert!(!live.is_started());
    }

    #[test]
    fn test_stop_without_start_is_noop() {
        let mut live = Live::new(Text::empty())
            .with_console(test_console())
            .with_auto_refresh(false);

        live.stop(); // should not panic
        assert!(!live.is_started());
    }

    // -- Update and renderable ----------------------------------------------

    #[test]
    fn test_update_renderable_changes_content() {
        let mut live = Live::new(Text::new("initial", Style::null()))
            .with_console(test_console())
            .with_auto_refresh(false);

        assert_eq!(live.renderable().plain(), "initial");
        live.update_renderable(Text::new("updated", Style::null()), false);
        assert_eq!(live.renderable().plain(), "updated");
    }

    #[test]
    fn test_update_alias() {
        let mut live = Live::new(Text::new("initial", Style::null()))
            .with_console(test_console())
            .with_auto_refresh(false);

        live.update(Text::new("via_update", Style::null()), false);
        assert_eq!(live.renderable().plain(), "via_update");
    }

    #[test]
    fn test_update_with_refresh() {
        let mut live = Live::new(Text::new("initial", Style::null()))
            .with_console(test_console())
            .with_auto_refresh(false);

        live.start();
        live.update_renderable(Text::new("refreshed", Style::null()), true);
        assert_eq!(live.renderable().plain(), "refreshed");
        live.stop();
    }

    #[test]
    fn test_renderable_returns_current() {
        let live = Live::new(Text::new("hello", Style::null()));
        assert_eq!(live.renderable().plain(), "hello");
    }

    #[test]
    fn test_update_also_updates_live_render() {
        let mut live = Live::new(Text::new("old", Style::null()))
            .with_console(test_console())
            .with_auto_refresh(false);

        live.update_renderable(Text::new("new", Style::null()), false);
        assert_eq!(live.live_render().renderable.plain(), "new");
    }

    // -- Refresh thread -----------------------------------------------------

    #[test]
    fn test_auto_refresh_thread_starts_and_stops() {
        let mut live = Live::new(Text::new("auto", Style::null()))
            .with_console(test_console())
            .with_auto_refresh(true)
            .with_refresh_per_second(20.0);

        live.start();
        assert!(live.refresh_thread.is_some());

        // Let the thread run briefly.
        thread::sleep(Duration::from_millis(100));

        live.stop();
        assert!(live.refresh_thread.is_none());
    }

    #[test]
    fn test_no_refresh_thread_when_disabled() {
        let mut live = Live::new(Text::empty())
            .with_console(test_console())
            .with_auto_refresh(false);

        live.start();
        assert!(live.refresh_thread.is_none());
        live.stop();
    }

    #[test]
    fn test_refresh_thread_calls_refresh() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let mut live = Live::new(Text::empty())
            .with_console(test_console())
            .with_auto_refresh(true)
            .with_refresh_per_second(100.0)
            .with_get_renderable(move || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                Text::new("tick", Style::null())
            });

        live.start();
        thread::sleep(Duration::from_millis(150));
        live.stop();

        let count = counter.load(Ordering::SeqCst);
        assert!(
            count >= 2,
            "expected at least 2 refresh calls, got {}",
            count
        );
    }

    // -- Transient mode -----------------------------------------------------

    #[test]
    fn test_transient_mode_flag() {
        let live = Live::new(Text::empty()).with_transient(true);
        assert!(live.transient);
    }

    #[test]
    fn test_transient_stop_does_not_panic() {
        let mut live = Live::new(Text::new("gone", Style::null()))
            .with_console(test_console())
            .with_transient(true)
            .with_auto_refresh(false);

        live.start();
        live.refresh();
        live.stop();
    }

    // -- Screen mode --------------------------------------------------------

    #[test]
    fn test_screen_mode_flag() {
        let live = Live::new(Text::empty()).with_screen(true);
        let s = live.state.lock().unwrap();
        assert!(s.screen);
    }

    #[test]
    fn test_screen_mode_start_stop() {
        let mut live = Live::new(Text::new("screen", Style::null()))
            .with_console(test_console())
            .with_screen(true)
            .with_auto_refresh(false);

        live.start();
        assert!(live.is_started());
        live.stop();
        assert!(!live.is_started());
    }

    // -- Drop trait ----------------------------------------------------------

    #[test]
    fn test_drop_calls_stop() {
        let stop_flag;
        {
            let mut live = Live::new(Text::empty())
                .with_console(test_console())
                .with_auto_refresh(false);
            live.start();
            assert!(live.is_started());
            stop_flag = Arc::clone(&live.stop_flag);
        }

        let stopped = stop_flag.0.lock().unwrap();
        assert!(*stopped, "Drop should have called stop()");
    }

    #[test]
    fn test_drop_with_auto_refresh_cleans_up() {
        let stop_flag;
        {
            let mut live = Live::new(Text::empty())
                .with_console(test_console())
                .with_auto_refresh(true)
                .with_refresh_per_second(20.0);
            live.start();
            stop_flag = Arc::clone(&live.stop_flag);
        }

        let stopped = stop_flag.0.lock().unwrap();
        assert!(*stopped, "Drop should have signalled the stop flag");
    }

    #[test]
    fn test_drop_without_start_does_not_panic() {
        let _live = Live::new(Text::empty())
            .with_console(test_console())
            .with_auto_refresh(true);
    }

    // -- Manual refresh -----------------------------------------------------

    #[test]
    fn test_manual_refresh() {
        let mut live = Live::new(Text::new("manual", Style::null()))
            .with_console(test_console())
            .with_auto_refresh(false);

        live.start();
        live.refresh();
        live.refresh();
        live.stop();
    }

    // -- get_renderable callback --------------------------------------------

    #[test]
    fn test_get_renderable_callback_used_on_refresh() {
        let mut live = Live::new(Text::empty())
            .with_console(test_console())
            .with_auto_refresh(false)
            .with_get_renderable(|| Text::new("from_callback", Style::null()));

        live.start();
        live.refresh();
        live.stop();
    }

    // -- Builder chaining ---------------------------------------------------

    #[test]
    fn test_full_builder_chain() {
        let live = Live::new(Text::new("test", Style::null()))
            .with_console(test_console())
            .with_auto_refresh(true)
            .with_refresh_per_second(10.0)
            .with_transient(false)
            .with_screen(false)
            .with_vertical_overflow(VerticalOverflowMethod::Visible);

        assert!(live.auto_refresh);
        assert!((live.refresh_per_second - 10.0).abs() < f64::EPSILON);
        assert!(!live.transient);
        assert_eq!(live.vertical_overflow, VerticalOverflowMethod::Visible);
    }

    // -- Edge cases ---------------------------------------------------------

    #[test]
    fn test_start_stop_start_again() {
        let mut live = Live::new(Text::empty())
            .with_console(test_console())
            .with_auto_refresh(false);

        live.start();
        live.stop();
        live.start();
        assert!(live.is_started());
        live.stop();
        assert!(!live.is_started());
    }

    #[test]
    fn test_update_before_start() {
        let mut live = Live::new(Text::empty())
            .with_console(test_console())
            .with_auto_refresh(false);

        live.update_renderable(Text::new("before start", Style::null()), false);
        assert_eq!(live.renderable().plain(), "before start");
    }

    #[test]
    fn test_refresh_before_start() {
        let live = Live::new(Text::new("pre-start", Style::null()))
            .with_console(test_console())
            .with_auto_refresh(false);

        live.refresh();
    }

    #[test]
    fn test_auto_refresh_restart() {
        let mut live = Live::new(Text::empty())
            .with_console(test_console())
            .with_auto_refresh(true)
            .with_refresh_per_second(20.0);

        live.start();
        assert!(live.refresh_thread.is_some());
        live.stop();
        assert!(live.refresh_thread.is_none());

        live.start();
        assert!(live.refresh_thread.is_some());
        live.stop();
        assert!(live.refresh_thread.is_none());
    }

    #[test]
    fn test_vertical_overflow_visible() {
        let live = Live::new(Text::empty()).with_vertical_overflow(VerticalOverflowMethod::Visible);
        assert_eq!(live.vertical_overflow, VerticalOverflowMethod::Visible);
    }

    #[test]
    fn test_vertical_overflow_ellipsis_default() {
        let live = Live::new(Text::empty());
        assert_eq!(live.vertical_overflow, VerticalOverflowMethod::Ellipsis);
    }

    #[test]
    fn test_console_accessor() {
        let live = Live::new(Text::new("test", Style::null()))
            .with_console(Console::builder().width(120).build());
        assert_eq!(live.console().width(), 120);
    }

    #[test]
    fn test_console_mut_accessor() {
        let live = Live::new(Text::new("test", Style::null())).with_console(test_console());
        let _console = live.console_mut();
    }
}
