//! `gilt-vhs` — tape-as-code for reproducible README/docs demos.
//!
//! A [`Tape`] is a scripted sequence of frames.  Running [`Tape::to_cast`]
//! produces a deterministic asciinema v2 `.cast` string whose event timestamps
//! match the accumulated frame delays — no wall-clock I/O, no sleeping.
//!
//! Running [`Tape::to_svg`] renders only the **final** frame as an SVG image.
//!
//! # Example
//!
//! ```
//! use gilt_vhs::Tape;
//! use gilt::text::Text;
//! use gilt::style::Style;
//! use std::time::Duration;
//!
//! let tape = Tape::new()
//!     .frame(Text::new("Loading…", Style::null()), Duration::ZERO)
//!     .frame(Text::new("Done!", Style::null()), Duration::from_millis(500));
//!
//! let cast = tape.to_cast(Some("demo"));
//! assert!(cast.contains("\"version\":2"));
//! ```

use std::sync::{Arc, Mutex};
use std::time::Duration;

use gilt::console::{Console, Renderable};

// ---------------------------------------------------------------------------
// Frame storage
// ---------------------------------------------------------------------------

/// A single tape frame: cumulative timestamp (seconds) + renderable content.
struct Frame {
    /// Cumulative seconds from the tape start (sum of all preceding delays
    /// plus this frame's own delay).
    cumulative_secs: f64,
    /// Boxed renderable content.
    renderable: Box<dyn Renderable>,
}

// ---------------------------------------------------------------------------
// Tape
// ---------------------------------------------------------------------------

/// A scripted sequence of frames that produces a deterministic asciinema cast.
///
/// Build with [`Tape::new`], add frames with [`Tape::frame`], then export with
/// [`Tape::to_cast`] or [`Tape::to_svg`].
pub struct Tape {
    frames: Vec<Frame>,
    /// Running cursor of cumulative time, updated by each [`frame`](Self::frame) call.
    cursor_secs: f64,
    /// Console width in columns (default 80).
    width: usize,
    /// Console height in rows (default 24).
    height: usize,
}

impl Tape {
    /// Create a new empty tape with default 80×24 console dimensions.
    pub fn new() -> Self {
        Tape {
            frames: Vec::new(),
            cursor_secs: 0.0,
            width: 80,
            height: 24,
        }
    }

    /// Append a frame.
    ///
    /// `delay` is the time since the *previous* frame (or since the start for
    /// the first frame).  Internally this is converted to a cumulative timestamp.
    pub fn frame(mut self, renderable: impl Renderable + 'static, delay: Duration) -> Self {
        self.cursor_secs += delay.as_secs_f64();
        self.frames.push(Frame {
            cumulative_secs: self.cursor_secs,
            renderable: Box::new(renderable),
        });
        self
    }

    /// Override the console width (default 80).
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Override the console height (default 24).
    pub fn height(mut self, height: usize) -> Self {
        self.height = height;
        self
    }

    // -------------------------------------------------------------------------
    // to_cast — deterministic asciinema v2 .cast
    // -------------------------------------------------------------------------

    /// Build a deterministic asciinema v2 `.cast` from the tape's frames.
    ///
    /// # Determinism
    ///
    /// Timing is driven by an **injected clock** (via
    /// [`Console::with_asciinema_clock`]) backed by an `Arc<Mutex<f64>>` cell.
    /// Before printing each frame, `to_cast` sets the cell's value to that
    /// frame's cumulative timestamp.  The clock closure reads the cell, so
    /// every event recorded for that frame carries the exact cumulative time —
    /// no wall-clock reads, no sleeping.
    ///
    /// The lazy-start logic in `maybe_record_asciinema_event` snapshots
    /// `asciinema_start` on the *first* event.  Because our clock starts at
    /// `0.0` and the first frame sets the cell to its cumulative time (which
    /// may also be `0.0`), `elapsed = now - start = 0.0` for that frame —
    /// matching the expected behaviour exactly.
    pub fn to_cast(&self, title: Option<&str>) -> String {
        // Shared mutable clock value — advanced per frame below.
        let clock_cell: Arc<Mutex<f64>> = Arc::new(Mutex::new(0.0));
        let clock_clone = Arc::clone(&clock_cell);

        // Build recording console with the injected clock.
        let mut console = Console::builder()
            .width(self.width)
            .height(self.height)
            .force_terminal(true)
            .no_color(true)
            .record(true)
            .build()
            .with_asciinema_clock(move || *clock_clone.lock().unwrap());

        console.begin_asciinema_record();

        for frame in &self.frames {
            // Advance the clock to this frame's cumulative timestamp.
            *clock_cell.lock().unwrap() = frame.cumulative_secs;
            // Print the renderable — this triggers `maybe_record_asciinema_event`
            // which reads the (now-updated) clock and appends a timed event.
            console.print(frame.renderable.as_ref());
        }

        console.export_asciinema(title)
    }

    // -------------------------------------------------------------------------
    // to_svg — SVG of the final frame
    // -------------------------------------------------------------------------

