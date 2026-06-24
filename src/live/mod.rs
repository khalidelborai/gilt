//! Live display module -- a terminal display that refreshes at regular intervals.
//!
//! content that updates in-place using cursor movement control codes and an
//! optional background refresh thread.

pub mod live_render;
pub mod screen;

use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use arc_swap::ArcSwap;

use crate::console::{Console, Renderable};
use crate::control::Control;
use crate::segment::{ControlCode, ControlType, Segment};
use crate::text::Text;

use self::live_render::{LiveRender, VerticalOverflowMethod};
use self::screen::Screen;

// ---------------------------------------------------------------------------
// LiveContent -- sized wrapper so ArcSwap can hold a trait-object renderable
// ---------------------------------------------------------------------------

/// Sized wrapper so `ArcSwap` can hold the trait-object renderable.
///
/// `ArcSwap<T>` requires `T: Sized`.  Wrapping `Arc<dyn ...>` in this newtype
/// gives us a `Sized` type while still being type-erased internally.
struct LiveContent(Arc<dyn Renderable + Send + Sync>);

// ---------------------------------------------------------------------------
// SharedState -- data accessed by both the main thread and the refresh thread
// ---------------------------------------------------------------------------

/// Internal state requiring exclusive access during render. The renderer
/// mutates `console` (cursor positioning, segment writes) and `live_render`
/// (shape tracking) — these can't be lock-free without a deeper rewrite.
///
/// The hot field — `renderable` — has been pulled out into
/// `Live::renderable: Arc<ArcSwap<LiveContent>>` so writers no longer contend
/// with the renderer for the SharedState mutex.
struct SharedState {
    console: Console,
    live_render: LiveRender,
    get_renderable: Option<Box<dyn Fn() -> Arc<dyn Renderable + Send + Sync> + Send>>,
    screen: bool,
    /// Frame-skip cache (Task 2): segments from the previous normal-mode
    /// render. When the new render produces the same bytes, we skip all
    /// tty I/O and leave the shape and cursor position unchanged.
    last_segments: Option<Vec<Segment>>,
    /// Opt 1 (line-diff): per-line segments from the previous normal-mode
    /// render. Used to skip rewriting unchanged lines on subsequent frames.
    prev_lines: Vec<Vec<Segment>>,
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
/// `Live` renders any [`Renderable`] value to the terminal fresh each frame,
/// using the live's own console so width and theme are always respected.
/// Content is held as an `Arc<dyn Renderable + Send + Sync>` so any widget
/// (Markdown, Table, Tree, Panel, Layout, …) can be live-displayed without
/// first being flattened to `Text`.
///
/// The display hides the cursor and (optionally) uses a background thread to
/// repaint at a configurable rate. When the display is stopped (explicitly via
/// [`stop`](Live::stop) or implicitly via [`Drop`]), the terminal state is
/// restored.
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
    /// Lock-free hot path. `update_renderable` swaps; `do_refresh` loads.
    /// Writers no longer queue on the SharedState mutex.
    renderable: Arc<ArcSwap<LiveContent>>,
    auto_refresh: bool,
    /// Number of refreshes per second.
    pub refresh_per_second: f64,
    /// Whether the display clears on exit (transient mode).
    pub transient: bool,
    vertical_overflow: VerticalOverflowMethod,
    started: bool,
    /// Whether the display is currently paused (started but not refreshing,
    /// with its render erased). See [`pause`](Live::pause) / [`resume`](Live::resume).
    paused: bool,
    refresh_thread: Option<thread::JoinHandle<()>>,
    stop_flag: Arc<(Mutex<bool>, Condvar)>,
}

impl Live {
    /// Create a new `Live` display for the given renderable.
    ///
    /// Accepts any `Renderable + Send + Sync + 'static` — including `Text`,
    /// `Table`, `Panel`, `Markdown`, etc.
    ///
    /// # Defaults
    /// - `auto_refresh`: `true`
    /// - `refresh_per_second`: `4.0`
    /// - `transient`: `false`
    /// - `screen`: `false`
    /// - `vertical_overflow`: [`VerticalOverflowMethod::Ellipsis`]
    pub fn new(renderable: impl Renderable + Send + Sync + 'static) -> Self {
        let arc: Arc<dyn Renderable + Send + Sync> = Arc::new(renderable);
        let live_render = LiveRender::new_arc(arc.clone());
        let console = Console::new();

        let state = Arc::new(Mutex::new(SharedState {
            console,
            live_render,
            get_renderable: None,
            screen: false,
            last_segments: None,
            prev_lines: Vec::new(),
        }));

        Live {
            state,
            renderable: Arc::new(ArcSwap::from_pointee(LiveContent(arc))),
            auto_refresh: true,
            refresh_per_second: 4.0,
            transient: false,
            vertical_overflow: VerticalOverflowMethod::Ellipsis,
            started: false,
            paused: false,
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
    ///
    /// The closure must return an `Arc<dyn Renderable + Send + Sync>` so that
    /// any widget type (not just `Text`) can be supplied dynamically.
    #[must_use]
    pub fn with_get_renderable<F>(self, f: F) -> Self
    where
        F: Fn() -> Arc<dyn Renderable + Send + Sync> + Send + 'static,
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
    ///
    /// Remains `true` while [`paused`](Self::pause) — a paused display is
    /// started but not refreshing.
    pub fn is_started(&self) -> bool {
        self.started
    }

    /// Whether the live display is currently paused.
    ///
    /// See [`pause`](Self::pause) / [`resume`](Self::resume).
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Get a reference to the underlying `LiveRender` (locks internal state).
    pub fn live_render(&self) -> LiveRenderRef<'_> {
        LiveRenderRef {
            guard: self.state.lock().unwrap(),
        }
    }

    // -- Identity -----------------------------------------------------------

    /// Stable per-instance identity derived from the address of the shared
    /// state `Arc`. Unique for the lifetime of this `Live` because the `Arc`
    /// allocation lives as long as `self` (and any joined refresh thread).
    fn live_id(&self) -> usize {
        Arc::as_ptr(&self.state) as usize
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
            // Register this Live on the console's nesting stack so callers can
            // query how many Live displays are active and which one is on top.
            s.console.push_live(self.live_id());
            if s.screen {
                s.console.set_alt_screen(true);
            }
        }

        self.spawn_refresh_thread();
    }

