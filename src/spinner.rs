//! Spinner animation widget.
//!
//! Port of Python rich's `spinner.py`. A spinner selects frames from a named
//! animation based on elapsed time, optionally combined with descriptive text.

use std::fmt;

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::spinners::SPINNERS;
use crate::style::Style;
use crate::text::{Text, TextPart};

// ---------------------------------------------------------------------------
// SpinnerError
// ---------------------------------------------------------------------------

/// Error returned when a spinner name is not found in the SPINNERS map.
#[derive(Debug, Clone)]
pub struct SpinnerError(pub String);

impl fmt::Display for SpinnerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SpinnerError {}

// ---------------------------------------------------------------------------
// Spinner
// ---------------------------------------------------------------------------

/// A spinner animation widget that cycles through frames based on elapsed time.
///
/// # Examples
///
/// ```
/// use gilt::spinner::Spinner;
///
/// let mut spinner = Spinner::new("dots").unwrap();
/// let frame = spinner.render(0.0);
/// ```
#[derive(Debug, Clone)]
pub struct Spinner {
    /// Name of the spinner animation (e.g. "dots", "line").
    pub name: String,
    /// Optional text to display alongside the spinner frame.
    pub text: Option<Text>,
    /// Animation frames.
    pub frames: Vec<String>,
    /// Milliseconds between frames.
    pub interval: f64,
    /// Time (in seconds) when the spinner was first rendered.
    pub start_time: Option<f64>,
    /// Optional style applied to the spinner frame.
    pub style: Option<Style>,
    /// Speed multiplier (1.0 = normal).
    pub speed: f64,
    /// Frame number offset used when speed changes mid-animation.
    pub frame_no_offset: f64,
    /// Pending speed update (applied on next render).
    update_speed: f64,
}

impl Spinner {
    /// Create a new spinner by name.
    ///
    /// Returns `Err(SpinnerError)` if the name is not found in the SPINNERS map.
    pub fn new(name: &str) -> Result<Spinner, SpinnerError> {
        let spinner_data = SPINNERS
            .get(name)
            .ok_or_else(|| SpinnerError(format!("no spinner called {:?}", name)))?;

        Ok(Spinner {
            name: name.to_string(),
            text: None,
            frames: spinner_data.frames.clone(),
            interval: spinner_data.interval,
            start_time: None,
            style: None,
            speed: 1.0,
            frame_no_offset: 0.0,
            update_speed: 0.0,
        })
    }

    /// Builder method: set the text displayed alongside the spinner.
    #[must_use]
    pub fn with_text(mut self, text: Text) -> Self {
        self.text = Some(text);
        self
    }

    /// Builder method: set the style applied to the spinner frame.
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Builder method: set the speed multiplier.
    #[must_use]
    pub fn with_speed(mut self, speed: f64) -> Self {
        self.speed = speed;
        self
    }

    /// Render the spinner for a given time (in seconds).
    ///
    /// On the first call, `start_time` is recorded. Subsequent calls compute
    /// the frame index from elapsed time, speed, and interval.
    pub fn render(&mut self, time: f64) -> Text {
        if self.start_time.is_none() {
            self.start_time = Some(time);
        }

        let elapsed = time - self.start_time.expect("start_time is set above when None");
        let frame_no = (elapsed * self.speed) / (self.interval / 1000.0) + self.frame_no_offset;
        let frame_idx = (frame_no as usize) % self.frames.len();

        let frame_style = self.style.clone().unwrap_or_else(Style::null);
        let frame = Text::new(&self.frames[frame_idx], frame_style);

        // Apply pending speed update
        if self.update_speed != 0.0 {
            self.frame_no_offset = frame_no;
            self.start_time = Some(time);
            self.speed = self.update_speed;
            self.update_speed = 0.0;
        }

        match &self.text {
            None => frame,
            Some(text) => Text::assemble(
                &[
                    TextPart::Rich(frame),
                    TextPart::Raw(" ".to_string()),
                    TextPart::Rich(text.clone()),
                ],
                Style::null(),
            ),
        }
    }

    /// Update spinner attributes after it has been started.
    ///
    /// - `text`: new text to display (if non-empty).
    /// - `style`: new style for the spinner frame.
    /// - `speed`: new speed multiplier (applied smoothly on next render).
    pub fn update(&mut self, text: Option<Text>, style: Option<Style>, speed: Option<f64>) {
        if let Some(t) = text {
            if !t.is_empty() {
                self.text = Some(t);
            }
        }
        if let Some(s) = style {
            self.style = Some(s);
        }
        if let Some(sp) = speed {
            self.update_speed = sp;
        }
    }
}