    /// Render the **final** frame as an SVG document.
    ///
    /// If the tape has no frames, returns an empty string.
    pub fn to_svg(&self, title: &str) -> String {
        let last = match self.frames.last() {
            Some(f) => f,
            None => return String::new(),
        };

        let mut console = Console::builder()
            .width(self.width)
            .height(self.height)
            .force_terminal(true)
            .no_color(true)
            .record(true)
            .build();

        console.print(last.renderable.as_ref());
        console.export_svg(title, None, false, None, 0.61)
    }
}

impl Default for Tape {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests (RED → GREEN)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use gilt::style::Style;
    use gilt::text::Text;

    /// Parse one line of a `.cast` as a JSON array `[t, "o", s]`.
    fn parse_event(line: &str) -> (f64, String, String) {
        let v: serde_json::Value =
            serde_json::from_str(line).expect("event line must be valid JSON");
        let arr = v.as_array().expect("event must be a JSON array");
        let t = arr[0].as_f64().expect("timestamp must be f64");
        let kind = arr[1].as_str().expect("kind must be a string").to_owned();
        let data = arr[2].as_str().expect("data must be a string").to_owned();
        (t, kind, data)
    }

    #[test]
    fn test_to_cast_header_and_events() {
        let tape = Tape::new()
            .frame(Text::new("a", Style::null()), Duration::ZERO)
            .frame(Text::new("b", Style::null()), Duration::from_millis(500));

        let cast = tape.to_cast(Some("demo"));

        let lines: Vec<&str> = cast.lines().collect();
        // There must be at least 3 lines: header + 2 events.
        assert!(
            lines.len() >= 3,
            "expected at least 3 lines, got {}:\n{}",
            lines.len(),
            cast
        );

        // --- Header (line 0) ---
        let header: serde_json::Value =
            serde_json::from_str(lines[0]).expect("header must be valid JSON");
        assert_eq!(header["version"], 2, "version must be 2");
        assert_eq!(header["width"], 80u64, "default width must be 80");

        // --- Events ---
        // Find event lines that contain "a" and "b" in their data.
        let event_lines: Vec<&str> = lines[1..].to_vec();

        let events: Vec<(f64, String, String)> =
            event_lines.iter().map(|l| parse_event(l)).collect();

        // There should be at least 2 events (one per frame).
        assert!(
            events.len() >= 2,
            "expected at least 2 events, got {}",
            events.len()
        );

        // Find the event containing "a" — should be at t ≈ 0.0.
        let ev_a = events
            .iter()
            .find(|(_, _, data)| data.contains('a'))
            .expect("must have an event containing 'a'");
        assert!(
            (ev_a.0 - 0.0).abs() < 1e-6,
            "frame 'a' timestamp must be 0.0, got {}",
            ev_a.0
        );
        assert_eq!(ev_a.1, "o", "event kind must be 'o'");

        // Find the event containing "b" — should be at t ≈ 0.5.
        let ev_b = events
            .iter()
            .find(|(_, _, data)| data.contains('b'))
            .expect("must have an event containing 'b'");
        assert!(
            (ev_b.0 - 0.5).abs() < 1e-6,
            "frame 'b' timestamp must be 0.5, got {}",
            ev_b.0
        );
        assert_eq!(ev_b.1, "o", "event kind must be 'o'");
    }

    #[test]
    fn test_to_svg_contains_final_frame() {
        let tape = Tape::new()
            .frame(Text::new("first frame", Style::null()), Duration::ZERO)
            .frame(
                Text::new("final frame content", Style::null()),
                Duration::from_millis(200),
            );

        let svg = tape.to_svg("svg test");
        assert!(svg.contains("<svg"), "SVG must start with <svg");
        assert!(
            svg.contains("final frame content"),
            "SVG must contain the final frame's text"
        );
        // The first frame's unique text should NOT be in the SVG (only last frame).
        assert!(
            !svg.contains("first frame"),
            "SVG must not contain earlier frame text"
        );
    }

    #[test]
    fn test_empty_tape_to_svg() {
        let tape = Tape::new();
        let svg = tape.to_svg("empty");
        assert!(svg.is_empty(), "empty tape must produce empty SVG");
    }

    #[test]
    fn test_tape_default_dimensions() {
        let tape = Tape::new().frame(Text::new("hi", Style::null()), Duration::ZERO);
        let cast = tape.to_cast(None);
        let header: serde_json::Value = serde_json::from_str(cast.lines().next().unwrap()).unwrap();
        assert_eq!(header["width"], 80u64);
        assert_eq!(header["height"], 24u64);
    }

    #[test]
    fn test_tape_custom_dimensions() {
        let tape = Tape::new()
            .width(120)
            .height(40)
            .frame(Text::new("hi", Style::null()), Duration::ZERO);
        let cast = tape.to_cast(None);
        let header: serde_json::Value = serde_json::from_str(cast.lines().next().unwrap()).unwrap();
        assert_eq!(header["width"], 120u64);
        assert_eq!(header["height"], 40u64);
    }
}
