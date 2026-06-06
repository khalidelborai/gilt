//! Asciinema v2 `.cast` export for [`Console`].
//!
//! Gated on `#[cfg(feature = "asciinema")]`.  When enabled, adds:
//!
//! - [`Console::with_asciinema_clock`] — injectable clock for WASM-safety and
//!   deterministic testing.
//! - [`Console::begin_asciinema_record`] — start a timed recording session.
//! - [`Console::export_asciinema`] — emit asciinema v2 NDJSON `.cast` text.
//! - [`Console::save_asciinema`] — write the `.cast` to a file (native only).

#[cfg(feature = "asciinema")]
use crate::console::Console;

// ---------------------------------------------------------------------------
// Clock storage type (feature-gated)
// ---------------------------------------------------------------------------

/// Boxed clock function: returns elapsed seconds as `f64`.
///
/// Stored inside `Console` when the `asciinema` feature is enabled.
#[cfg(feature = "asciinema")]
pub type AsciinemaClock = Box<dyn Fn() -> f64 + Send + Sync + 'static>;

// ---------------------------------------------------------------------------
// Console impl
// ---------------------------------------------------------------------------

#[cfg(feature = "asciinema")]
impl Console {
    // -- Clock injection ----------------------------------------------------

    /// Replace the asciinema clock with a custom implementation.
    ///
    /// The clock is a `Fn() -> f64` returning the *current time in seconds*
    /// (absolute, not relative — the recording start time is subtracted
    /// internally by [`begin_asciinema_record`](Self::begin_asciinema_record)).
    ///
    /// Use this for:
    /// - **Deterministic tests**: pass a mock that returns pre-set values.
    /// - **WASM targets**: the default native clock (`Instant`) is not
    ///   available on wasm; the default there is `|| 0.0`; supply a
    ///   `performance.now()` wrapper via JS bindings if you need real timing.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "asciinema")] {
    /// use gilt::console::Console;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let counter = Arc::new(Mutex::new(0u32));
    /// let counter_clone = Arc::clone(&counter);
    /// let mut console = Console::builder()
    ///     .width(80)
    ///     .no_color(true)
    ///     .build()
    ///     .with_asciinema_clock(move || {
    ///         let mut c = counter_clone.lock().unwrap();
    ///         let v = *c;
    ///         *c += 1;
    ///         v as f64 * 0.5
    ///     });
    /// # }
    /// ```
    pub fn with_asciinema_clock<F>(mut self, clock: F) -> Self
    where
        F: Fn() -> f64 + Send + Sync + 'static,
    {
        self.asciinema_clock = Some(Box::new(clock));
        self
    }

    // -- Recording start ----------------------------------------------------

    /// Begin a timed asciinema recording session.
    ///
    /// Sets the console's `record` flag, records the start time from the
    /// clock, and clears any previously-collected asciinema events.  Each
    /// subsequent call to `write_segments` will append a `(elapsed_secs,
    /// ansi_string)` entry to the events list.
    ///
    /// Call [`export_asciinema`](Self::export_asciinema) when done.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "asciinema")] {
    /// use gilt::console::Console;
    ///
    /// let mut console = Console::builder()
    ///     .width(80)
    ///     .no_color(true)
    ///     .build();
    /// console.begin_asciinema_record();
    /// console.print_text("hello");
    /// let cast = console.export_asciinema(Some("my session"));
    /// assert!(cast.contains("\"version\":2") || cast.contains("\"version\": 2"));
    /// # }
    /// ```
    pub fn begin_asciinema_record(&mut self) {
        // Enable the generic record flag so record_buffer is also populated
        // (needed for the t=0 fallback path).
        self.record = true;
        // Reset the timed-events list.
        self.asciinema_events.clear();
        // Clear the start sentinel so the first event lazily initialises it.
        // A negative sentinel (f64::NAN is also fine) signals "not yet started".
        self.asciinema_start = f64::NAN;
        // Mark the session as active so write_segments starts capturing.
        self.asciinema_active = true;
    }

    // -- Export -------------------------------------------------------------