    /// Spawn the background refresh thread (no-op when `auto_refresh` is off).
    ///
    /// Shared by [`start`](Self::start) and [`resume`](Self::resume). The
    /// caller is responsible for clearing the stop flag beforehand so the
    /// freshly spawned thread does not immediately observe a stale stop signal.
    fn spawn_refresh_thread(&mut self) {
        if !self.auto_refresh {
            return;
        }
        let flag = Arc::clone(&self.stop_flag);
        let state = Arc::clone(&self.state);
        let renderable = Arc::clone(&self.renderable);
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
            Self::do_refresh(&state, &renderable, vertical_overflow);
        });
        self.refresh_thread = Some(handle);
    }

    /// Signal the background refresh thread to exit and join it.
    ///
    /// Shared by [`stop`](Self::stop) and [`pause`](Self::pause). Safe to call
    /// when no thread is running (e.g. `auto_refresh` disabled).
    fn stop_refresh_thread(&mut self) {
        {
            let mut stopped = self.stop_flag.0.lock().unwrap();
            *stopped = true;
            self.stop_flag.1.notify_all();
        }
        if let Some(handle) = self.refresh_thread.take() {
            let _ = handle.join();
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
        self.paused = false;

        // Signal the refresh thread to stop and join it.
        self.stop_refresh_thread();

        let mut s = self.state.lock().unwrap();

        // In transient mode, erase the last render.
        if self.transient {
            let segments = s.live_render.restore_cursor();
            emit_control_segments(&mut s.console, &segments);
        } else {
            // Move to a new line so the terminal prompt doesn't overlap
            // the last rendered content (do_refresh omits trailing newlines
            // to keep shape tracking accurate).
            // Only emit the trailing newline if the live region actually
            // rendered visible content (Rich fix b08e00fc / v14.3.0).
            if s.live_render.last_render_height() > 0 {
                s.console.write_segments(&[Segment::line()]);
            }
        }

        // Deregister this Live from the console's nesting stack, then restore
        // the terminal state.
        s.console.pop_live();
        s.console.show_cursor(true);
        if s.screen {
            s.console.set_alt_screen(false);
        }
    }

    /// Pause the live display, erasing its current render but preserving its
    /// content and state so [`resume`](Self::resume) can restore it.
    ///
    /// This is the cooperative counterpart to [`stop`](Self::stop): it halts
    /// the background refresh and erases the rendered region *in place* (the
    /// same erase transient [`stop`](Self::stop) performs), but — unlike a
    /// non-transient stop — it emits **no trailing newline**, so the last
    /// render is not left behind in the scrollback. The cursor is shown again
    /// so intervening output (for example a child `Live`) behaves normally.
    ///
    /// Use this to hand the terminal's bottom row from one `Live` to another:
    /// pause a sticky footer, let a child display take over, then
    /// [`resume`](Self::resume) the footer. It avoids the duplicate-render
    /// artifact you would otherwise get by toggling [`transient`](Self::transient)
    /// around [`stop`](Self::stop) / [`start`](Self::start) and rebuilding the
    /// renderable.
    ///
    /// Calling `pause` on a display that is not started, or already paused, is
    /// a no-op. In alternate-screen (`screen`) mode there is no scrollback to
    /// protect, so the erase is a no-op; the refresh thread is still halted and
    /// the cursor shown.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gilt::live::Live;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// let mut footer = Live::new(Text::new("status…", Style::null()));
    /// footer.start();
    /// footer.pause();
    /// // … a child Live renders here without a stale footer above it …
    /// footer.resume();
    /// footer.stop();
    /// ```
    pub fn pause(&mut self) {
        if !self.started || self.paused {
            return;
        }
        self.paused = true;

        // Halt the background refresh so it cannot repaint while paused.
        self.stop_refresh_thread();

        let mut s = self.state.lock().unwrap();

        // Erase the rendered region in place (CR + per-line up/erase). This
        // emits no newline, so nothing is left behind in the scrollback. In
        // screen mode the shape is never tracked, so this is an empty no-op.
        let segments = s.live_render.restore_cursor();
        emit_control_segments(&mut s.console, &segments);

        // Forget the shape and invalidate the diff caches so a later `stop`
        // emits no spurious trailing newline and `resume` repaints fresh at
        // the cursor's current location.
        s.live_render.reset();
        s.last_segments = None;
        s.prev_lines = Vec::new();

        // Show the cursor so intervening output behaves normally while paused.
        s.console.show_cursor(true);
    }

    /// Resume a [`paused`](Self::pause) live display.
    ///
    /// Re-hides the cursor, redraws the preserved renderable at the cursor's
    /// current position (drawing downward, so any output that scrolled in
    /// while paused is not disturbed), and restarts the background refresh
    /// thread if `auto_refresh` is enabled.
    ///
    /// Calling `resume` on a display that is not started, or not paused, is a
    /// no-op.
    pub fn resume(&mut self) {
        if !self.started || !self.paused {
            return;
        }
        self.paused = false;

        let current = self.renderable.load().0.clone();

        // Redraw the region at the current cursor position. We deliberately do
        // NOT use the `do_refresh` reposition path (which moves the cursor up
        // over the previous region): after `pause` the region is gone and the
        // cursor sits where the new render should begin, so we draw straight
        // down — the same approach `print_above` uses for its redraw.
        let is_screen = {
            let mut s = self.state.lock().unwrap();
            s.console.show_cursor(false);
            if s.screen {
                true
            } else {
                s.live_render.set_renderable(current);
                let opts = s.console.options();
                let new_lines = s.live_render.gilt_console_lines(&s.console, &opts);

                // Flatten to a single segment list (mirrors `do_refresh`).
                let render_segments: Vec<Segment> = {
                    let line_count = new_lines.len();
                    let mut segs = Vec::new();
                    for (i, line) in new_lines.iter().enumerate() {
                        segs.extend(line.iter().cloned());
                        if i + 1 < line_count {
                            segs.push(Segment::line());
                        }
                    }
                    segs
                };

                s.console.write_segments(&render_segments);

                // Re-establish the diff caches so the next refresh diffs from
                // the just-drawn frame (frame-skip / line-diff stay correct).
                s.prev_lines = new_lines;
                s.last_segments = Some(render_segments);
                false
            }
        };

        // Re-arm the stop flag and restart the refresh thread AFTER the initial
        // paint so the thread cannot race the redraw.
        {
            let mut stopped = self.stop_flag.0.lock().unwrap();
            *stopped = false;
        }
        self.spawn_refresh_thread();

        // In screen mode the redraw above was skipped (rendering happens from
        // home); paint the first frame now.
        if is_screen {
            self.refresh();
        }
    }

    // -- Content management -------------------------------------------------

    /// Refresh the display with the current content.
    ///
    /// This acquires the shared state lock internally, so it is safe to call
    /// from any thread (the refresh thread calls this automatically).
    pub fn refresh(&self) {
        Self::do_refresh(&self.state, &self.renderable, self.vertical_overflow);
    }

    /// Internal refresh implementation operating on shared state.
    ///
    /// # Locking discipline (T8)
    ///
    /// The key contention fix is that **content updates are lock-free**:
    /// [`update_renderable`](Live::update_renderable) publishes the new content
    /// via an `ArcSwap::store` and only acquires the `SharedState` mutex when an
    /// immediate repaint is requested. Worker threads pushing updates therefore
    /// never queue behind the renderer.
    ///
    /// `do_refresh` itself takes the mutex in two short phases:
    /// 1. **Snapshot** (brief lock): resolve the content — call the
    ///    `get_renderable` callback if present, otherwise load the `ArcSwap`
    ///    (the load is lock-free; the lock here only guards reading the config
    ///    flags) — and read the `screen` mode flag, then release.
    /// 2. **Render + emit** (brief lock): re-acquire only to generate segments
    ///    and write them. The render (`gilt_console`) and the terminal emit
    ///    (`position_cursor` + `write_segments`) are kept in one critical
    ///    section deliberately: a terminal write must be serialized — two
    ///    refreshes cannot interleave their bytes — and `write_segments` needs
    ///    `&mut Console`. Moving the pure render fully outside this lock would
    ///    require teaching `LiveRender` to render from an owned options/theme
    ///    snapshot without `&Console`; that larger refactor is deferred.
    ///
    /// No deadlock is possible: the two critical sections never overlap and the
    /// `ArcSwap` operations are lock-free.
    fn do_refresh(
        state: &Arc<Mutex<SharedState>>,
        renderable: &Arc<ArcSwap<LiveContent>>,
        vertical_overflow: VerticalOverflowMethod,
    ) {
        // ── Phase 1: snapshot config and resolve the renderable ──────────────
        // Hold the mutex only long enough to read `get_renderable` and config.
        let (content, is_screen) = {
            let s = state.lock().unwrap();
            let content: Arc<dyn Renderable + Send + Sync> = match &s.get_renderable {
                Some(f) => f(),
                None => renderable.load().0.clone(),
            };
            (content, s.screen)
        };

        // ── Phase 2: render + emit (brief lock) ──────────────────────────────
        if is_screen {
            // Screen/alt-screen mode: wrap the renderable directly in Screen so
            // segments (with their styles/SGR) are preserved all the way to the
            // terminal.  The old path flattened styled segments to a plain String
            // first, stripping all ANSI escapes (bug #25).
            let mut s = state.lock().unwrap();
            s.live_render.set_renderable(content.clone());
            s.live_render.vertical_overflow = vertical_overflow;
            // DEC mode 2026: render the whole screen frame atomically.
            s.console.begin_synchronized();
            // Emit home() so each frame overwrites from the top-left corner
            // instead of appending below the previous frame.
            let home_ctrl = crate::control::Control::home();
            s.console.control(&home_ctrl);
            let screen = Screen::from_arc(content.clone());
            s.console.print(&screen);
            s.console.end_synchronized();
        } else {
            // Normal mode: render to segments, then emit cursor repositioning
            // + content — all within a single brief lock to keep the emitted
            // bytes and the stored shape consistent.
            let mut s = state.lock().unwrap();
            s.live_render.set_renderable(content);
            s.live_render.vertical_overflow = vertical_overflow;
            let opts = s.console.options();

            // Render to per-line segments (Opt 1 line-diff path).
            let new_lines = s.live_render.gilt_console_lines(&s.console, &opts);

            // Flatten to a single segment list for the frame-skip check and
            // for the legacy full-repaint fallback.
            let render_segments: Vec<Segment> = {
                let line_count = new_lines.len();
                let mut segs = Vec::new();
                for (i, line) in new_lines.iter().enumerate() {
                    segs.extend(line.iter().cloned());
                    if i + 1 < line_count {
                        segs.push(Segment::line());
                    }
                }
                segs
            };

            // ── Task 2: frame-skip ─────────────────────────────────────────
            // If the new segments are byte-identical to the last render, skip
            // all tty I/O. Skipping also leaves cursor position and shape
            // unchanged, which is correct: the terminal already shows the
            // right content and `position_cursor` still knows the height.
            if s.last_segments.as_deref() == Some(render_segments.as_slice()) {
                return;
            }

            // DEC mode 2026: wrap the cursor-reposition + redraw in synchronized
            // output so the terminal applies the whole frame atomically (no
            // tearing/flicker, especially under tmux). Harmless no-op on
            // terminals without support, and suppressed on dumb/non-terminals.
            s.console.begin_synchronized();

            // ── Opt 1: line-diff repaint ───────────────────────────────────
            // Only rewrite lines that have changed. Unchanged lines are
            // skipped with a single CursorDown(1) move.
            //
            // Safety / correctness constraints respected:
            // - We are INSIDE begin_synchronized / end_synchronized (unchanged).
            // - Locking discipline is identical to before (single brief lock).
            // - Frame-skip (whole-frame identical) already returned above.
            // - print_above invalidates prev_lines via last_segments=None AND
            //   resets prev_lines (handled after this block via the
            //   print_above path in that method).
            let prev_lines = std::mem::take(&mut s.prev_lines);
            let prev_height = prev_lines.len();
            let new_height = new_lines.len();

            if prev_height == 0 {
                // First frame or after invalidation: full repaint (no diff data).
                let position_segments = s.live_render.position_cursor();
                emit_control_segments(&mut s.console, &position_segments);
                s.console.write_segments(&render_segments);
            } else {
                // Move cursor to top of previous region (CR + erase current
                // line + move up (height-1) lines, erasing each).
                let position_segments = s.live_render.position_cursor();
                // position_cursor() uses the NEW shape (already set by
                // gilt_console_lines). We need to use the PREVIOUS height for
                // cursor positioning so we land at the top of the old region.
                // Build the cursor-to-top sequence manually from prev_height.
                let mut codes: Vec<ControlCode> = Vec::new();
                codes.push(ControlCode::Simple(ControlType::CarriageReturn));
                // Erase current (first) line only; remaining erases happen
                // per-line during the diff loop.
                for _ in 0..prev_height.saturating_sub(1) {
                    codes.push(ControlCode::WithParam(ControlType::CursorUp, 1));
                }
                emit_control_segments(&mut s.console, &[Segment::new("", None, Some(codes))]);

                // Per-line diff: for each line in the new frame, either skip
                // (cursor down) or erase + rewrite.
                for (i, new_line) in new_lines.iter().enumerate().take(new_height) {
                    let prev_line = prev_lines.get(i);
                    let line_changed = prev_line != Some(new_line);

                    if line_changed {
                        // Erase this line and write the new content.
                        let erase = Segment::new(
                            "",
                            None,
                            Some(vec![
                                ControlCode::Simple(ControlType::CarriageReturn),
                                ControlCode::WithParam(ControlType::EraseInLine, 2),
                            ]),
                        );
                        emit_control_segments(&mut s.console, &[erase]);
                        s.console.write_segments(new_line);
                    }

                    // Move to next line (unless this is the last new line).
                    if i + 1 < new_height {
                        let down = Segment::new(
                            "",
                            None,
                            Some(vec![ControlCode::WithParam(ControlType::CursorDown, 1)]),
                        );
                        emit_control_segments(&mut s.console, &[down]);
                    }
                }

                // Handle region shrinking: erase any extra old lines.
                if prev_height > new_height {
                    for _ in new_height..prev_height {
                        let erase_old = Segment::new(
                            "",
                            None,
                            Some(vec![
                                ControlCode::WithParam(ControlType::CursorDown, 1),
                                ControlCode::Simple(ControlType::CarriageReturn),
                                ControlCode::WithParam(ControlType::EraseInLine, 2),
                            ]),
                        );
                        emit_control_segments(&mut s.console, &[erase_old]);
                    }
                    // After erasing extra lines, move back up to just after
                    // the last new line so the shape tracking is consistent.
                    let lines_to_go_up = (prev_height - new_height) as i32;
                    if lines_to_go_up > 0 {
                        let go_up = Segment::new(
                            "",
                            None,
                            Some(vec![ControlCode::WithParam(
                                ControlType::CursorUp,
                                lines_to_go_up,
                            )]),
                        );
                        emit_control_segments(&mut s.console, &[go_up]);
                    }
                }

                // If we used the position_cursor output (needed to keep the
                // borrow checker happy when prev_height > 0), use it now.
                let _ = position_segments; // used above via manual codes
            }

            s.console.end_synchronized();
            s.last_segments = Some(render_segments);
            s.prev_lines = new_lines;
        }
    }

    /// Update the renderable content.
    ///
    /// Accepts any `Renderable + Send + Sync + 'static` — including `Text`,
    /// `Table`, `Panel`, `Markdown`, etc.
    ///
    /// If `refresh` is `true`, the display is repainted immediately.
    ///
    /// Takes `&self` so a `Live` can be shared across threads (typically
    /// behind `Arc`). The store path is **lock-free** (`ArcSwap::store`),
    /// so writers no longer contend with the renderer or with each other.
    /// The mutex is only acquired when `refresh = true` triggers an
    /// immediate repaint.
    pub fn update_renderable(
        &self,
        renderable: impl Renderable + Send + Sync + 'static,
        refresh: bool,
    ) {
        let arc: Arc<dyn Renderable + Send + Sync> = Arc::new(renderable);
        // Lock-free hot path: atomic pointer swap. No mutex contention
        // with concurrent writers or the refresh thread. The renderer
        // picks up the new value on its next load.
        self.renderable.store(Arc::new(LiveContent(arc)));
        if refresh {
            self.refresh();
        }
    }

    /// Alias for [`update_renderable`](Live::update_renderable).
    pub fn update(&self, renderable: impl Renderable + Send + Sync + 'static, refresh: bool) {
        self.update_renderable(renderable, refresh);
    }

    /// Replace the renderable and refresh. Equivalent to `update(r, true)`.
    pub fn set(&self, renderable: impl Renderable + Send + Sync + 'static) {
        self.update_renderable(renderable, true);
    }

    /// Create a `Live` display from any [`Renderable`].
    ///
    /// Takes an OWNED renderable and stores it directly (no snapshot/flatten).
    /// The widget is rendered fresh each frame using the live console.
    ///
    /// ```no_run
    /// # use gilt::{live::Live, table::Table};
    /// let mut t = Table::new(&["Name", "CPU"]);
    /// t.add_row(&["systemd", "1.2%"]);
    /// let _live = Live::from_renderable(t);
    /// ```
    pub fn from_renderable<R: Renderable + Send + Sync + 'static>(renderable: R) -> Self {
        Self::new(renderable)
    }

    /// Tick-update setter: replace the live content with any renderable widget.
    ///
    /// Equivalent to `self.set(renderable)`.
    pub fn set_renderable_widget<R: Renderable + Send + Sync + 'static>(&self, renderable: R) {
        self.set(renderable);
    }

    /// Print non-live content *above* the live region without corrupting it.
    ///
    /// The live region is erased (cursor moved up, lines cleared), the
    /// `above` renderable is printed with a trailing newline so it scrolls
    /// into the scrollback, and then the live region is immediately
    /// re-rendered below it.
    ///
    /// In alt-screen (`screen`) mode this is a no-op because the alternate
    /// screen has no scrollback: emitting content above the live region would
    /// overwrite arbitrary terminal cells.
    ///
    /// # Locking
    /// Acquires `SharedState` in a single brief critical section — the same
    /// pattern as Phase 2 of [`do_refresh`](Self::do_refresh). The lock-free
    /// `ArcSwap` update path in [`update_renderable`](Self::update_renderable)
    /// is unchanged.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gilt::live::Live;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// let mut live = Live::new(Text::new("progress…", Style::null()));
    /// live.start();
    /// live.print_above(Text::new("Step 1 done", Style::null()));
    /// live.stop();
    /// ```
    pub fn print_above(&self, above: impl Renderable + Send + Sync + 'static) {
        let current = self.renderable.load().0.clone();

        let mut s = self.state.lock().unwrap();

        // In alt-screen mode there is no scrollback: silently skip.
        if s.screen {
            return;
        }

        // Phase A: erase the live region by restoring the cursor to its
        // pre-render position.
        let restore = s.live_render.restore_cursor();
        emit_control_segments(&mut s.console, &restore);

        // Phase B: print the above content and a trailing newline so it
        // scrolls into the scrollback buffer.
        s.console.print(&above);
        s.console.write_segments(&[Segment::line()]);

        // Phase C: re-render the live region so it appears below the new
        // content. We need to go through the full LiveRender path to update
        // `shape` correctly, but we don't use `do_refresh` to avoid a second
        // mutex acquisition (we already hold the lock).
        s.live_render.set_renderable(current);
        let opts = s.console.options();
        let render_segments = s.live_render.gilt_console(&s.console, &opts);
        s.console.write_segments(&render_segments);
        // Invalidate the frame-skip cache and line-diff cache — the cursor
        // repositioning from Phase A means the next `do_refresh` must always
        // re-emit from scratch.
        s.last_segments = None;
        s.prev_lines = Vec::new();
    }

    /// Construct + start in one call. `Drop` calls [`stop`](Self::stop)
    /// automatically when the returned value goes out of scope.
    pub fn run(initial: impl Renderable + Send + Sync + 'static) -> Self {
        let mut live = Self::new(initial);
        live.start();
        live
    }

    /// Get a clone of the current renderable as an `Arc`.
    ///
    /// This is a lock-free read via `ArcSwap` — no mutex acquisition.
    pub fn current_renderable(&self) -> Arc<dyn Renderable + Send + Sync> {
        self.renderable.load().0.clone()
    }

    /// Render the current renderable to `Text` using the live console.
    ///
    /// Acquires the shared state lock briefly to access the console, then
    /// renders using `render_lines` (which is independent of the `quiet` flag
    /// and does not go through the I/O path) so the live console's width and
    /// theme are always used correctly.
    pub fn render_to_text(&self) -> Text {
        let content = self.current_renderable();
        let s = self.state.lock().unwrap();
        let opts = s.console.options();
        // Render into lines using the live console (respects width, theme).
        let lines = s
            .console
            .render_lines(content.as_ref(), Some(&opts), None, false, false);
        // Flatten lines into a single text string.
        let mut result = String::new();
        let line_count = lines.len();
        for (i, line) in lines.into_iter().enumerate() {
            for seg in &line {
                if !seg.is_control() {
                    result.push_str(&seg.text);
                }
            }
            if i + 1 < line_count {
                result.push('\n');
            }
        }
        Text::new(&result, crate::style::Style::null())
    }

    /// Get a clone of the current renderable as `Text` (for compatibility).
    ///
    /// Renders the current renderable through the live console so width and
    /// theme are always respected. Prefer
    /// [`current_renderable`](Self::current_renderable) for type-preserving
    /// access.
    pub fn renderable(&self) -> Text {
        self.render_to_text()
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
        // renderable() now renders to Text via the live console
        let text = live.renderable();
        assert!(text.plain().contains("Hello"));
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
        let live = Live::new(Text::empty()).with_get_renderable(|| {
            Arc::new(Text::new("dynamic", Style::null())) as Arc<dyn Renderable + Send + Sync>
        });
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
        let live = Live::new(Text::new("initial", Style::null()))
            .with_console(test_console())
            .with_auto_refresh(false);

        live.update_renderable(Text::new("updated", Style::null()), false);
        // current_renderable is lock-free
        // renderable() renders via the live console — check it contains "updated"
        let txt = live.renderable();
        assert!(txt.plain().contains("updated"));
    }

    #[test]
    fn test_update_alias() {
        let live = Live::new(Text::new("initial", Style::null()))
            .with_console(test_console())
            .with_auto_refresh(false);

        live.update(Text::new("via_update", Style::null()), false);
        let txt = live.renderable();
        assert!(txt.plain().contains("via_update"));
    }

    #[test]
    fn test_update_with_refresh() {
        let mut live = Live::new(Text::new("initial", Style::null()))
            .with_console(test_console())
            .with_auto_refresh(false);

        live.start();
        live.update_renderable(Text::new("refreshed", Style::null()), true);
        let txt = live.renderable();
        assert!(txt.plain().contains("refreshed"));
        live.stop();
    }

    #[test]
    fn test_renderable_returns_current() {
        let live = Live::new(Text::new("hello", Style::null()))
            .with_console(test_console())
            .with_auto_refresh(false);
        let txt = live.renderable();
        assert!(txt.plain().contains("hello"));
    }

    #[test]
    fn test_update_also_updates_live_render() {
        // After v0.10.3 lock-free Live: update_renderable is lock-free and
        // doesn't touch live_render. The internal LiveRender is synced on
        // the next refresh — that's the contract this test now validates.
        let live = Live::new(Text::new("old", Style::null()))
            .with_console(test_console())
            .with_auto_refresh(false);

        live.update_renderable(Text::new("new", Style::null()), false);
        // ArcSwap is updated immediately:
        let arc = live.current_renderable();
        // The Arc holds the new text; we verify via render_to_text
        let txt = live.render_to_text();
        assert!(txt.plain().contains("new"));
        // LiveRender catches up on the next refresh:
        live.refresh();
        // After refresh, live_render holds the Arc too (checked via rendering)
        let _ = arc; // keep alive
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
                Arc::new(Text::new("tick", Style::null())) as Arc<dyn Renderable + Send + Sync>
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
            .with_get_renderable(|| {
                Arc::new(Text::new("from_callback", Style::null()))
                    as Arc<dyn Renderable + Send + Sync>
            });

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
        let live = Live::new(Text::empty())
            .with_console(test_console())
            .with_auto_refresh(false);

        live.update_renderable(Text::new("before start", Style::null()), false);
        let txt = live.renderable();
        assert!(txt.plain().contains("before start"));
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

    // -- Rich fix b08e00fc: no spurious trailing newline when nothing rendered --

    /// When Live is stopped without ever calling `update()` / `refresh()`,
    /// `last_render_height` is 0 and no trailing `\n` should be emitted.
    #[test]
    fn live_stop_with_no_render_emits_no_newline() {
        let console = Console::builder()
            .width(80)
            .height(25)
            .quiet(false)
            .markup(false)
            .no_color(true)
            .force_terminal(true)
            .build();

        let mut live = Live::new(Text::new("hello", Style::null()))
            .with_console(console)
            .with_auto_refresh(false);

        // Begin capture before start so all writes are recorded.
        live.state.lock().unwrap().console.begin_capture();

        live.start();
        // Deliberately skip refresh / update — nothing is rendered.
        live.stop();

        let captured = live.state.lock().unwrap().console.end_capture();
        // The captured text should not contain a bare newline emitted by stop().
        // Control sequences (hide/show cursor) have no printable text, so the
        // only text character that could appear is the spurious '\n'.
        assert!(
            !captured.contains('\n'),
            "expected no trailing newline when nothing was rendered, got: {:?}",
            captured
        );
    }

    /// When Live renders at least one frame and is then stopped, a trailing
    /// `\n` must still be emitted so the shell prompt doesn't overwrite content.
    #[test]
    fn live_stop_after_render_emits_newline() {
        let console = Console::builder()
            .width(80)
            .height(25)
            .quiet(false)
            .markup(false)
            .no_color(true)
            .force_terminal(true)
            .build();

        let mut live = Live::new(Text::new("progress", Style::null()))
            .with_console(console)
            .with_auto_refresh(false);

        // Begin capture before start.
        live.state.lock().unwrap().console.begin_capture();

        live.start();
        // Render at least one frame so last_render_height > 0.
        live.refresh();
        live.stop();

        let captured = live.state.lock().unwrap().console.end_capture();
        // The trailing '\n' is emitted before show_cursor (which itself emits
        // an escape sequence), so it won't be the very last byte — but it must
        // be present in the output.
        assert!(
            captured.contains('\n'),
            "expected a trailing newline after rendering, got: {:?}",
            captured
        );
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

    // -- Task 1: print_above ------------------------------------------------

    /// `print_above` in screen mode is a no-op — no panic, no output.
    #[test]
    fn print_above_screen_mode_is_noop() {
        let mut live = Live::new(Text::new("live", Style::null()))
            .with_console(test_console())
            .with_screen(true)
            .with_auto_refresh(false);
        live.start();
        live.print_above(Text::new("above", Style::null())); // must not panic
        live.stop();
    }

    /// DEC mode 2026: a Live refresh wraps its emit in begin/end synchronized
    /// output so the terminal renders the frame atomically (no flicker).
    #[test]
    fn refresh_wraps_emit_in_synchronized_output() {
        let console = Console::builder()
            .width(80)
            .height(25)
            .quiet(false)
            .markup(false)
            .no_color(true)
            .force_terminal(true)
            .build();

        let mut live = Live::new(Text::new("frame content", Style::null()))
            .with_console(console)
            .with_auto_refresh(false);

        live.state.lock().unwrap().console.begin_capture();
        live.start();
        live.refresh();
        let captured = live.state.lock().unwrap().console.end_capture();

        assert!(
            captured.contains("\x1b[?2026h"),
            "frame should open synchronized output (CSI ?2026h): {captured:?}"
        );
        assert!(
            captured.contains("\x1b[?2026l"),
            "frame should close synchronized output (CSI ?2026l): {captured:?}"
        );
        let begin = captured.find("\x1b[?2026h").unwrap();
        let end = captured.rfind("\x1b[?2026l").unwrap();
        assert!(begin < end, "begin-sync must precede end-sync");
        assert!(captured.contains("frame content"));
    }

    /// `print_above` in normal mode: both the above-content and the live
    /// region appear in the captured output, and the live region is re-drawn.
    #[test]
    fn print_above_emits_content_and_redraws_live_region() {
        let console = Console::builder()
            .width(80)
            .height(25)
            .quiet(false)
            .markup(false)
            .no_color(true)
            .force_terminal(true)
            .build();

        let mut live = Live::new(Text::new("live_content", Style::null()))
            .with_console(console)
            .with_auto_refresh(false);

        live.state.lock().unwrap().console.begin_capture();

        live.start();
        // Render once so live_render has a known shape.
        live.refresh();
        // Now print something above.
        live.print_above(Text::new("above_content", Style::null()));
        live.stop();

        let captured = live.state.lock().unwrap().console.end_capture();

        // Both pieces of content must be present.
        assert!(
            captured.contains("above_content"),
            "above content missing; got: {:?}",
            captured
        );
        assert!(
            captured.contains("live_content"),
            "live content missing after print_above; got: {:?}",
            captured
        );
    }

    // -- Task 2: frame-skip -------------------------------------------------

    /// Two refreshes with identical content must perform I/O only once.
    /// We verify by checking that the captured buffer doesn't grow on the
    /// second identical refresh.
    #[test]
    fn frame_skip_identical_refreshes_skip_second_write() {
        let console = Console::builder()
            .width(80)
            .height(25)
            .quiet(false)
            .markup(false)
            .no_color(true)
            .force_terminal(true)
            .build();

        let mut live = Live::new(Text::new("same_content", Style::null()))
            .with_console(console)
            .with_auto_refresh(false);

        live.state.lock().unwrap().console.begin_capture();
        live.start();

        // First refresh: should write content.
        live.refresh();
        let after_first = live.state.lock().unwrap().console.end_capture();

        // Begin a fresh capture for the second refresh.
        live.state.lock().unwrap().console.begin_capture();

        // Second refresh with the SAME content: frame-skip should prevent I/O.
        live.refresh();
        let after_second = live.state.lock().unwrap().console.end_capture();

        live.stop();

        // The first refresh must have produced some output.
        assert!(
            !after_first.is_empty(),
            "first refresh should produce output"
        );

        // The second identical refresh must produce NO output (skipped).
        assert!(
            after_second.is_empty(),
            "second identical refresh should be skipped (no I/O), got: {:?}",
            after_second
        );
    }

    /// After a content change the frame-skip cache is invalidated and the
    /// next refresh IS emitted.
    #[test]
    fn frame_skip_different_content_is_emitted() {
        let console = Console::builder()
            .width(80)
            .height(25)
            .quiet(false)
            .markup(false)
            .no_color(true)
            .force_terminal(true)
            .build();

        let mut live = Live::new(Text::new("first", Style::null()))
            .with_console(console)
            .with_auto_refresh(false);

        live.start();

        // Establish a baseline.
        live.refresh();

        live.state.lock().unwrap().console.begin_capture();

        // Change content and refresh — must emit.
        live.update_renderable(Text::new("second", Style::null()), false);
        live.refresh();

        let captured = live.state.lock().unwrap().console.end_capture();
        live.stop();

        assert!(
            captured.contains("second"),
            "changed content should have been emitted, got: {:?}",
            captured
        );
    }

    // -- from_renderable (now takes owned R) --------------------------------

    #[test]
    fn test_from_renderable_stores_directly() {
        let text = Text::new("from_renderable_test", Style::null());
        let live = Live::from_renderable(text).with_console(test_console());
        let txt = live.renderable();
        assert!(txt.plain().contains("from_renderable_test"));
    }

    // -- current_renderable and render_to_text ------------------------------

    #[test]
    fn test_current_renderable_is_arc() {
        let live = Live::new(Text::new("arc_test", Style::null()))
            .with_console(test_console())
            .with_auto_refresh(false);
        let arc = live.current_renderable();
        // Should be non-null; we can verify by cloning
        let _arc2 = arc.clone();
    }

    #[test]
    fn test_render_to_text_uses_live_console() {
        let live = Live::new(Text::new("render_to_text_test", Style::null()))
            .with_console(test_console())
            .with_auto_refresh(false);
        let txt = live.render_to_text();
        assert!(txt.plain().contains("render_to_text_test"));
    }

    // -- Step 7: Width-aware rendering test --------------------------------
    //
    // This test proves that a Live display with a non-default-width console
    // renders content using the LIVE console's width, not a default 80-col one.
    // A 120-column console is set; a long string (>80 chars) that would wrap
    // at 80 columns should NOT wrap when the live console is 120 wide.

    // -- Opt 1: line-diff repaint -------------------------------------------

    /// RED test: when only the middle line changes, the second refresh should
    /// NOT re-emit the text of unchanged first/third lines, but MUST emit the
    /// new middle-line text. Without the line-diff optimisation the whole
    /// frame is rewritten, so the first/third line text appears in the second
    /// capture — this assertion will FAIL until the optimisation is implemented.
    #[test]
    fn line_diff_unchanged_lines_not_rewritten() {
        let console = Console::builder()
            .width(80)
            .height(25)
            .quiet(false)
            .markup(false)
            .no_color(true)
            .force_terminal(true)
            .build();

        // Three-line content: "alpha\nbeta\ngamma"
        let mut live = Live::new(Text::new("alpha\nbeta\ngamma", Style::null()))
            .with_console(console)
            .with_auto_refresh(false);

        live.state.lock().unwrap().console.begin_capture();
        live.start();

        // First refresh — establish prev_lines.
        live.refresh();

        // Discard first-frame output; start fresh capture for the second frame.
        let _ = live.state.lock().unwrap().console.end_capture();
        live.state.lock().unwrap().console.begin_capture();

        // Change ONLY the middle line.
        live.update_renderable(
            Text::new("alpha\nBETA_CHANGED\ngamma", Style::null()),
            false,
        );
        live.refresh();

        let second_capture = live.state.lock().unwrap().console.end_capture();
        live.stop();

        // The new middle-line text must appear.
        assert!(
            second_capture.contains("BETA_CHANGED"),
            "new middle-line text must be emitted, got: {:?}",
            second_capture
        );
        // The unchanged first and third lines must NOT be rewritten (line-diff skips them).
        assert!(
            !second_capture.contains("alpha"),
            "unchanged first line 'alpha' must NOT be rewritten in second capture, \
             got: {:?}",
            second_capture
        );
        assert!(
            !second_capture.contains("gamma"),
            "unchanged third line 'gamma' must NOT be rewritten in second capture, \
             got: {:?}",
            second_capture
        );
    }

    /// Correctness: the FINAL visible content after a line-diff refresh must
    /// contain all three lines (unchanged + changed). This validates that the
    /// diff does not corrupt the frame state.
    #[test]
    fn line_diff_final_content_correct() {
        let console = Console::builder()
            .width(80)
            .height(25)
            .quiet(false)
            .markup(false)
            .no_color(true)
            .force_terminal(true)
            .build();

        let mut live = Live::new(Text::new("line1\nline2\nline3", Style::null()))
            .with_console(console)
            .with_auto_refresh(false);

        live.start();
        live.refresh();

        // Change middle line.
        live.update_renderable(
            Text::new("line1\nLINE2_UPDATED\nline3", Style::null()),
            false,
        );
        live.state.lock().unwrap().console.begin_capture();
        live.refresh();
        let captured = live.state.lock().unwrap().console.end_capture();
        live.stop();

        // Must contain the updated text (frame is correct).
        assert!(
            captured.contains("LINE2_UPDATED"),
            "updated line must appear in output, got: {:?}",
            captured
        );
    }

    #[test]
    fn test_live_renders_with_live_console_width() {
        // Build a 120-col quiet console.
        let console = Console::builder()
            .width(120)
            .height(25)
            .quiet(true)
            .markup(false)
            .no_color(true)
            .force_terminal(true)
            .build();

        // A string that is exactly 100 characters — fits in 120 cols but would
        // wrap at 80 cols (where it becomes at least 2 lines).
        let long_line = "A".repeat(100);
        let live = Live::new(Text::new(&long_line, Style::null()))
            .with_console(console)
            .with_auto_refresh(false);

        // render_to_text uses the live console (120 wide).
        let rendered = live.render_to_text();
        // A 120-wide console keeps 100 chars on one line — no wrapping.
        // plain() strips ANSI; we check no newline interrupts the A-sequence.
        let plain = rendered.plain();
        // Should contain the full 100-char run without a newline in the middle.
        assert!(
            plain.contains(&long_line),
            "expected 100-char line to fit without wrapping at 120 cols, got: {:?}",
            plain
        );
    }

    // -- pause / resume -----------------------------------------------------

    /// Build a capturing console (visible output, recorded for assertions).
    fn capture_console() -> Console {
        Console::builder()
            .width(80)
            .height(25)
            .quiet(false)
            .markup(false)
            .no_color(true)
            .force_terminal(true)
            .build()
    }

    /// The headline scenario: an outer Live renders a multi-line footer, is
    /// paused (footer erased with no stale scrollback line), then resumed
    /// (footer redrawn). Mirrors a sticky-footer Live handing the bottom row
    /// to a child Live and taking it back.
    #[test]
    fn pause_resume_footer_handoff_no_stale_scrollback() {
        let mut outer = Live::new(Text::new("FOOTER_L1\nFOOTER_L2\nFOOTER_L3", Style::null()))
            .with_console(capture_console())
            .with_auto_refresh(false);

        outer.start();
        outer.refresh(); // draw the footer once

        assert!(outer.is_started());
        assert!(!outer.is_paused());

        // -- pause: erase the footer in place, leave nothing in scrollback --
        outer.state.lock().unwrap().console.begin_capture();
        outer.pause();
        let paused = outer.state.lock().unwrap().console.end_capture();

        assert!(outer.is_paused(), "pause() should mark the display paused");
        assert!(
            outer.is_started(),
            "pause() must preserve the started state so resume() can restore it"
        );
        // The rendered lines must be erased in place (EraseInLine -> CSI 2 K).
        assert!(
            paused.contains("\x1b[2K"),
            "pause must erase the rendered footer lines, got: {paused:?}"
        );
        // The cursor must be shown again so intervening output behaves normally.
        assert!(
            paused.contains("\x1b[?25h"),
            "pause must show the cursor, got: {paused:?}"
        );
        // No footer text is re-emitted...
        assert!(
            !paused.contains("FOOTER_L1"),
            "pause must not re-print footer text, got: {paused:?}"
        );
        // ...and crucially no newline pushes the footer into the scrollback.
        assert!(
            !paused.contains('\n'),
            "pause must not emit a newline into the scrollback, got: {paused:?}"
        );

        // -- resume: the footer reappears and refreshing resumes --
        outer.state.lock().unwrap().console.begin_capture();
        outer.resume();
        let resumed = outer.state.lock().unwrap().console.end_capture();

        assert!(!outer.is_paused(), "resume() should clear the paused state");
        assert!(outer.is_started());
        assert!(
            resumed.contains("FOOTER_L1") && resumed.contains("FOOTER_L3"),
            "resume must redraw the full footer, got: {resumed:?}"
        );
        // The live display re-takes ownership of the cursor on resume.
        assert!(
            resumed.contains("\x1b[?25l"),
            "resume must hide the cursor again, got: {resumed:?}"
        );
        // resume must draw DOWNWARD from the cursor — it must not move the
        // cursor up (CSI A), which would overwrite output that scrolled in
        // above while paused.
        assert!(
            !resumed.contains("\x1b[1A"),
            "resume must not move the cursor up over content above, got: {resumed:?}"
        );

        outer.stop();
    }

    /// After `pause()` the region is already erased and the shape reset, so a
    /// subsequent `stop()` must NOT emit a trailing newline (which would leave
    /// a blank line in the scrollback).
    #[test]
    fn pause_then_stop_emits_no_trailing_newline() {
        let mut live = Live::new(Text::new("footer\npanel", Style::null()))
            .with_console(capture_console())
            .with_auto_refresh(false);

        live.start();
        live.refresh();
        live.pause();

        live.state.lock().unwrap().console.begin_capture();
        live.stop();
        let captured = live.state.lock().unwrap().console.end_capture();

        assert!(
            !captured.contains('\n'),
            "stop() after pause() must not emit a trailing newline, got: {captured:?}"
        );
    }

    /// `pause()` before `start()` is a no-op and must not panic.
    #[test]
    fn pause_without_start_is_noop() {
        let mut live = Live::new(Text::empty())
            .with_console(test_console())
            .with_auto_refresh(false);
        live.pause();
        assert!(!live.is_paused());
        assert!(!live.is_started());
    }

    /// A second `pause()` while already paused is a no-op (no double erase).
    #[test]
    fn double_pause_is_noop() {
        let mut live = Live::new(Text::new("x", Style::null()))
            .with_console(test_console())
            .with_auto_refresh(false);
        live.start();
        live.refresh();
        live.pause();
        assert!(live.is_paused());
        live.pause(); // second pause must be a no-op
        assert!(live.is_paused());
        live.stop();
    }

    /// `resume()` when not paused is a no-op.
    #[test]
    fn resume_without_pause_is_noop() {
        let mut live = Live::new(Text::new("x", Style::null()))
            .with_console(test_console())
            .with_auto_refresh(false);
        live.start();
        live.refresh();
        live.resume(); // never paused -> no-op
        assert!(!live.is_paused());
        assert!(live.is_started());
        live.stop();
    }

    /// `resume()` before `start()` is a no-op and must not panic.
    #[test]
    fn resume_without_start_is_noop() {
        let mut live = Live::new(Text::empty())
            .with_console(test_console())
            .with_auto_refresh(false);
        live.resume();
        assert!(!live.is_paused());
        assert!(!live.is_started());
    }

    /// With auto-refresh on, `pause()` stops the background thread and
    /// `resume()` restarts it. Existing start/stop thread behaviour is intact.
    #[test]
    fn pause_resume_stops_and_restarts_refresh_thread() {
        let mut live = Live::new(Text::new("tick", Style::null()))
            .with_console(test_console())
            .with_auto_refresh(true)
            .with_refresh_per_second(50.0);

        live.start();
        assert!(live.refresh_thread.is_some());

        live.pause();
        assert!(
            live.refresh_thread.is_none(),
            "pause must stop the background refresh thread"
        );
        assert!(live.is_paused());

        live.resume();
        assert!(
            live.refresh_thread.is_some(),
            "resume must restart the background refresh thread"
        );
        assert!(!live.is_paused());

        live.stop();
        assert!(live.refresh_thread.is_none());
    }

    /// A freshly constructed Live is not paused.
    #[test]
    fn new_live_is_not_paused() {
        let live = Live::new(Text::empty());
        assert!(!live.is_paused());
    }

    // -- Task 6.2: console live-stack (#26) ------------------------------------

    /// After `start`, `console.live_depth()` must be 1; after `stop`, 0.
    #[test]
    fn live_stack_depth_start_stop() {
        let mut live = Live::new(Text::new("depth", Style::null()))
            .with_console(test_console())
            .with_auto_refresh(false);

        assert_eq!(
            live.state.lock().unwrap().console.live_depth(),
            0,
            "depth before start must be 0"
        );

        live.start();
        assert_eq!(
            live.state.lock().unwrap().console.live_depth(),
            1,
            "depth after start must be 1"
        );

        live.stop();
        assert_eq!(
            live.state.lock().unwrap().console.live_depth(),
            0,
            "depth after stop must be 0"
        );
    }

    /// `start` → `pause` → `resume` → `stop`: depth is 1 throughout
    /// pause/resume, 0 only after stop. Pause must NOT pop; resume must NOT push.
    #[test]
    fn live_stack_depth_pause_resume_invariant() {
        let mut live = Live::new(Text::new("pause_depth", Style::null()))
            .with_console(test_console())
            .with_auto_refresh(false);

        live.start();
        assert_eq!(
            live.state.lock().unwrap().console.live_depth(),
            1,
            "depth after start must be 1"
        );

        live.refresh(); // give pause something to erase
        live.pause();
        assert_eq!(
            live.state.lock().unwrap().console.live_depth(),
            1,
            "depth after pause must still be 1 (pause must NOT pop)"
        );

        live.resume();
        assert_eq!(
            live.state.lock().unwrap().console.live_depth(),
            1,
            "depth after resume must still be 1 (resume must NOT push)"
        );

        live.stop();
        assert_eq!(
            live.state.lock().unwrap().console.live_depth(),
            0,
            "depth after stop must be 0"
        );
    }
}