impl Renderable for Spinner {
    fn gilt_console(&self, _console: &Console, _options: &ConsoleOptions) -> Vec<Segment> {
        // We need a mutable self to call render, so we clone and render at time 0.
        let mut spinner_clone = Spinner {
            name: self.name.clone(),
            text: self.text.clone(),
            frames: self.frames.clone(),
            interval: self.interval,
            start_time: self.start_time,
            style: self.style.clone(),
            speed: self.speed,
            frame_no_offset: self.frame_no_offset,
            update_speed: self.update_speed,
        };
        let text = spinner_clone.render(0.0);
        text.render()
    }
}

impl Spinner {
    /// Measure the spinner by rendering at time 0 and measuring the resulting text.
    pub fn measure(&self) -> Measurement {
        let mut spinner_clone = Spinner {
            name: self.name.clone(),
            text: self.text.clone(),
            frames: self.frames.clone(),
            interval: self.interval,
            start_time: None,
            style: self.style.clone(),
            speed: self.speed,
            frame_no_offset: self.frame_no_offset,
            update_speed: self.update_speed,
        };
        let text = spinner_clone.render(0.0);
        text.measure()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construction_valid_name() {
        let spinner = Spinner::new("dots");
        assert!(spinner.is_ok());
        let spinner = spinner.unwrap();
        assert_eq!(spinner.name, "dots");
        assert_eq!(spinner.frames.len(), 10);
        assert_eq!(spinner.interval, 80.0);
        assert_eq!(spinner.speed, 1.0);
        assert!(spinner.start_time.is_none());
        assert!(spinner.text.is_none());
        assert!(spinner.style.is_none());
    }

    #[test]
    fn test_construction_invalid_name() {
        let spinner = Spinner::new("nonexistent_spinner_xyz");
        assert!(spinner.is_err());
        let err = spinner.unwrap_err();
        assert!(err.0.contains("nonexistent_spinner_xyz"));
    }

    #[test]
    fn test_with_text() {
        let spinner = Spinner::new("dots")
            .unwrap()
            .with_text(Text::new("Loading...", Style::null()));
        assert!(spinner.text.is_some());
        assert_eq!(spinner.text.as_ref().unwrap().plain(), "Loading...");
    }

    #[test]
    fn test_with_style() {
        let style = Style::parse("bold red").unwrap();
        let spinner = Spinner::new("dots").unwrap().with_style(style.clone());
        assert!(spinner.style.is_some());
        assert_eq!(spinner.style.unwrap(), style);
    }

    #[test]
    fn test_with_speed() {
        let spinner = Spinner::new("dots").unwrap().with_speed(2.0);
        assert_eq!(spinner.speed, 2.0);
    }

    #[test]
    fn test_render_at_time_zero_returns_first_frame() {
        let mut spinner = Spinner::new("dots").unwrap();
        let text = spinner.render(0.0);
        // At time 0, elapsed = 0, frame_no = 0, so first frame
        let first_frame = &spinner.frames[0];
        assert_eq!(text.plain(), first_frame.as_str());
    }

    #[test]
    fn test_render_at_different_times_returns_different_frames() {
        let mut spinner = Spinner::new("dots").unwrap();
        let text0 = spinner.render(0.0);
        // interval = 80ms, so at 0.08s we should get frame 1
        let text1 = spinner.render(0.08);
        // At time 0.16s, we should get frame 2
        let text2 = spinner.render(0.16);

        let plain0 = text0.plain().to_string();
        let plain1 = text1.plain().to_string();
        let plain2 = text2.plain().to_string();

        assert_ne!(plain0, plain1);
        assert_ne!(plain1, plain2);
    }

    #[test]
    fn test_render_wraps_around() {
        let mut spinner = Spinner::new("dots").unwrap();
        let _frame_count = spinner.frames.len();
        // Render at time 0 to set start_time
        spinner.render(0.0);
        // Render past all frames (10 frames * 80ms = 800ms = 0.8s)
        let text = spinner.render(0.8);
        // Should wrap back to first frame
        let first_frame = &spinner.frames[0];
        assert_eq!(text.plain(), first_frame.as_str());
    }

    #[test]
    fn test_render_with_text() {
        let mut spinner = Spinner::new("dots")
            .unwrap()
            .with_text(Text::new("Working", Style::null()));
        let text = spinner.render(0.0);
        let plain = text.plain().to_string();
        // Should contain both the frame and the text separated by a space
        assert!(plain.contains("Working"));
        assert!(plain.contains(&spinner.frames[0]));
        // Pattern: "<frame> Working"
        let expected = format!("{} Working", spinner.frames[0]);
        assert_eq!(plain, expected);
    }

    #[test]
    fn test_speed_affects_frame_selection() {
        let mut spinner_normal = Spinner::new("dots").unwrap();
        let mut spinner_fast = Spinner::new("dots").unwrap().with_speed(2.0);

        // Both start at time 0
        spinner_normal.render(0.0);
        spinner_fast.render(0.0);

        // At time 0.08s:
        // normal: frame_no = 0.08 / 0.08 = 1 -> frame[1]
        // fast:   frame_no = 0.08 * 2 / 0.08 = 2 -> frame[2]
        let text_normal = spinner_normal.render(0.08);
        let text_fast = spinner_fast.render(0.08);

        assert_ne!(text_normal.plain(), text_fast.plain());
    }

    #[test]
    fn test_update_text() {
        let mut spinner = Spinner::new("dots").unwrap();
        assert!(spinner.text.is_none());

        spinner.update(Some(Text::new("New text", Style::null())), None, None);
        assert!(spinner.text.is_some());
        assert_eq!(spinner.text.as_ref().unwrap().plain(), "New text");
    }

    #[test]
    fn test_update_style() {
        let mut spinner = Spinner::new("dots").unwrap();
        assert!(spinner.style.is_none());

        let style = Style::parse("bold").unwrap();
        spinner.update(None, Some(style.clone()), None);
        assert_eq!(spinner.style, Some(style));
    }

    #[test]
    fn test_update_speed() {
        let mut spinner = Spinner::new("dots").unwrap();
        assert_eq!(spinner.speed, 1.0);

        // Speed update is deferred until next render
        spinner.update(None, None, Some(3.0));
        assert_eq!(spinner.update_speed, 3.0);
        assert_eq!(spinner.speed, 1.0); // Not yet applied

        // Render to apply the speed change
        spinner.render(0.0);
        spinner.render(0.1);
        assert_eq!(spinner.speed, 3.0);
        assert_eq!(spinner.update_speed, 0.0);
    }

    #[test]
    fn test_update_does_not_set_empty_text() {
        let mut spinner = Spinner::new("dots")
            .unwrap()
            .with_text(Text::new("Original", Style::null()));

        // Empty text should not replace existing text
        spinner.update(Some(Text::empty()), None, None);
        assert_eq!(spinner.text.as_ref().unwrap().plain(), "Original");
    }

    #[test]
    fn test_renderable_trait() {
        let spinner = Spinner::new("dots").unwrap();
        let console = Console::builder().width(80).build();
        let opts = console.options();
        let segments = spinner.gilt_console(&console, &opts);
        assert!(!segments.is_empty());
        let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(!combined.is_empty());
    }

    #[test]
    fn test_renderable_with_text() {
        let spinner = Spinner::new("dots")
            .unwrap()
            .with_text(Text::new("Loading", Style::null()));
        let console = Console::builder().width(80).build();
        let opts = console.options();
        let segments = spinner.gilt_console(&console, &opts);
        let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(combined.contains("Loading"));
    }

    #[test]
    fn test_measure_returns_reasonable_values() {
        let spinner = Spinner::new("dots").unwrap();
        let measurement = spinner.measure();
        // A single braille character is 1 cell wide
        assert!(measurement.minimum >= 1);
        assert!(measurement.maximum >= 1);
    }

    #[test]
    fn test_measure_with_text() {
        let spinner = Spinner::new("dots")
            .unwrap()
            .with_text(Text::new("Loading", Style::null()));
        let measurement = spinner.measure();
        // "<frame> Loading" = 1 + 1 + 7 = 9 cells max width
        // minimum is the longest word which is "Loading" = 7
        assert!(measurement.minimum >= 1);
        assert_eq!(measurement.maximum, 9);
    }

    #[test]
    fn test_start_time_set_on_first_render() {
        let mut spinner = Spinner::new("dots").unwrap();
        assert!(spinner.start_time.is_none());
        spinner.render(5.0);
        assert_eq!(spinner.start_time, Some(5.0));
    }

    #[test]
    fn test_line_spinner() {
        let mut spinner = Spinner::new("line").unwrap();
        assert_eq!(spinner.interval, 130.0);
        let text = spinner.render(0.0);
        assert_eq!(text.plain(), "-");
    }

    #[test]
    fn test_spinner_error_display() {
        let err = SpinnerError("test error".to_string());
        assert_eq!(format!("{}", err), "test error");
    }

    #[test]
    fn test_render_with_style() {
        let style = Style::parse("bold").unwrap();
        let mut spinner = Spinner::new("dots").unwrap().with_style(style);
        let text = spinner.render(0.0);
        // The rendered text should have spans (from the style)
        let segments = text.render();
        // At least one segment should have a style
        let has_styled = segments.iter().any(|s| s.style.is_some());
        assert!(has_styled);
    }

    #[test]
    fn test_various_spinners() {
        // Verify that several different spinners can be created and rendered
        let spinner_names = [
            "dots",
            "line",
            "arc",
            "bouncingBar",
            "star",
            "bounce",
            "toggle",
            "arrow",
            "circle",
        ];
        for name in &spinner_names {
            let mut spinner = Spinner::new(name)
                .unwrap_or_else(|e| panic!("failed to create spinner '{}': {}", name, e));
            let text = spinner.render(0.0);
            assert!(
                !text.plain().is_empty(),
                "spinner '{}' rendered empty text",
                name
            );
        }
    }
}