    /// Export the recorded session as an asciinema v2 `.cast` NDJSON string.
    ///
    /// **Line 1** is the header JSON object (`{"version":2,"width":W,...}`).
    /// **Subsequent lines** are event arrays `[elapsed_secs, "o", ansi_string]`.
    ///
    /// If `begin_asciinema_record` was called and events were captured, those
    /// are used.  Otherwise the method falls back to a single `t=0` event
    /// containing the entire `record_buffer` rendered as ANSI — useful for
    /// exporting a static styled report as a one-frame cast.
    ///
    /// The ANSI strings are JSON-escaped via `serde_json::to_string` so they
    /// are safe for embedding in any JSON parser.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "asciinema")] {
    /// use gilt::console::Console;
    ///
    /// let mut console = Console::builder()
    ///     .width(80)
    ///     .height(24)
    ///     .no_color(true)
    ///     .record(true)
    ///     .build();
    /// console.print_text("hello cast");
    /// let cast = console.export_asciinema(Some("demo"));
    /// let first_line: serde_json::Value = serde_json::from_str(cast.lines().next().unwrap()).unwrap();
    /// assert_eq!(first_line["version"], 2);
    /// # }
    /// ```
    pub fn export_asciinema(&self, title: Option<&str>) -> String {
        let w = self.width();
        let h = self.height();

        // --- Build the header -----------------------------------------------
        let mut header = serde_json::Map::new();
        header.insert("version".into(), serde_json::Value::Number(2.into()));
        header.insert("width".into(), serde_json::Value::Number((w as u64).into()));
        header.insert(
            "height".into(),
            serde_json::Value::Number((h as u64).into()),
        );
        if let Some(t) = title {
            header.insert("title".into(), serde_json::Value::String(t.to_string()));
        }
        // Env block — harmless metadata, standard in asciinema casts.
        {
            let mut env = serde_json::Map::new();
            let term = std::env::var("TERM").unwrap_or_else(|_| "xterm-256color".to_string());
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
            env.insert("TERM".into(), serde_json::Value::String(term));
            env.insert("SHELL".into(), serde_json::Value::String(shell));
            header.insert("env".into(), serde_json::Value::Object(env));
        }

        let mut cast = String::new();
        let header_json =
            serde_json::to_string(&serde_json::Value::Object(header)).unwrap_or_default();
        cast.push_str(&header_json);
        cast.push('\n');

        // --- Choose event source -------------------------------------------
        if !self.asciinema_events.is_empty() {
            // Timed path: emit one line per captured event.
            for (t, ansi) in &self.asciinema_events {
                let event = serde_json::json!([*t, "o", ansi]);
                let line = serde_json::to_string(&event).unwrap_or_default();
                cast.push_str(&line);
                cast.push('\n');
            }
        } else {
            // Fallback path: render the record_buffer and emit as t=0.
            let rendered = self.render_buffer(&self.record_buffer);
            if !rendered.is_empty() {
                let event = serde_json::json!([0.0f64, "o", rendered]);
                let line = serde_json::to_string(&event).unwrap_or_default();
                cast.push_str(&line);
                cast.push('\n');
            }
        }

        // Trim the trailing newline so `cast.lines()` gives the exact count.
        if cast.ends_with('\n') {
            cast.pop();
        }

        cast
    }

    // -- Save to file (native only) -----------------------------------------

    /// Write the asciinema `.cast` to a file at `path`.
    ///
    /// Convenience wrapper around [`export_asciinema`](Self::export_asciinema).
    /// Not available on wasm targets (file I/O is not available there).
    ///
    /// # Errors
    ///
    /// Returns `Err` if the file cannot be created or written.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(all(feature = "asciinema", not(target_arch = "wasm32")))] {
    /// use gilt::console::Console;
    ///
    /// let mut console = Console::builder().width(80).record(true).build();
    /// console.print_text("saved session");
    /// console.save_asciinema("session.cast", Some("my session")).unwrap();
    /// # }
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_asciinema(
        &self,
        path: impl AsRef<std::path::Path>,
        title: Option<&str>,
    ) -> std::io::Result<()> {
        let cast = self.export_asciinema(title);
        std::fs::write(path, cast)
    }

    // -- Internal helpers ---------------------------------------------------

    /// Return the current time from the injected clock, or the default clock.
    ///
    /// Default clock:
    /// - **native**: current unix time in fractional seconds via
    ///   `SystemTime::now()`.  `begin_asciinema_record` snapshots the start
    ///   time; `write_segments` subtracts it to produce elapsed seconds.
    /// - **wasm32** or **no clock set**: `0.0`.
    pub(crate) fn clock_now(&self) -> f64 {
        if let Some(ref clock) = self.asciinema_clock {
            return clock();
        }
        // Default native clock.
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64()
        }
        #[cfg(target_arch = "wasm32")]
        {
            0.0
        }
    }

    /// Record one asciinema event during an active timed session.
    ///
    /// Called from `write_segments` whenever an asciinema session is active.
    /// Renders `segments` to ANSI, computes elapsed time (lazily snapshotting
    /// the start time on the first event), and appends to `asciinema_events`.
    ///
    /// This is intentionally cheap when not recording (`asciinema_active = false`).
    pub(crate) fn maybe_record_asciinema_event(&mut self, segments: &[crate::segment::Segment]) {
        if !self.asciinema_active {
            return;
        }
        let now = self.clock_now();
        // Lazy start: initialise on the very first event so that the first
        // recorded chunk always gets elapsed = 0.0, regardless of how long
        // `begin_asciinema_record` itself took to execute.
        if self.asciinema_start.is_nan() {
            self.asciinema_start = now;
        }
        let elapsed = (now - self.asciinema_start).max(0.0);
        let ansi = self.render_buffer(segments);
        if !ansi.is_empty() {
            self.asciinema_events.push((elapsed, ansi));
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "console_asciinema_tests.rs"]
mod tests;
