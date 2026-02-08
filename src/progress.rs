//! Progress tracking system -- configurable progress bars with live display.
//!
//! Port of Python's `rich/progress.py`. Provides task tracking with
//! completion percentages, configurable columns (text, bar, spinner,
//! time, speed), live-updating display, and iterator wrapping.

use std::collections::{HashMap, VecDeque};
use std::io::{self, Read};
use std::time::SystemTime;

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::filesize;
use crate::live::Live;
use crate::progress_bar::ProgressBar;
use crate::segment::Segment;
use crate::spinner::Spinner;
use crate::style::Style;
use crate::table::Table;
use crate::text::{JustifyMethod, Text};

// ---------------------------------------------------------------------------
// TaskId
// ---------------------------------------------------------------------------

/// Unique identifier for a progress task.
pub type TaskId = usize;

// ---------------------------------------------------------------------------
// ProgressSample
// ---------------------------------------------------------------------------

/// A timestamped progress measurement used for speed calculation.
#[derive(Debug, Clone)]
pub struct ProgressSample {
    pub timestamp: f64,
    pub completed: f64,
}

// ---------------------------------------------------------------------------
// Task
// ---------------------------------------------------------------------------

/// A tracked unit of work within a [`Progress`] display.
///
/// Each task has a description, optional total, and records progress
/// samples for speed estimation.
#[derive(Debug, Clone)]
pub struct Task {
    /// Unique identifier for this task.
    pub id: TaskId,
    /// Human-readable description shown in the progress display.
    pub description: String,
    /// Total number of steps (None = indeterminate).
    pub total: Option<f64>,
    /// Number of steps completed so far.
    pub completed: f64,
    /// Whether this task is visible in the display.
    pub visible: bool,
    /// Arbitrary key-value fields for template substitution.
    pub fields: HashMap<String, String>,
    /// Time when this task was started (seconds since epoch).
    pub start_time: Option<f64>,
    /// Time when this task was stopped.
    pub stop_time: Option<f64>,
    /// Time when this task was marked finished.
    pub finished_time: Option<f64>,
    /// Cached speed at finish time.
    pub finished_speed: Option<f64>,
    /// Sliding window of samples for speed calculation.
    samples: VecDeque<ProgressSample>,
    /// All recorded progress samples.
    progress: Vec<ProgressSample>,
}

impl Task {
    /// Create a new task with the given id, description, and optional total.
    pub fn new(id: TaskId, description: &str, total: Option<f64>) -> Self {
        Task {
            id,
            description: description.to_string(),
            total,
            completed: 0.0,
            visible: true,
            fields: HashMap::new(),
            start_time: None,
            stop_time: None,
            finished_time: None,
            finished_speed: None,
            samples: VecDeque::new(),
            progress: Vec::new(),
        }
    }

    /// Whether this task has been started.
    pub fn started(&self) -> bool {
        self.start_time.is_some()
    }

    /// Whether this task has been finished.
    pub fn finished(&self) -> bool {
        self.finished_time.is_some()
    }

    /// The remaining work (total - completed), if total is known.
    pub fn remaining(&self) -> Option<f64> {
        self.total.map(|t| (t - self.completed).max(0.0))
    }

    /// Elapsed time in seconds since the task was started.
    pub fn elapsed(&self) -> Option<f64> {
        self.start_time.map(|start| {
            let end = self.stop_time.unwrap_or_else(current_time_secs);
            (end - start).max(0.0)
        })
    }

    /// Percentage complete (0..100). Returns 0.0 if total is None or zero.
    pub fn percentage(&self) -> f64 {
        match self.total {
            Some(total) if total > 0.0 => ((self.completed / total) * 100.0).clamp(0.0, 100.0),
            _ => 0.0,
        }
    }

    /// Calculate speed from the sliding window of samples.
    ///
    /// Returns the average rate of change per second computed from the
    /// first and last samples in the window.
    pub fn speed(&self) -> Option<f64> {
        if self.finished() {
            return self.finished_speed;
        }
        if self.samples.len() < 2 {
            return None;
        }
        let first = self.samples.front().unwrap();
        let last = self.samples.back().unwrap();
        let time_delta = last.timestamp - first.timestamp;
        if time_delta <= 0.0 {
            return None;
        }
        let completed_delta = last.completed - first.completed;
        Some(completed_delta / time_delta)
    }

    /// Estimated time remaining in seconds, based on current speed.
    pub fn time_remaining(&self) -> Option<f64> {
        if self.finished() {
            return Some(0.0);
        }
        let remaining = self.remaining()?;
        let speed = self.speed()?;
        if speed <= 0.0 {
            return None;
        }
        Some(remaining / speed)
    }

    /// Record a progress sample for speed estimation.
    ///
    /// Samples older than `speed_estimate_period` seconds are pruned
    /// from the sliding window.
    fn record_sample(&mut self, timestamp: f64, speed_estimate_period: f64) {
        self.samples.push_back(ProgressSample {
            timestamp,
            completed: self.completed,
        });
        self.progress.push(ProgressSample {
            timestamp,
            completed: self.completed,
        });

        // Prune samples outside the estimation window.
        let cutoff = timestamp - speed_estimate_period;
        while let Some(front) = self.samples.front() {
            if front.timestamp < cutoff {
                self.samples.pop_front();
            } else {
                break;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Time helper
// ---------------------------------------------------------------------------

/// Return the current time as seconds since the UNIX epoch.
fn current_time_secs() -> f64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// Format a duration in seconds as `H:MM:SS`.
fn format_time(seconds: f64) -> String {
    let total = seconds.round() as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

// ---------------------------------------------------------------------------
// ProgressColumn trait
// ---------------------------------------------------------------------------

/// Trait for columns that render task information in a progress display.
///
/// Each column is responsible for producing a [`Text`] renderable from
/// a [`Task`] reference.
pub trait ProgressColumn: Send + Sync {
    /// Render this column for the given task.
    fn render(&self, task: &Task) -> Text;

    /// Maximum refresh rate in seconds, or None for unlimited.
    fn max_refresh(&self) -> Option<f64> {
        None
    }
}

// ---------------------------------------------------------------------------
// TextColumn
// ---------------------------------------------------------------------------

/// A column that renders text with simple template substitution.
///
/// Supported placeholders:
/// - `{task.description}` - task description
/// - `{task.percentage:.0f}` (or any format) - percentage complete
/// - `{task.completed}` - completed count
/// - `{task.total}` - total count (or "?" if None)
/// - `{task.speed}` - current speed (or "?" if unknown)
///
/// Any field key `{task.fields.KEY}` substitutes the corresponding
/// entry from `task.fields`.
#[derive(Debug, Clone)]
pub struct TextColumn {
    /// Template string with `{task.*}` placeholders.
    pub text: String,
    /// Style applied to the rendered text.
    pub style: Option<Style>,
    /// Horizontal justification.
    pub justify: JustifyMethod,
}

impl TextColumn {
    /// Create a new TextColumn with the given template.
    pub fn new(text: &str) -> Self {
        TextColumn {
            text: text.to_string(),
            style: None,
            justify: JustifyMethod::Left,
        }
    }

    /// Builder: set the style.
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Builder: set justification.
    #[must_use]
    pub fn with_justify(mut self, justify: JustifyMethod) -> Self {
        self.justify = justify;
        self
    }

    /// Perform template substitution for a task.
    fn substitute(&self, task: &Task) -> String {
        let mut result = self.text.clone();

        // {task.description}
        result = result.replace("{task.description}", &task.description);

        // {task.percentage} and {task.percentage:.Nf}
        if result.contains("{task.percentage") {
            let pct = task.percentage();
            // Handle format specifiers like {task.percentage:.0f}
            if let Some(start) = result.find("{task.percentage:") {
                if let Some(end) = result[start..].find('}') {
                    let spec = &result[start..start + end + 1];
                    // Parse precision from :.Nf pattern
                    let formatted = if spec.contains(".0f") {
                        format!("{pct:.0}")
                    } else if spec.contains(".1f") {
                        format!("{pct:.1}")
                    } else if spec.contains(".2f") {
                        format!("{pct:.2}")
                    } else {
                        format!("{pct:.1}")
                    };
                    result = result.replace(spec, &formatted);
                }
            }
            result = result.replace("{task.percentage}", &format!("{pct:.1}"));
        }

        // {task.completed}
        result = result.replace("{task.completed}", &format!("{}", task.completed));

        // {task.total}
        let total_str = match task.total {
            Some(t) => format!("{t}"),
            None => "?".to_string(),
        };
        result = result.replace("{task.total}", &total_str);

        // {task.speed}
        let speed_str = match task.speed() {
            Some(s) => format!("{s:.1}"),
            None => "?".to_string(),
        };
        result = result.replace("{task.speed}", &speed_str);

        // {task.fields.KEY}
        for (key, value) in &task.fields {
            let placeholder = format!("{{task.fields.{key}}}");
            result = result.replace(&placeholder, value);
        }

        result
    }
}

impl ProgressColumn for TextColumn {
    fn render(&self, task: &Task) -> Text {
        let content = self.substitute(task);
        let style = self.style.clone().unwrap_or_else(Style::null);
        let mut text = Text::new(&content, style);
        text.justify = Some(self.justify);
        text
    }
}

// ---------------------------------------------------------------------------
// BarColumn
// ---------------------------------------------------------------------------

/// A column that renders a progress bar.
#[derive(Debug, Clone)]
pub struct BarColumn {
    /// Fixed width of the bar, or None for flexible sizing.
    pub bar_width: Option<usize>,
    /// Style for the bar background.
    pub style: String,
    /// Style for the completed portion.
    pub complete_style: String,
    /// Style for a finished bar.
    pub finished_style: String,
    /// Style for pulse animation.
    pub pulse_style: String,
}

impl BarColumn {
    /// Create a new BarColumn with default styles.
    pub fn new() -> Self {
        BarColumn {
            bar_width: Some(40),
            style: "bar.back".to_string(),
            complete_style: "bar.complete".to_string(),
            finished_style: "bar.finished".to_string(),
            pulse_style: "bar.pulse".to_string(),
        }
    }

    /// Builder: set bar width.
    #[must_use]
    pub fn with_bar_width(mut self, width: Option<usize>) -> Self {
        self.bar_width = width;
        self
    }
}

impl Default for BarColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressColumn for BarColumn {
    fn render(&self, task: &Task) -> Text {
        let bar = ProgressBar::new()
            .with_total(task.total)
            .with_completed(task.completed)
            .with_width(self.bar_width)
            .with_style(&self.style)
            .with_complete_style(&self.complete_style)
            .with_finished_style(&self.finished_style)
            .with_pulse_style(&self.pulse_style);

        // Render the bar through the Renderable trait to get segments,
        // then convert to text.
        let console = Console::builder()
            .width(self.bar_width.unwrap_or(40))
            .color_system("truecolor")
            .build();
        let opts = console.options();
        let segments = bar.rich_console(&console, &opts);

        let mut text = Text::empty();
        for seg in &segments {
            text.append_str(&seg.text, seg.style.clone());
        }
        text.end = String::new();
        text
    }
}

// ---------------------------------------------------------------------------
// SpinnerColumn
// ---------------------------------------------------------------------------

/// A column that renders a spinner animation.
#[derive(Debug, Clone)]
pub struct SpinnerColumn {
    /// Name of the spinner (from the SPINNERS registry).
    pub spinner_name: String,
    /// Style for the spinner frame.
    pub style: Option<Style>,
    /// Text shown when the task is finished.
    pub finished_text: Text,
}

impl SpinnerColumn {
    /// Create a new SpinnerColumn with the given spinner name.
    pub fn new(name: &str) -> Self {
        SpinnerColumn {
            spinner_name: name.to_string(),
            style: None,
            finished_text: Text::styled(
                "\u{2714}",
                Style::parse("green").unwrap_or_else(|_| Style::null()),
            ),
        }
    }

    /// Builder: set the style.
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Builder: set the finished text.
    #[must_use]
    pub fn with_finished_text(mut self, text: Text) -> Self {
        self.finished_text = text;
        self
    }
}

impl Default for SpinnerColumn {
    fn default() -> Self {
        Self::new("dots")
    }
}

impl ProgressColumn for SpinnerColumn {
    fn render(&self, task: &Task) -> Text {
        if task.finished() {
            return self.finished_text.clone();
        }

        let mut spinner = match Spinner::new(&self.spinner_name) {
            Ok(s) => s,
            Err(_) => return Text::new("?", Style::null()),
        };
        if let Some(ref style) = self.style {
            spinner = spinner.with_style(style.clone());
        }

        let elapsed = task.elapsed().unwrap_or(0.0);
        spinner.render(elapsed)
    }

    fn max_refresh(&self) -> Option<f64> {
        // Spinners typically update at ~12.5 FPS
        Some(0.08)
    }
}

// ---------------------------------------------------------------------------
// TimeElapsedColumn
// ---------------------------------------------------------------------------

/// A column that shows elapsed time as `[H:MM:SS]`.
#[derive(Debug, Clone)]
pub struct TimeElapsedColumn;

impl Default for TimeElapsedColumn {
    fn default() -> Self {
        Self
    }
}

impl ProgressColumn for TimeElapsedColumn {
    fn render(&self, task: &Task) -> Text {
        let elapsed = task.elapsed().unwrap_or(0.0);
        let formatted = format_time(elapsed);
        Text::new(
            &formatted,
            Style::parse("progress.elapsed").unwrap_or_else(|_| Style::null()),
        )
    }
}

// ---------------------------------------------------------------------------
// TimeRemainingColumn
// ---------------------------------------------------------------------------

/// A column that shows estimated remaining time as `[H:MM:SS]` or
/// `-:--:--` when the estimate is unavailable.
#[derive(Debug, Clone)]
pub struct TimeRemainingColumn {
    /// Whether to show compact format.
    pub compact: bool,
    /// Whether to show elapsed time when finished.
    pub elapsed_when_finished: bool,
}

impl TimeRemainingColumn {
    /// Create a new TimeRemainingColumn with default settings.
    pub fn new() -> Self {
        TimeRemainingColumn {
            compact: false,
            elapsed_when_finished: false,
        }
    }
}

impl Default for TimeRemainingColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressColumn for TimeRemainingColumn {
    fn render(&self, task: &Task) -> Text {
        let style = Style::parse("progress.remaining").unwrap_or_else(|_| Style::null());

        if task.finished() {
            if self.elapsed_when_finished {
                let elapsed = task.elapsed().unwrap_or(0.0);
                return Text::new(&format_time(elapsed), style);
            }
            return Text::new("0:00", style);
        }

        match task.time_remaining() {
            Some(remaining) if remaining.is_finite() => Text::new(&format_time(remaining), style),
            _ => Text::new("-:--:--", style),
        }
    }
}

// ---------------------------------------------------------------------------
// TaskProgressColumn
// ---------------------------------------------------------------------------

/// A column that shows `completed/total` counts.
#[derive(Debug, Clone)]
pub struct TaskProgressColumn {
    /// Separator between completed and total.
    pub separator: String,
}

impl TaskProgressColumn {
    /// Create a new TaskProgressColumn with the default separator.
    pub fn new() -> Self {
        TaskProgressColumn {
            separator: "/".to_string(),
        }
    }

    /// Builder: set the separator.
    #[must_use]
    pub fn with_separator(mut self, sep: &str) -> Self {
        self.separator = sep.to_string();
        self
    }
}

impl Default for TaskProgressColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressColumn for TaskProgressColumn {
    fn render(&self, task: &Task) -> Text {
        let style = Style::parse("progress.percentage").unwrap_or_else(|_| Style::null());
        let completed = task.completed;
        let total_str = match task.total {
            Some(t) => format!("{t}"),
            None => "?".to_string(),
        };
        Text::new(&format!("{completed}{}{total_str}", self.separator), style)
    }
}

// ---------------------------------------------------------------------------
// FileSizeColumn
// ---------------------------------------------------------------------------

/// A column that shows the completed amount as a human-readable file size.
#[derive(Debug, Clone)]
pub struct FileSizeColumn;

impl Default for FileSizeColumn {
    fn default() -> Self {
        Self
    }
}

impl ProgressColumn for FileSizeColumn {
    fn render(&self, task: &Task) -> Text {
        let size = task.completed as u64;
        let formatted = filesize::decimal(size, 1, " ");
        Text::new(
            &formatted,
            Style::parse("progress.filesize").unwrap_or_else(|_| Style::null()),
        )
    }
}

// ---------------------------------------------------------------------------
// TotalFileSizeColumn
// ---------------------------------------------------------------------------

/// A column that shows the total as a human-readable file size.
#[derive(Debug, Clone)]
pub struct TotalFileSizeColumn;

impl Default for TotalFileSizeColumn {
    fn default() -> Self {
        Self
    }
}

impl ProgressColumn for TotalFileSizeColumn {
    fn render(&self, task: &Task) -> Text {
        let size = task.total.unwrap_or(0.0) as u64;
        let formatted = filesize::decimal(size, 1, " ");
        Text::new(
            &formatted,
            Style::parse("progress.filesize.total").unwrap_or_else(|_| Style::null()),
        )
    }
}

// ---------------------------------------------------------------------------
// MofNCompleteColumn
// ---------------------------------------------------------------------------

/// A column that shows `M/N` with optional separator customization.
#[derive(Debug, Clone)]
pub struct MofNCompleteColumn {
    /// Separator between M and N.
    pub separator: String,
}

impl MofNCompleteColumn {
    /// Create a new MofNCompleteColumn with the default `/` separator.
    pub fn new() -> Self {
        MofNCompleteColumn {
            separator: "/".to_string(),
        }
    }

    /// Builder: set the separator.
    #[must_use]
    pub fn with_separator(mut self, sep: &str) -> Self {
        self.separator = sep.to_string();
        self
    }
}

impl Default for MofNCompleteColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressColumn for MofNCompleteColumn {
    fn render(&self, task: &Task) -> Text {
        let completed = task.completed as u64;
        let total_str = match task.total {
            Some(t) => format!("{}", t as u64),
            None => "?".to_string(),
        };
        let style = Style::parse("progress.percentage").unwrap_or_else(|_| Style::null());
        Text::new(&format!("{completed}{}{total_str}", self.separator), style)
    }
}

// ---------------------------------------------------------------------------
// DownloadColumn
// ---------------------------------------------------------------------------

/// A column that shows `downloaded/total` as human-readable file sizes.
#[derive(Debug, Clone)]
pub struct DownloadColumn;

impl Default for DownloadColumn {
    fn default() -> Self {
        Self
    }
}

impl ProgressColumn for DownloadColumn {
    fn render(&self, task: &Task) -> Text {
        let completed = filesize::decimal(task.completed as u64, 1, " ");
        let total = match task.total {
            Some(t) => filesize::decimal(t as u64, 1, " "),
            None => "?".to_string(),
        };
        let style = Style::parse("progress.download").unwrap_or_else(|_| Style::null());
        Text::new(&format!("{completed}/{total}"), style)
    }
}

// ---------------------------------------------------------------------------
// TransferSpeedColumn
// ---------------------------------------------------------------------------

/// A column that shows the current transfer speed in human-readable form.
#[derive(Debug, Clone)]
pub struct TransferSpeedColumn;

impl Default for TransferSpeedColumn {
    fn default() -> Self {
        Self
    }
}

impl ProgressColumn for TransferSpeedColumn {
    fn render(&self, task: &Task) -> Text {
        let style = Style::parse("progress.data.speed").unwrap_or_else(|_| Style::null());
        match task.speed() {
            Some(speed) => {
                let formatted = filesize::decimal(speed as u64, 1, " ");
                Text::new(&format!("{formatted}/s"), style)
            }
            None => Text::new("?", style),
        }
    }
}

// ---------------------------------------------------------------------------
// RenderableColumn
// ---------------------------------------------------------------------------

/// A column that renders custom content via a user-supplied callback.
///
/// This allows callers to inject arbitrary rendering logic without
/// defining a new struct that implements [`ProgressColumn`].
///
/// # Examples
///
/// ```
/// use gilt::progress::{ProgressColumn, RenderableColumn, Task};
/// use gilt::text::Text;
/// use gilt::style::Style;
///
/// let col = RenderableColumn::new(|task: &Task| {
///     Text::new(&format!("Step {}", task.completed as u64), Style::null())
/// });
/// let task = Task::new(0, "demo", Some(10.0));
/// assert_eq!(col.render(&task).plain(), "Step 0");
/// ```
pub struct RenderableColumn {
    /// Callback that produces a [`Text`] renderable from a [`Task`].
    pub callback: Box<dyn Fn(&Task) -> Text + Send + Sync>,
}

impl RenderableColumn {
    /// Create a new RenderableColumn with the given rendering callback.
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(&Task) -> Text + Send + Sync + 'static,
    {
        RenderableColumn {
            callback: Box::new(callback),
        }
    }
}

impl ProgressColumn for RenderableColumn {
    fn render(&self, task: &Task) -> Text {
        (self.callback)(task)
    }
}

// ---------------------------------------------------------------------------
// Progress
// ---------------------------------------------------------------------------

/// The main progress tracking orchestrator.
///
/// Manages a collection of [`Task`]s, renders them through configurable
/// [`ProgressColumn`]s, and displays the result via a [`Live`] display.
///
/// # Examples
///
/// ```no_run
/// use gilt::progress::{Progress, BarColumn, TextColumn, TaskProgressColumn, TimeRemainingColumn};
///
/// let mut progress = Progress::new(Progress::default_columns());
/// let task_id = progress.add_task("Downloading...", Some(100.0));
/// progress.start();
/// for i in 0..100 {
///     progress.advance(task_id, 1.0);
/// }
/// progress.stop();
/// ```
pub struct Progress {
    /// Columns to render for each task.
    columns: Vec<Box<dyn ProgressColumn>>,
    /// All tracked tasks.
    tasks: Vec<Task>,
    /// Live display for rendering.
    live: Live,
    /// Counter for generating unique task IDs.
    task_id_counter: usize,
    /// Duration in seconds for the speed estimation sliding window.
    speed_estimate_period: f64,
    /// Function to get the current time (injectable for testing).
    get_time: Box<dyn Fn() -> f64 + Send>,
    /// Whether rendering is disabled.
    disable: bool,
    /// Whether the table should expand to fill available width.
    expand: bool,
}

impl Progress {
    /// Create a new Progress with the given columns.
    pub fn new(columns: Vec<Box<dyn ProgressColumn>>) -> Self {
        Progress {
            columns,
            tasks: Vec::new(),
            live: Live::new(Text::empty())
                .with_auto_refresh(true)
                .with_refresh_per_second(10.0),
            task_id_counter: 0,
            speed_estimate_period: 30.0,
            get_time: Box::new(current_time_secs),
            disable: false,
            expand: false,
        }
    }

    /// Return the default set of columns:
    /// TextColumn (description), BarColumn, TaskProgressColumn, TimeRemainingColumn.
    pub fn default_columns() -> Vec<Box<dyn ProgressColumn>> {
        vec![
            Box::new(TextColumn::new("{task.description}")),
            Box::new(BarColumn::default()),
            Box::new(TaskProgressColumn::default()),
            Box::new(TimeRemainingColumn::default()),
        ]
    }

    // -- Builder methods ----------------------------------------------------

    /// Set the console for the live display (builder pattern).
    #[must_use]
    pub fn with_console(mut self, console: Console) -> Self {
        self.live = self.live.with_console(console);
        self
    }

    /// Enable or disable auto-refresh (builder pattern).
    #[must_use]
    pub fn with_auto_refresh(mut self, auto_refresh: bool) -> Self {
        self.live = self.live.with_auto_refresh(auto_refresh);
        self
    }

    /// Enable or disable transient mode (builder pattern).
    #[must_use]
    pub fn with_transient(mut self, transient: bool) -> Self {
        self.live = self.live.with_transient(transient);
        self
    }

    /// Set the refresh rate in refreshes per second (builder pattern).
    #[must_use]
    pub fn with_refresh_per_second(mut self, rate: f64) -> Self {
        self.live = self.live.with_refresh_per_second(rate);
        self
    }

    /// Set the speed estimation period in seconds (builder pattern).
    #[must_use]
    pub fn with_speed_estimate_period(mut self, seconds: f64) -> Self {
        self.speed_estimate_period = seconds;
        self
    }

    /// Enable or disable progress display (builder pattern).
    #[must_use]
    pub fn with_disable(mut self, disable: bool) -> Self {
        self.disable = disable;
        self
    }

    /// Enable or disable table expansion (builder pattern).
    #[must_use]
    pub fn with_expand(mut self, expand: bool) -> Self {
        self.expand = expand;
        self
    }

    /// Set a custom time function for testing (builder pattern).
    #[must_use]
    pub fn with_get_time<F>(mut self, f: F) -> Self
    where
        F: Fn() -> f64 + Send + 'static,
    {
        self.get_time = Box::new(f);
        self
    }

    // -- Task management ----------------------------------------------------

    /// Add a new task and return its ID.
    ///
    /// The task is created with `completed = 0.0` and is automatically
    /// started (start_time is set).
    pub fn add_task(&mut self, description: &str, total: Option<f64>) -> TaskId {
        let id = self.task_id_counter;
        self.task_id_counter += 1;
        let mut task = Task::new(id, description, total);
        let now = (self.get_time)();
        task.start_time = Some(now);
        self.tasks.push(task);
        id
    }

    /// Update a task with new values.
    ///
    /// Any parameter set to `None` is left unchanged. Use `advance` to
    /// set a relative increment instead of an absolute `completed` value.
    pub fn update(
        &mut self,
        task_id: TaskId,
        completed: Option<f64>,
        total: Option<f64>,
        advance: Option<f64>,
        description: Option<&str>,
        visible: Option<bool>,
    ) {
        let now = (self.get_time)();
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            if let Some(desc) = description {
                task.description = desc.to_string();
            }
            if let Some(t) = total {
                task.total = Some(t);
            }
            if let Some(c) = completed {
                task.completed = c;
            }
            if let Some(a) = advance {
                task.completed += a;
            }
            if let Some(v) = visible {
                task.visible = v;
            }

            // Record a sample for speed estimation.
            if task.started() && !task.finished() {
                task.record_sample(now, self.speed_estimate_period);
            }

            // Check if task just finished.
            if let Some(t) = task.total {
                if task.completed >= t && task.finished_time.is_none() {
                    task.finished_speed = task.speed();
                    task.finished_time = Some(now);
                }
            }
        }
    }

    /// Advance a task's completed count by the given amount.
    pub fn advance(&mut self, task_id: TaskId, advance: f64) {
        self.update(task_id, None, None, Some(advance), None, None);
    }

    /// Mark a task as started (set start_time to now).
    pub fn start_task(&mut self, task_id: TaskId) {
        let now = (self.get_time)();
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            if task.start_time.is_none() {
                task.start_time = Some(now);
            }
        }
    }

    /// Mark a task as stopped (set stop_time to now).
    pub fn stop_task(&mut self, task_id: TaskId) {
        let now = (self.get_time)();
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.stop_time = Some(now);
        }
    }

    /// Remove a task from tracking entirely.
    pub fn remove_task(&mut self, task_id: TaskId) {
        self.tasks.retain(|t| t.id != task_id);
    }

    /// Get a reference to a task by ID.
    pub fn get_task(&self, task_id: TaskId) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == task_id)
    }

    /// Get a mutable reference to a task by ID.
    pub fn get_task_mut(&mut self, task_id: TaskId) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == task_id)
    }

    /// Return a slice of all tasks.
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Return the number of finished tasks.
    pub fn finished_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.finished()).count()
    }

    /// Return the number of visible tasks.
    pub fn visible_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.visible).count()
    }

    // -- Task reset & query -------------------------------------------------

    /// Reset a task's progress to zero.
    ///
    /// Restarts timing from now. The task's total and description remain
    /// unchanged.
    pub fn reset(&mut self, task_id: TaskId) {
        let now = (self.get_time)();
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.completed = 0.0;
            task.start_time = Some(now);
            task.stop_time = None;
            task.finished_time = None;
            task.finished_speed = None;
            task.samples.clear();
        }
    }

    /// Returns true if all visible tasks are finished.
    ///
    /// An empty task list (no visible tasks) returns `true`.
    pub fn all_tasks_finished(&self) -> bool {
        self.tasks
            .iter()
            .filter(|t| t.visible)
            .all(|t| t.finished())
    }

    // -- Console convenience ------------------------------------------------

    /// Print a renderable to the underlying console.
    pub fn print(&self, renderable: &dyn Renderable) {
        self.live.console_mut().print(renderable);
    }

    /// Log a message to the underlying console.
    pub fn log(&self, message: &str) {
        self.live.console_mut().log(message);
    }

    // -- Iterator tracking --------------------------------------------------

    /// Wrap an iterator with automatic progress tracking.
    ///
    /// Creates a task with the given description and optional total,
    /// then returns a [`ProgressTracker`] iterator that advances the
    /// task by 1.0 on each call to `next()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::progress::Progress;
    ///
    /// let mut progress = Progress::new(Progress::default_columns())
    ///     .with_disable(true);
    /// let items: Vec<i32> = progress.track(0..5, "Counting", Some(5.0)).collect();
    /// assert_eq!(items, vec![0, 1, 2, 3, 4]);
    /// ```
    pub fn track<I>(
        &mut self,
        iter: I,
        description: &str,
        total: Option<f64>,
    ) -> ProgressTracker<'_, I::IntoIter>
    where
        I: IntoIterator,
    {
        let task_id = self.add_task(description, total);
        ProgressTracker {
            inner: iter.into_iter(),
            progress: self,
            task_id,
        }
    }

    // -- Display lifecycle --------------------------------------------------

    /// Start the live display.
    pub fn start(&mut self) {
        if self.disable {
            return;
        }
        self.live.start();
    }

    /// Stop the live display.
    pub fn stop(&mut self) {
        if self.disable {
            return;
        }
        self.live.stop();
    }

    /// Refresh the live display with current task state.
    pub fn refresh(&mut self) {
        if self.disable {
            return;
        }
        let table_text = self.render_tasks_text();
        self.live.update_renderable(table_text, true);
    }

    // -- Rendering ----------------------------------------------------------

    /// Build a text representation of the progress table.
    ///
    /// This renders each visible task through the configured columns,
    /// producing a multi-line text output. The table has one row per
    /// visible task and one table-column per configured ProgressColumn.
    pub fn make_tasks_table(&self) -> Table {
        let headers: Vec<&str> = self.columns.iter().map(|_| "").collect();
        let mut table = Table::grid(&headers);
        table.padding = (0, 1, 0, 0);

        if self.expand {
            table.set_expand(true);
        }

        // Ensure all columns have no_wrap set.
        for col in &mut table.columns {
            col.no_wrap = true;
        }

        // Add a row for each visible task.
        for task in &self.tasks {
            if !task.visible {
                continue;
            }
            let cells: Vec<String> = self
                .columns
                .iter()
                .map(|col| {
                    let text = col.render(task);
                    text.plain().to_string()
                })
                .collect();
            let cell_refs: Vec<&str> = cells.iter().map(|s| s.as_str()).collect();
            table.add_row(&cell_refs);
        }

        table
    }

    /// Render the tasks table as a single Text for the live display.
    ///
    /// Preserves styled spans from each column render (bar colors, etc.).
    fn render_tasks_text(&self) -> Text {
        let visible_tasks: Vec<&Task> = self.tasks.iter().filter(|t| t.visible).collect();
        if visible_tasks.is_empty() {
            return Text::empty();
        }

        let separator = Text::new(" ", Style::null());
        let mut result = Text::empty();

        for (i, task) in visible_tasks.iter().enumerate() {
            if i > 0 {
                result.append_str("\n", None);
            }
            for (j, col) in self.columns.iter().enumerate() {
                if j > 0 {
                    result.append_text(&separator);
                }
                let rendered = col.render(task);
                result.append_text(&rendered);
            }
        }

        result
    }
}

impl Renderable for Progress {
    fn rich_console(&self, _console: &Console, _options: &ConsoleOptions) -> Vec<Segment> {
        let text = self.render_tasks_text();
        text.render()
    }
}

// ---------------------------------------------------------------------------
// ProgressTracker
// ---------------------------------------------------------------------------

/// An iterator wrapper that advances a task within a borrowed [`Progress`]
/// on each yielded item.
///
/// Created by [`Progress::track`].
pub struct ProgressTracker<'a, I> {
    inner: I,
    progress: &'a mut Progress,
    task_id: TaskId,
}

impl<'a, I> ProgressTracker<'a, I> {
    /// Return the task ID associated with this tracker.
    pub fn task_id(&self) -> TaskId {
        self.task_id
    }
}

impl<I> Iterator for ProgressTracker<'_, I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next()?;
        self.progress.advance(self.task_id, 1.0);
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

// ---------------------------------------------------------------------------
// TrackIterator
// ---------------------------------------------------------------------------

/// An iterator wrapper that updates a Progress display as items are yielded.
///
/// Created by [`track`] or by manually wrapping an iterator.
pub struct TrackIterator<I> {
    inner: I,
    progress: Progress,
    task_id: TaskId,
    started: bool,
}

impl<I> TrackIterator<I>
where
    I: Iterator,
{
    /// Create a new TrackIterator wrapping the given iterator.
    pub fn new(iter: I, description: &str, total: Option<f64>) -> Self {
        let mut progress = Progress::new(Progress::default_columns()).with_auto_refresh(false);
        let task_id = progress.add_task(description, total);
        TrackIterator {
            inner: iter,
            progress,
            task_id,
            started: false,
        }
    }
}

impl<I> Iterator for TrackIterator<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.started {
            self.progress.start();
            self.started = true;
        }

        match self.inner.next() {
            Some(item) => {
                self.progress.advance(self.task_id, 1.0);
                self.progress.refresh();
                Some(item)
            }
            None => {
                self.progress.stop();
                None
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I> Drop for TrackIterator<I> {
    fn drop(&mut self) {
        if self.started {
            self.progress.stop();
        }
    }
}

/// Convenience function to wrap an iterator with a progress display.
///
/// # Examples
///
/// ```no_run
/// use gilt::progress::track;
///
/// for item in track(0..100, "Processing", Some(100.0)) {
///     // work with item
/// }
/// ```
pub fn track<I>(iter: I, description: &str, total: Option<f64>) -> TrackIterator<I::IntoIter>
where
    I: IntoIterator,
{
    TrackIterator::new(iter.into_iter(), description, total)
}

// ---------------------------------------------------------------------------
// ProgressIteratorExt -- `.progress()` adapter for any iterator
// ---------------------------------------------------------------------------

/// Extension trait that adds [`.progress()`](ProgressIteratorExt::progress)
/// to any iterator, wrapping it with a live progress bar.
///
/// The progress bar total is inferred from
/// [`size_hint()`](Iterator::size_hint) when an upper bound is available
/// (e.g. `Vec::iter()`, `Range`). For iterators without a known length the
/// bar runs in indeterminate mode.
///
/// # Examples
///
/// ```no_run
/// use gilt::progress::ProgressIteratorExt;
///
/// // Range -- total inferred from size_hint
/// for i in (0..100).progress("Counting") {
///     // work
/// }
///
/// // Vec -- total inferred from size_hint
/// let items = vec![1, 2, 3, 4, 5];
/// for item in items.iter().progress("Loading") {
///     // work
/// }
/// ```
pub trait ProgressIteratorExt: Iterator + Sized {
    /// Wrap this iterator with a progress bar.
    ///
    /// The progress bar total is inferred from `size_hint()` if an upper
    /// bound is available; otherwise the bar is indeterminate.
    fn progress(self, description: &str) -> ProgressIter<Self>;

    /// Wrap this iterator with a progress bar, explicitly setting the total.
    fn progress_with_total(self, description: &str, total: f64) -> ProgressIter<Self>;
}

impl<I: Iterator> ProgressIteratorExt for I {
    fn progress(self, description: &str) -> ProgressIter<Self> {
        let total = self.size_hint().1.map(|n| n as f64);
        ProgressIter::new(self, description, total)
    }

    fn progress_with_total(self, description: &str, total: f64) -> ProgressIter<Self> {
        ProgressIter::new(self, description, Some(total))
    }
}

/// An iterator adapter that displays a live progress bar while yielding
/// items from an inner iterator.
///
/// Created by [`ProgressIteratorExt::progress`]. Owns its own [`Progress`]
/// display; the progress bar starts on the first call to `next()` and stops
/// automatically when the iterator is exhausted or dropped.
pub struct ProgressIter<I> {
    inner: I,
    progress: Progress,
    task_id: TaskId,
    started: bool,
}

impl<I: Iterator> ProgressIter<I> {
    /// Create a new `ProgressIter` wrapping the given iterator.
    fn new(iter: I, description: &str, total: Option<f64>) -> Self {
        let mut progress = Progress::new(Progress::default_columns()).with_auto_refresh(true);
        let task_id = progress.add_task(description, total);
        ProgressIter {
            inner: iter,
            progress,
            task_id,
            started: false,
        }
    }

    /// Return the [`TaskId`] for the underlying progress task.
    pub fn task_id(&self) -> TaskId {
        self.task_id
    }
}

impl<I: Iterator> Iterator for ProgressIter<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.started {
            self.progress.start();
            self.started = true;
        }

        match self.inner.next() {
            Some(item) => {
                self.progress.advance(self.task_id, 1.0);
                self.progress.refresh();
                Some(item)
            }
            None => {
                self.progress.stop();
                None
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I> Drop for ProgressIter<I> {
    fn drop(&mut self) {
        if self.started {
            self.progress.stop();
        }
    }
}
// ---------------------------------------------------------------------------
// ProgressReader
// ---------------------------------------------------------------------------

/// A reader wrapper that calls a callback on each read for progress tracking.
///
/// This wraps any [`Read`] implementor and invokes a user-supplied callback
/// with the number of bytes read on each call to [`read`](Read::read). The
/// callback is typically a closure that calls [`Progress::advance`].
///
/// # Examples
///
/// ```
/// use std::io::Read;
/// use std::sync::atomic::{AtomicUsize, Ordering};
/// use std::sync::Arc;
/// use gilt::progress::ProgressReader;
///
/// let data = vec![0u8; 1024];
/// let bytes_seen = Arc::new(AtomicUsize::new(0));
/// let counter = bytes_seen.clone();
/// let mut reader = ProgressReader::new(
///     data.as_slice(),
///     move |n| { counter.fetch_add(n, Ordering::Relaxed); },
/// );
/// let mut buf = vec![0u8; 256];
/// reader.read(&mut buf).unwrap();
/// assert_eq!(bytes_seen.load(Ordering::Relaxed), 256);
/// ```
pub struct ProgressReader<R> {
    inner: R,
    callback: Box<dyn FnMut(usize)>,
    total_read: usize,
}

impl<R> ProgressReader<R> {
    /// Wrap a reader with a progress callback.
    ///
    /// The `callback` is invoked after every successful read with the
    /// number of bytes that were read.
    pub fn new(inner: R, callback: impl FnMut(usize) + 'static) -> Self {
        ProgressReader {
            inner,
            callback: Box::new(callback),
            total_read: 0,
        }
    }

    /// Total bytes read so far through this wrapper.
    pub fn total_read(&self) -> usize {
        self.total_read
    }

    /// Consume the wrapper and return the inner reader.
    pub fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: Read> Read for ProgressReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.total_read += n;
        (self.callback)(n);
        Ok(n)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Helper: test console -----------------------------------------------

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

    // =======================================================================
    // ProgressSample tests
    // =======================================================================

    #[test]
    fn test_progress_sample_creation() {
        let sample = ProgressSample {
            timestamp: 1.0,
            completed: 50.0,
        };
        assert!((sample.timestamp - 1.0).abs() < f64::EPSILON);
        assert!((sample.completed - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_sample_clone() {
        let sample = ProgressSample {
            timestamp: 2.5,
            completed: 75.0,
        };
        let cloned = sample.clone();
        assert!((cloned.timestamp - 2.5).abs() < f64::EPSILON);
        assert!((cloned.completed - 75.0).abs() < f64::EPSILON);
    }

    // =======================================================================
    // Task tests
    // =======================================================================

    #[test]
    fn test_task_creation() {
        let task = Task::new(0, "Test task", Some(100.0));
        assert_eq!(task.id, 0);
        assert_eq!(task.description, "Test task");
        assert_eq!(task.total, Some(100.0));
        assert!((task.completed - 0.0).abs() < f64::EPSILON);
        assert!(task.visible);
        assert!(task.fields.is_empty());
        assert!(task.start_time.is_none());
        assert!(task.stop_time.is_none());
        assert!(task.finished_time.is_none());
        assert!(task.finished_speed.is_none());
    }

    #[test]
    fn test_task_creation_no_total() {
        let task = Task::new(1, "Indeterminate", None);
        assert_eq!(task.total, None);
    }

    #[test]
    fn test_task_started() {
        let mut task = Task::new(0, "test", Some(100.0));
        assert!(!task.started());
        task.start_time = Some(1.0);
        assert!(task.started());
    }

    #[test]
    fn test_task_finished() {
        let mut task = Task::new(0, "test", Some(100.0));
        assert!(!task.finished());
        task.finished_time = Some(2.0);
        assert!(task.finished());
    }

    #[test]
    fn test_task_remaining_with_total() {
        let mut task = Task::new(0, "test", Some(100.0));
        task.completed = 30.0;
        assert_eq!(task.remaining(), Some(70.0));
    }

    #[test]
    fn test_task_remaining_over_total() {
        let mut task = Task::new(0, "test", Some(100.0));
        task.completed = 120.0;
        assert_eq!(task.remaining(), Some(0.0));
    }

    #[test]
    fn test_task_remaining_no_total() {
        let task = Task::new(0, "test", None);
        assert_eq!(task.remaining(), None);
    }

    #[test]
    fn test_task_elapsed_not_started() {
        let task = Task::new(0, "test", Some(100.0));
        assert_eq!(task.elapsed(), None);
    }

    #[test]
    fn test_task_elapsed_with_stop() {
        let mut task = Task::new(0, "test", Some(100.0));
        task.start_time = Some(10.0);
        task.stop_time = Some(15.0);
        let elapsed = task.elapsed().unwrap();
        assert!((elapsed - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_task_percentage_normal() {
        let mut task = Task::new(0, "test", Some(100.0));
        task.completed = 50.0;
        assert!((task.percentage() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_task_percentage_zero_completed() {
        let task = Task::new(0, "test", Some(100.0));
        assert!((task.percentage() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_task_percentage_full() {
        let mut task = Task::new(0, "test", Some(100.0));
        task.completed = 100.0;
        assert!((task.percentage() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_task_percentage_over() {
        let mut task = Task::new(0, "test", Some(100.0));
        task.completed = 150.0;
        assert!((task.percentage() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_task_percentage_no_total() {
        let task = Task::new(0, "test", None);
        assert!((task.percentage() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_task_percentage_zero_total() {
        let task = Task::new(0, "test", Some(0.0));
        assert!((task.percentage() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_task_percentage_custom_total() {
        let mut task = Task::new(0, "test", Some(200.0));
        task.completed = 100.0;
        assert!((task.percentage() - 50.0).abs() < f64::EPSILON);
    }

    // -- Task speed tests ---------------------------------------------------

    #[test]
    fn test_task_speed_no_samples() {
        let task = Task::new(0, "test", Some(100.0));
        assert!(task.speed().is_none());
    }

    #[test]
    fn test_task_speed_one_sample() {
        let mut task = Task::new(0, "test", Some(100.0));
        task.start_time = Some(0.0);
        task.completed = 10.0;
        task.record_sample(1.0, 30.0);
        assert!(task.speed().is_none()); // Need at least 2 samples
    }

    #[test]
    fn test_task_speed_two_samples() {
        let mut task = Task::new(0, "test", Some(100.0));
        task.start_time = Some(0.0);

        task.completed = 10.0;
        task.record_sample(1.0, 30.0);

        task.completed = 30.0;
        task.record_sample(3.0, 30.0);

        let speed = task.speed().unwrap();
        // 20 units in 2 seconds = 10 units/sec
        assert!((speed - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_task_speed_multiple_samples() {
        let mut task = Task::new(0, "test", Some(100.0));
        task.start_time = Some(0.0);

        for i in 0..10 {
            task.completed = (i + 1) as f64 * 5.0;
            task.record_sample((i + 1) as f64, 30.0);
        }

        let speed = task.speed().unwrap();
        // 50 units in 10 seconds = 5 units/sec (from first to last sample)
        assert!((speed - 5.0).abs() < 0.1);
    }

    #[test]
    fn test_task_speed_with_sliding_window() {
        let mut task = Task::new(0, "test", Some(100.0));
        task.start_time = Some(0.0);

        // Record samples with a small window period.
        task.completed = 10.0;
        task.record_sample(1.0, 5.0);

        task.completed = 20.0;
        task.record_sample(2.0, 5.0);

        // These old samples should be pruned (window = 5 seconds).
        task.completed = 60.0;
        task.record_sample(10.0, 5.0);

        task.completed = 70.0;
        task.record_sample(11.0, 5.0);

        let speed = task.speed().unwrap();
        // After pruning, only samples from t=10 and t=11 remain in window:
        // speed = (70 - 60) / (11 - 10) = 10.0
        // But actually, the sample at t=2 might also remain since 11-5=6 > 2.
        // Let's check: cutoff at t=11 - 5 = 6, so t=1 and t=2 are pruned.
        // Remaining: t=10 (60) and t=11 (70).
        assert!((speed - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_task_speed_finished() {
        let mut task = Task::new(0, "test", Some(100.0));
        task.start_time = Some(0.0);

        task.completed = 50.0;
        task.record_sample(1.0, 30.0);
        task.completed = 100.0;
        task.record_sample(2.0, 30.0);

        task.finished_speed = Some(50.0);
        task.finished_time = Some(2.0);

        assert_eq!(task.speed(), Some(50.0));
    }

    // -- Task time_remaining tests ------------------------------------------

    #[test]
    fn test_task_time_remaining_no_speed() {
        let mut task = Task::new(0, "test", Some(100.0));
        task.completed = 50.0;
        assert!(task.time_remaining().is_none());
    }

    #[test]
    fn test_task_time_remaining_no_total() {
        let task = Task::new(0, "test", None);
        assert!(task.time_remaining().is_none());
    }

    #[test]
    fn test_task_time_remaining_with_speed() {
        let mut task = Task::new(0, "test", Some(100.0));
        task.start_time = Some(0.0);

        task.completed = 20.0;
        task.record_sample(1.0, 30.0);
        task.completed = 40.0;
        task.record_sample(2.0, 30.0);

        // speed = 20 units/sec, remaining = 60
        // time_remaining = 60 / 20 = 3.0
        let remaining = task.time_remaining().unwrap();
        assert!((remaining - 3.0).abs() < 0.1);
    }

    #[test]
    fn test_task_time_remaining_finished() {
        let mut task = Task::new(0, "test", Some(100.0));
        task.completed = 100.0;
        task.finished_time = Some(5.0);
        assert_eq!(task.time_remaining(), Some(0.0));
    }

    // -- Task record_sample tests -------------------------------------------

    #[test]
    fn test_task_record_sample() {
        let mut task = Task::new(0, "test", Some(100.0));
        task.completed = 10.0;
        task.record_sample(1.0, 30.0);

        assert_eq!(task.samples.len(), 1);
        assert_eq!(task.progress.len(), 1);
    }

    #[test]
    fn test_task_record_sample_pruning() {
        let mut task = Task::new(0, "test", Some(100.0));

        // Record 5 samples over 50 seconds with a 10-second window.
        for i in 0..5 {
            task.completed = (i + 1) as f64 * 10.0;
            task.record_sample((i + 1) as f64 * 10.0, 10.0);
        }

        // Samples at t=10, t=20, t=30, t=40, t=50
        // With window of 10 seconds and latest at t=50, cutoff is t=40.
        // Only t=40 and t=50 should remain in the sliding window.
        // (t=40 has timestamp 40 >= 40, so it stays)
        assert_eq!(task.samples.len(), 2);
        // progress always keeps all samples
        assert_eq!(task.progress.len(), 5);
    }

    // =======================================================================
    // format_time tests
    // =======================================================================

    #[test]
    fn test_format_time_zero() {
        assert_eq!(format_time(0.0), "0:00");
    }

    #[test]
    fn test_format_time_seconds() {
        assert_eq!(format_time(45.0), "0:45");
    }

    #[test]
    fn test_format_time_minutes() {
        assert_eq!(format_time(125.0), "2:05");
    }

    #[test]
    fn test_format_time_hours() {
        assert_eq!(format_time(3661.0), "1:01:01");
    }

    #[test]
    fn test_format_time_rounding() {
        assert_eq!(format_time(59.6), "1:00");
    }

    // =======================================================================
    // TextColumn tests
    // =======================================================================

    #[test]
    fn test_text_column_description() {
        let col = TextColumn::new("{task.description}");
        let task = Task::new(0, "Downloading", Some(100.0));
        let text = col.render(&task);
        assert_eq!(text.plain(), "Downloading");
    }

    #[test]
    fn test_text_column_percentage() {
        let col = TextColumn::new("{task.percentage:.0f}%");
        let mut task = Task::new(0, "test", Some(100.0));
        task.completed = 50.0;
        let text = col.render(&task);
        assert_eq!(text.plain(), "50%");
    }

    #[test]
    fn test_text_column_percentage_1f() {
        let col = TextColumn::new("{task.percentage:.1f}%");
        let mut task = Task::new(0, "test", Some(100.0));
        task.completed = 33.3;
        let text = col.render(&task);
        assert_eq!(text.plain(), "33.3%");
    }

    #[test]
    fn test_text_column_completed_and_total() {
        let col = TextColumn::new("{task.completed}/{task.total}");
        let mut task = Task::new(0, "test", Some(200.0));
        task.completed = 50.0;
        let text = col.render(&task);
        assert_eq!(text.plain(), "50/200");
    }

    #[test]
    fn test_text_column_no_total() {
        let col = TextColumn::new("{task.total}");
        let task = Task::new(0, "test", None);
        let text = col.render(&task);
        assert_eq!(text.plain(), "?");
    }

    #[test]
    fn test_text_column_speed_unknown() {
        let col = TextColumn::new("{task.speed}");
        let task = Task::new(0, "test", Some(100.0));
        let text = col.render(&task);
        assert_eq!(text.plain(), "?");
    }

    #[test]
    fn test_text_column_fields() {
        let col = TextColumn::new("Status: {task.fields.status}");
        let mut task = Task::new(0, "test", Some(100.0));
        task.fields
            .insert("status".to_string(), "running".to_string());
        let text = col.render(&task);
        assert_eq!(text.plain(), "Status: running");
    }

    #[test]
    fn test_text_column_with_style() {
        let style = Style::parse("bold").unwrap();
        let col = TextColumn::new("test").with_style(style.clone());
        assert_eq!(col.style, Some(style));
    }

    #[test]
    fn test_text_column_with_justify() {
        let col = TextColumn::new("test").with_justify(JustifyMethod::Right);
        assert_eq!(col.justify, JustifyMethod::Right);
    }

    // =======================================================================
    // BarColumn tests
    // =======================================================================

    #[test]
    fn test_bar_column_default() {
        let col = BarColumn::default();
        assert_eq!(col.bar_width, Some(40));
        assert_eq!(col.style, "bar.back");
        assert_eq!(col.complete_style, "bar.complete");
    }

    #[test]
    fn test_bar_column_render() {
        let col = BarColumn::new().with_bar_width(Some(10));
        let mut task = Task::new(0, "test", Some(100.0));
        task.completed = 50.0;
        let text = col.render(&task);
        assert!(!text.plain().is_empty());
    }

    #[test]
    fn test_bar_column_render_no_total() {
        let col = BarColumn::new().with_bar_width(Some(10));
        let task = Task::new(0, "test", None);
        let text = col.render(&task);
        assert!(!text.plain().is_empty());
    }

    #[test]
    fn test_bar_column_render_complete() {
        let col = BarColumn::new().with_bar_width(Some(10));
        let mut task = Task::new(0, "test", Some(100.0));
        task.completed = 100.0;
        let text = col.render(&task);
        assert!(!text.plain().is_empty());
    }

    // =======================================================================
    // SpinnerColumn tests
    // =======================================================================

    #[test]
    fn test_spinner_column_default() {
        let col = SpinnerColumn::default();
        assert_eq!(col.spinner_name, "dots");
    }

    #[test]
    fn test_spinner_column_render() {
        let col = SpinnerColumn::new("dots");
        let mut task = Task::new(0, "test", Some(100.0));
        task.start_time = Some(0.0);
        let text = col.render(&task);
        assert!(!text.plain().is_empty());
    }

    #[test]
    fn test_spinner_column_render_finished() {
        let col = SpinnerColumn::new("dots");
        let mut task = Task::new(0, "test", Some(100.0));
        task.finished_time = Some(1.0);
        let text = col.render(&task);
        // Should show the finished text (checkmark)
        assert!(text.plain().contains('\u{2714}'));
    }

    #[test]
    fn test_spinner_column_max_refresh() {
        let col = SpinnerColumn::default();
        assert!(col.max_refresh().is_some());
    }

    #[test]
    fn test_spinner_column_with_style() {
        let style = Style::parse("bold").unwrap();
        let col = SpinnerColumn::new("dots").with_style(style.clone());
        assert_eq!(col.style, Some(style));
    }

    #[test]
    fn test_spinner_column_with_finished_text() {
        let text = Text::new("DONE", Style::null());
        let col = SpinnerColumn::new("dots").with_finished_text(text.clone());
        assert_eq!(col.finished_text.plain(), "DONE");
    }

    #[test]
    fn test_spinner_column_invalid_name() {
        let col = SpinnerColumn::new("nonexistent_spinner_xyz");
        let task = Task::new(0, "test", Some(100.0));
        let text = col.render(&task);
        assert_eq!(text.plain(), "?");
    }

    // =======================================================================
    // TimeElapsedColumn tests
    // =======================================================================

    #[test]
    fn test_time_elapsed_not_started() {
        let col = TimeElapsedColumn;
        let task = Task::new(0, "test", Some(100.0));
        let text = col.render(&task);
        assert_eq!(text.plain(), "0:00");
    }

    #[test]
    fn test_time_elapsed_with_time() {
        let col = TimeElapsedColumn;
        let mut task = Task::new(0, "test", Some(100.0));
        task.start_time = Some(0.0);
        task.stop_time = Some(65.0);
        let text = col.render(&task);
        assert_eq!(text.plain(), "1:05");
    }

    #[test]
    fn test_time_elapsed_hours() {
        let col = TimeElapsedColumn;
        let mut task = Task::new(0, "test", Some(100.0));
        task.start_time = Some(0.0);
        task.stop_time = Some(3723.0);
        let text = col.render(&task);
        assert_eq!(text.plain(), "1:02:03");
    }

    // =======================================================================
    // TimeRemainingColumn tests
    // =======================================================================

    #[test]
    fn test_time_remaining_unknown() {
        let col = TimeRemainingColumn::default();
        let task = Task::new(0, "test", Some(100.0));
        let text = col.render(&task);
        assert_eq!(text.plain(), "-:--:--");
    }

    #[test]
    fn test_time_remaining_finished() {
        let col = TimeRemainingColumn::default();
        let mut task = Task::new(0, "test", Some(100.0));
        task.finished_time = Some(5.0);
        let text = col.render(&task);
        assert_eq!(text.plain(), "0:00");
    }

    #[test]
    fn test_time_remaining_with_speed() {
        let col = TimeRemainingColumn::default();
        let mut task = Task::new(0, "test", Some(100.0));
        task.start_time = Some(0.0);
        task.completed = 20.0;
        task.record_sample(1.0, 30.0);
        task.completed = 40.0;
        task.record_sample(2.0, 30.0);
        // speed = 20/s, remaining = 60, time = 3s
        let text = col.render(&task);
        assert_eq!(text.plain(), "0:03");
    }

    #[test]
    fn test_time_remaining_elapsed_when_finished() {
        let col = TimeRemainingColumn {
            compact: false,
            elapsed_when_finished: true,
        };
        let mut task = Task::new(0, "test", Some(100.0));
        task.start_time = Some(0.0);
        task.stop_time = Some(120.0);
        task.finished_time = Some(120.0);
        let text = col.render(&task);
        assert_eq!(text.plain(), "2:00");
    }

    // =======================================================================
    // TaskProgressColumn tests
    // =======================================================================

    #[test]
    fn test_task_progress_column() {
        let col = TaskProgressColumn::default();
        let mut task = Task::new(0, "test", Some(100.0));
        task.completed = 42.0;
        let text = col.render(&task);
        assert_eq!(text.plain(), "42/100");
    }

    #[test]
    fn test_task_progress_column_no_total() {
        let col = TaskProgressColumn::default();
        let task = Task::new(0, "test", None);
        let text = col.render(&task);
        assert_eq!(text.plain(), "0/?");
    }

    #[test]
    fn test_task_progress_column_custom_separator() {
        let col = TaskProgressColumn::new().with_separator(" of ");
        let mut task = Task::new(0, "test", Some(50.0));
        task.completed = 25.0;
        let text = col.render(&task);
        assert_eq!(text.plain(), "25 of 50");
    }

    // =======================================================================
    // FileSizeColumn tests
    // =======================================================================

    #[test]
    fn test_file_size_column_bytes() {
        let col = FileSizeColumn;
        let mut task = Task::new(0, "test", Some(1000.0));
        task.completed = 500.0;
        let text = col.render(&task);
        assert_eq!(text.plain(), "500 bytes");
    }

    #[test]
    fn test_file_size_column_kb() {
        let col = FileSizeColumn;
        let mut task = Task::new(0, "test", Some(100000.0));
        task.completed = 5000.0;
        let text = col.render(&task);
        assert_eq!(text.plain(), "5.0 kB");
    }

    #[test]
    fn test_file_size_column_mb() {
        let col = FileSizeColumn;
        let mut task = Task::new(0, "test", Some(100000000.0));
        task.completed = 5000000.0;
        let text = col.render(&task);
        assert_eq!(text.plain(), "5.0 MB");
    }

    // =======================================================================
    // TotalFileSizeColumn tests
    // =======================================================================

    #[test]
    fn test_total_file_size_column() {
        let col = TotalFileSizeColumn;
        let task = Task::new(0, "test", Some(5000.0));
        let text = col.render(&task);
        assert_eq!(text.plain(), "5.0 kB");
    }

    #[test]
    fn test_total_file_size_column_no_total() {
        let col = TotalFileSizeColumn;
        let task = Task::new(0, "test", None);
        let text = col.render(&task);
        assert_eq!(text.plain(), "0 bytes");
    }

    // =======================================================================
    // MofNCompleteColumn tests
    // =======================================================================

    #[test]
    fn test_mofn_column() {
        let col = MofNCompleteColumn::default();
        let mut task = Task::new(0, "test", Some(50.0));
        task.completed = 25.0;
        let text = col.render(&task);
        assert_eq!(text.plain(), "25/50");
    }

    #[test]
    fn test_mofn_column_no_total() {
        let col = MofNCompleteColumn::default();
        let task = Task::new(0, "test", None);
        let text = col.render(&task);
        assert_eq!(text.plain(), "0/?");
    }

    #[test]
    fn test_mofn_column_custom_separator() {
        let col = MofNCompleteColumn::new().with_separator(" / ");
        let mut task = Task::new(0, "test", Some(100.0));
        task.completed = 75.0;
        let text = col.render(&task);
        assert_eq!(text.plain(), "75 / 100");
    }

    // =======================================================================
    // DownloadColumn tests
    // =======================================================================

    #[test]
    fn test_download_column() {
        let col = DownloadColumn;
        let mut task = Task::new(0, "test", Some(1000000.0));
        task.completed = 500000.0;
        let text = col.render(&task);
        assert_eq!(text.plain(), "500.0 kB/1.0 MB");
    }

    #[test]
    fn test_download_column_no_total() {
        let col = DownloadColumn;
        let mut task = Task::new(0, "test", None);
        task.completed = 1000.0;
        let text = col.render(&task);
        assert_eq!(text.plain(), "1.0 kB/?");
    }

    // =======================================================================
    // TransferSpeedColumn tests
    // =======================================================================

    #[test]
    fn test_transfer_speed_column_no_speed() {
        let col = TransferSpeedColumn;
        let task = Task::new(0, "test", Some(100.0));
        let text = col.render(&task);
        assert_eq!(text.plain(), "?");
    }

    #[test]
    fn test_transfer_speed_column_with_speed() {
        let col = TransferSpeedColumn;
        let mut task = Task::new(0, "test", Some(100000.0));
        task.start_time = Some(0.0);
        task.completed = 1000.0;
        task.record_sample(1.0, 30.0);
        task.completed = 11000.0;
        task.record_sample(2.0, 30.0);
        // speed = 10000 bytes/sec
        let text = col.render(&task);
        assert_eq!(text.plain(), "10.0 kB/s");
    }

    // =======================================================================
    // Progress tests
    // =======================================================================

    #[test]
    fn test_progress_creation() {
        let progress = Progress::new(Progress::default_columns());
        assert!(progress.tasks.is_empty());
        assert_eq!(progress.task_id_counter, 0);
    }

    #[test]
    fn test_progress_default_columns() {
        let cols = Progress::default_columns();
        assert_eq!(cols.len(), 4);
    }

    #[test]
    fn test_progress_add_task() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        let id = progress.add_task("Test", Some(100.0));
        assert_eq!(id, 0);
        assert_eq!(progress.tasks.len(), 1);
        assert_eq!(progress.tasks[0].description, "Test");
        assert_eq!(progress.tasks[0].total, Some(100.0));
        assert!(progress.tasks[0].started());
    }

    #[test]
    fn test_progress_add_multiple_tasks() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        let id1 = progress.add_task("First", Some(100.0));
        let id2 = progress.add_task("Second", Some(200.0));
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(progress.tasks.len(), 2);
    }

    #[test]
    fn test_progress_update_completed() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        let id = progress.add_task("Test", Some(100.0));
        progress.update(id, Some(50.0), None, None, None, None);
        assert!((progress.tasks[0].completed - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_update_total() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        let id = progress.add_task("Test", Some(100.0));
        progress.update(id, None, Some(200.0), None, None, None);
        assert_eq!(progress.tasks[0].total, Some(200.0));
    }

    #[test]
    fn test_progress_update_advance() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        let id = progress.add_task("Test", Some(100.0));
        progress.update(id, None, None, Some(10.0), None, None);
        assert!((progress.tasks[0].completed - 10.0).abs() < f64::EPSILON);
        progress.update(id, None, None, Some(15.0), None, None);
        assert!((progress.tasks[0].completed - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_update_description() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        let id = progress.add_task("Old", Some(100.0));
        progress.update(id, None, None, None, Some("New"), None);
        assert_eq!(progress.tasks[0].description, "New");
    }

    #[test]
    fn test_progress_update_visible() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        let id = progress.add_task("Test", Some(100.0));
        assert!(progress.tasks[0].visible);
        progress.update(id, None, None, None, None, Some(false));
        assert!(!progress.tasks[0].visible);
    }

    #[test]
    fn test_progress_advance() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        let id = progress.add_task("Test", Some(100.0));
        progress.advance(id, 30.0);
        assert!((progress.tasks[0].completed - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_start_task() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        let id = progress.add_task("Test", Some(100.0));
        // add_task already starts the task
        assert!(progress.tasks[0].started());
        // start_task should be a no-op if already started
        progress.start_task(id);
        assert!(progress.tasks[0].started());
    }

    #[test]
    fn test_progress_stop_task() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        let id = progress.add_task("Test", Some(100.0));
        progress.stop_task(id);
        assert!(progress.tasks[0].stop_time.is_some());
    }

    #[test]
    fn test_progress_remove_task() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        let id = progress.add_task("Test", Some(100.0));
        assert_eq!(progress.tasks.len(), 1);
        progress.remove_task(id);
        assert_eq!(progress.tasks.len(), 0);
    }

    #[test]
    fn test_progress_remove_nonexistent_task() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        progress.add_task("Test", Some(100.0));
        progress.remove_task(999);
        assert_eq!(progress.tasks.len(), 1);
    }

    #[test]
    fn test_progress_get_task() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        let id = progress.add_task("Test", Some(100.0));
        let task = progress.get_task(id).unwrap();
        assert_eq!(task.description, "Test");
    }

    #[test]
    fn test_progress_get_task_nonexistent() {
        let progress = Progress::new(Progress::default_columns());
        assert!(progress.get_task(999).is_none());
    }

    #[test]
    fn test_progress_get_task_mut() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        let id = progress.add_task("Test", Some(100.0));
        {
            let task = progress.get_task_mut(id).unwrap();
            task.description = "Modified".to_string();
        }
        assert_eq!(progress.tasks[0].description, "Modified");
    }

    #[test]
    fn test_progress_finished_count() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        let id1 = progress.add_task("First", Some(100.0));
        let _id2 = progress.add_task("Second", Some(100.0));
        assert_eq!(progress.finished_count(), 0);
        progress.update(id1, Some(100.0), None, None, None, None);
        assert_eq!(progress.finished_count(), 1);
    }

    #[test]
    fn test_progress_visible_count() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        let id1 = progress.add_task("First", Some(100.0));
        let _id2 = progress.add_task("Second", Some(100.0));
        assert_eq!(progress.visible_count(), 2);
        progress.update(id1, None, None, None, None, Some(false));
        assert_eq!(progress.visible_count(), 1);
    }

    // -- Progress auto-finish -----------------------------------------------

    #[test]
    fn test_progress_auto_finish() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        let id = progress.add_task("Test", Some(100.0));
        progress.update(id, Some(100.0), None, None, None, None);
        assert!(progress.tasks[0].finished());
        assert!(progress.tasks[0].finished_time.is_some());
    }

    #[test]
    fn test_progress_auto_finish_over_total() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        let id = progress.add_task("Test", Some(100.0));
        progress.update(id, Some(150.0), None, None, None, None);
        assert!(progress.tasks[0].finished());
    }

    // -- Progress make_tasks_table ------------------------------------------

    #[test]
    fn test_progress_make_tasks_table_empty() {
        let progress = Progress::new(Progress::default_columns());
        let table = progress.make_tasks_table();
        assert_eq!(table.rows.len(), 0);
    }

    #[test]
    fn test_progress_make_tasks_table_with_tasks() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        progress.add_task("First", Some(100.0));
        progress.add_task("Second", Some(200.0));
        let table = progress.make_tasks_table();
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.columns.len(), 4); // 4 default columns
    }

    #[test]
    fn test_progress_make_tasks_table_hidden_tasks() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        let _id1 = progress.add_task("Visible", Some(100.0));
        let id2 = progress.add_task("Hidden", Some(100.0));
        progress.update(id2, None, None, None, None, Some(false));
        let table = progress.make_tasks_table();
        assert_eq!(table.rows.len(), 1);
    }

    // -- Progress Renderable ------------------------------------------------

    #[test]
    fn test_progress_renderable() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        progress.add_task("Test", Some(100.0));
        let console = test_console();
        let opts = console.options();
        let segments = progress.rich_console(&console, &opts);
        assert!(!segments.is_empty());
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("Test"));
    }

    #[test]
    fn test_progress_renderable_empty() {
        let progress = Progress::new(Progress::default_columns());
        let console = test_console();
        let opts = console.options();
        let segments = progress.rich_console(&console, &opts);
        // Empty progress should produce minimal output
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.is_empty() || text.trim().is_empty());
    }

    // -- Progress builder methods -------------------------------------------

    #[test]
    fn test_progress_with_speed_estimate_period() {
        let progress = Progress::new(Progress::default_columns()).with_speed_estimate_period(60.0);
        assert!((progress.speed_estimate_period - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_with_disable() {
        let progress = Progress::new(Progress::default_columns()).with_disable(true);
        assert!(progress.disable);
    }

    #[test]
    fn test_progress_with_expand() {
        let progress = Progress::new(Progress::default_columns()).with_expand(true);
        assert!(progress.expand);
    }

    #[test]
    fn test_progress_with_get_time() {
        let progress = Progress::new(Progress::default_columns()).with_get_time(|| 42.0);
        assert!(((progress.get_time)() - 42.0).abs() < f64::EPSILON);
    }

    // -- Progress lifecycle -------------------------------------------------

    #[test]
    fn test_progress_start_stop() {
        let mut progress = Progress::new(Progress::default_columns())
            .with_console(test_console())
            .with_auto_refresh(false);

        progress.start();
        progress.stop();
    }

    #[test]
    fn test_progress_start_stop_disabled() {
        let mut progress = Progress::new(Progress::default_columns()).with_disable(true);

        // Should not panic even when disabled
        progress.start();
        progress.stop();
    }

    #[test]
    fn test_progress_refresh() {
        let mut progress = Progress::new(Progress::default_columns())
            .with_console(test_console())
            .with_auto_refresh(false)
            .with_get_time(|| 1.0);

        progress.add_task("Test", Some(100.0));
        progress.start();
        progress.refresh();
        progress.stop();
    }

    #[test]
    fn test_progress_refresh_disabled() {
        let mut progress = Progress::new(Progress::default_columns()).with_disable(true);

        // Should not panic
        progress.refresh();
    }

    // -- Multiple tasks tests -----------------------------------------------

    #[test]
    fn test_progress_multiple_tasks() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);

        let id1 = progress.add_task("Download", Some(1000.0));
        let id2 = progress.add_task("Process", Some(500.0));
        let id3 = progress.add_task("Upload", Some(2000.0));

        progress.advance(id1, 500.0);
        progress.advance(id2, 250.0);
        progress.advance(id3, 100.0);

        assert!((progress.tasks[0].completed - 500.0).abs() < f64::EPSILON);
        assert!((progress.tasks[1].completed - 250.0).abs() < f64::EPSILON);
        assert!((progress.tasks[2].completed - 100.0).abs() < f64::EPSILON);
    }

    // -- Task with no total (indeterminate) ---------------------------------

    #[test]
    fn test_progress_indeterminate_task() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);

        let id = progress.add_task("Loading...", None);
        progress.advance(id, 10.0);
        assert!((progress.tasks[0].completed - 10.0).abs() < f64::EPSILON);
        assert_eq!(progress.tasks[0].total, None);
        assert!((progress.tasks[0].percentage() - 0.0).abs() < f64::EPSILON);
        assert!(!progress.tasks[0].finished());
    }

    // -- TrackIterator tests ------------------------------------------------

    #[test]
    fn test_track_iterator() {
        let items: Vec<i32> = track(0..5, "Counting", Some(5.0)).collect();
        assert_eq!(items, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_track_iterator_empty() {
        let items: Vec<i32> = track(std::iter::empty::<i32>(), "Empty", Some(0.0)).collect();
        assert!(items.is_empty());
    }

    #[test]
    fn test_track_iterator_count() {
        let count = track(0..10, "Test", Some(10.0)).count();
        assert_eq!(count, 10);
    }

    // -- Tasks slice accessor -----------------------------------------------

    #[test]
    fn test_progress_tasks_accessor() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        progress.add_task("A", Some(100.0));
        progress.add_task("B", Some(200.0));

        let tasks = progress.tasks();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].description, "A");
        assert_eq!(tasks[1].description, "B");
    }

    // -- Edge cases ---------------------------------------------------------

    #[test]
    fn test_progress_update_nonexistent_task() {
        let mut progress = Progress::new(Progress::default_columns()).with_get_time(|| 1.0);
        // Should not panic
        progress.update(999, Some(50.0), None, None, None, None);
        progress.advance(999, 10.0);
        progress.start_task(999);
        progress.stop_task(999);
    }

    #[test]
    fn test_task_fields_in_render() {
        let col = TextColumn::new("{task.fields.status} - {task.description}");
        let mut task = Task::new(0, "My Task", Some(100.0));
        task.fields
            .insert("status".to_string(), "active".to_string());
        let text = col.render(&task);
        assert_eq!(text.plain(), "active - My Task");
    }

    #[test]
    fn test_progress_custom_columns() {
        let columns: Vec<Box<dyn ProgressColumn>> = vec![
            Box::new(SpinnerColumn::default()),
            Box::new(TextColumn::new("{task.description}")),
            Box::new(BarColumn::new().with_bar_width(Some(20))),
            Box::new(TaskProgressColumn::default()),
            Box::new(TimeElapsedColumn),
            Box::new(TimeRemainingColumn::default()),
        ];
        let mut progress = Progress::new(columns).with_get_time(|| 1.0);
        let id = progress.add_task("Custom", Some(100.0));
        progress.advance(id, 50.0);

        let table = progress.make_tasks_table();
        assert_eq!(table.columns.len(), 6);
        assert_eq!(table.rows.len(), 1);
    }

    #[test]
    fn test_progress_all_column_types() {
        let columns: Vec<Box<dyn ProgressColumn>> = vec![
            Box::new(SpinnerColumn::default()),
            Box::new(TextColumn::new("{task.description}")),
            Box::new(BarColumn::default()),
            Box::new(TaskProgressColumn::default()),
            Box::new(TimeElapsedColumn),
            Box::new(TimeRemainingColumn::default()),
            Box::new(FileSizeColumn),
            Box::new(TotalFileSizeColumn),
            Box::new(MofNCompleteColumn::default()),
            Box::new(DownloadColumn),
            Box::new(TransferSpeedColumn),
        ];

        let mut progress = Progress::new(columns).with_get_time(|| 1.0);
        let id = progress.add_task("Full test", Some(100000.0));
        progress.advance(id, 50000.0);

        // Render each column for the task
        let task = progress.get_task(id).unwrap();
        for (i, col) in progress.columns.iter().enumerate() {
            let text = col.render(task);
            assert!(
                !text.plain().is_empty() || i == 0, // spinner might be empty if not started
                "column {} rendered empty text",
                i
            );
        }
    }

    // =======================================================================
    // RenderableColumn tests
    // =======================================================================

    #[test]
    fn test_renderable_column_basic() {
        let col = RenderableColumn::new(|task: &Task| Text::new(&task.description, Style::null()));
        let task = Task::new(0, "Hello", Some(100.0));
        let text = col.render(&task);
        assert_eq!(text.plain(), "Hello");
    }

    #[test]
    fn test_renderable_column_custom_content() {
        let col = RenderableColumn::new(|task: &Task| {
            let msg = format!(
                "Step {} of {}",
                task.completed as u64,
                task.total.map(|t| t as u64).unwrap_or(0)
            );
            Text::new(&msg, Style::null())
        });
        let mut task = Task::new(0, "test", Some(10.0));
        task.completed = 7.0;
        let text = col.render(&task);
        assert_eq!(text.plain(), "Step 7 of 10");
    }

    #[test]
    fn test_renderable_column_uses_task_fields() {
        let col = RenderableColumn::new(|task: &Task| {
            let status = task
                .fields
                .get("status")
                .map(|s| s.as_str())
                .unwrap_or("unknown");
            Text::new(status, Style::null())
        });
        let mut task = Task::new(0, "test", Some(100.0));
        task.fields
            .insert("status".to_string(), "downloading".to_string());
        let text = col.render(&task);
        assert_eq!(text.plain(), "downloading");
    }

    #[test]
    fn test_renderable_column_with_percentage() {
        let col = RenderableColumn::new(|task: &Task| {
            Text::new(&format!("{:.0}%", task.percentage()), Style::null())
        });
        let mut task = Task::new(0, "test", Some(200.0));
        task.completed = 100.0;
        let text = col.render(&task);
        assert_eq!(text.plain(), "50%");
    }

    #[test]
    fn test_renderable_column_in_progress() {
        // Verify RenderableColumn works as a ProgressColumn in a Progress instance.
        let columns: Vec<Box<dyn ProgressColumn>> = vec![
            Box::new(TextColumn::new("{task.description}")),
            Box::new(RenderableColumn::new(|task: &Task| {
                Text::new(&format!("[{}]", task.completed as u64), Style::null())
            })),
        ];
        let mut progress = Progress::new(columns).with_get_time(|| 1.0);
        let id = progress.add_task("Demo", Some(100.0));
        progress.advance(id, 42.0);

        let task = progress.get_task(id).unwrap();
        let rendered = progress.columns[1].render(task);
        assert_eq!(rendered.plain(), "[42]");
    }

    #[test]
    fn test_renderable_column_indeterminate_task() {
        let col = RenderableColumn::new(|task: &Task| {
            if task.total.is_none() {
                Text::new("...", Style::null())
            } else {
                Text::new("ok", Style::null())
            }
        });
        let task = Task::new(0, "test", None);
        assert_eq!(col.render(&task).plain(), "...");

        let task2 = Task::new(1, "test", Some(10.0));
        assert_eq!(col.render(&task2).plain(), "ok");
    }

    // =======================================================================
    // ProgressTracker (Progress::track) tests
    // =======================================================================

    #[test]
    fn test_progress_track_iterates_all_items() {
        let mut progress = Progress::new(Progress::default_columns())
            .with_disable(true)
            .with_get_time(|| 1.0);
        let items: Vec<i32> = progress.track(0..5, "Counting", Some(5.0)).collect();
        assert_eq!(items, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_progress_track_advances_task() {
        let mut progress = Progress::new(Progress::default_columns())
            .with_disable(true)
            .with_get_time(|| 1.0);
        let task_id;
        {
            let mut tracker = progress.track(0..3, "Test", Some(3.0));
            task_id = tracker.task_id();
            // Consume all items
            while tracker.next().is_some() {}
        }
        // After iterating 3 items, completed should be 3.0
        let task = progress.get_task(task_id).unwrap();
        assert!((task.completed - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_track_with_known_total_finishes() {
        let mut progress = Progress::new(Progress::default_columns())
            .with_disable(true)
            .with_get_time(|| 1.0);
        let task_id;
        {
            let mut tracker = progress.track(0..10, "Finish", Some(10.0));
            task_id = tracker.task_id();
            while tracker.next().is_some() {}
        }
        let task = progress.get_task(task_id).unwrap();
        assert!(
            task.finished(),
            "task should be finished after iterating all items"
        );
        assert!((task.percentage() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_track_with_none_total_indeterminate() {
        let mut progress = Progress::new(Progress::default_columns())
            .with_disable(true)
            .with_get_time(|| 1.0);
        let task_id;
        {
            let mut tracker = progress.track(0..4, "Indeterminate", None);
            task_id = tracker.task_id();
            while tracker.next().is_some() {}
        }
        let task = progress.get_task(task_id).unwrap();
        // With no total, the task is never auto-finished
        assert!(!task.finished());
        // completed still tracks the count
        assert!((task.completed - 4.0).abs() < f64::EPSILON);
        // percentage is 0 with no total
        assert!((task.percentage() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_track_empty_iterator() {
        let mut progress = Progress::new(Progress::default_columns())
            .with_disable(true)
            .with_get_time(|| 1.0);
        let items: Vec<i32> = progress
            .track(std::iter::empty::<i32>(), "Empty", Some(0.0))
            .collect();
        assert!(items.is_empty());
    }

    #[test]
    fn test_progress_track_size_hint() {
        let mut progress = Progress::new(Progress::default_columns())
            .with_disable(true)
            .with_get_time(|| 1.0);
        let tracker = progress.track(0..10, "Sized", Some(10.0));
        let (lower, upper) = tracker.size_hint();
        assert_eq!(lower, 10);
        assert_eq!(upper, Some(10));
    }

    #[test]
    fn test_progress_track_partial_iteration() {
        let mut progress = Progress::new(Progress::default_columns())
            .with_disable(true)
            .with_get_time(|| 1.0);
        let task_id;
        {
            let mut tracker = progress.track(0..10, "Partial", Some(10.0));
            task_id = tracker.task_id();
            // Only consume 3 items
            tracker.next();
            tracker.next();
            tracker.next();
        }
        let task = progress.get_task(task_id).unwrap();
        assert!((task.completed - 3.0).abs() < f64::EPSILON);
        assert!(!task.finished());
        assert!((task.percentage() - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_track_creates_task() {
        let mut progress = Progress::new(Progress::default_columns())
            .with_disable(true)
            .with_get_time(|| 1.0);
        assert_eq!(progress.tasks().len(), 0);
        let _tracker = progress.track(0..5, "NewTask", Some(5.0));
        assert_eq!(progress.tasks().len(), 1);
        assert_eq!(progress.tasks()[0].description, "NewTask");
        assert_eq!(progress.tasks()[0].total, Some(5.0));
    }

    // =======================================================================
    // ProgressIteratorExt tests
    // =======================================================================

    #[test]
    fn test_progress_iter_collects_all() {
        // All items must be yielded unchanged.
        let items: Vec<i32> = (0..10).progress("Collecting").collect();
        assert_eq!(items, (0..10).collect::<Vec<_>>());
    }

    #[test]
    fn test_progress_iter_advances_count() {
        // After full iteration the task's completed count must match.
        let mut pi = (0..7).progress("Counting");
        // Disable live display to avoid terminal writes in tests.
        pi.progress = Progress::new(Progress::default_columns()).with_disable(true);
        pi.task_id = pi.progress.add_task("Counting", Some(7.0));
        // Drain the iterator.
        while pi.next().is_some() {}
        let task = pi.progress.get_task(pi.task_id).unwrap();
        assert!((task.completed - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_iter_size_hint() {
        // size_hint must delegate to the inner iterator.
        let v = vec![1, 2, 3, 4, 5];
        let pi = v.iter().progress("Hint");
        let (lo, hi) = pi.size_hint();
        assert_eq!(lo, 5);
        assert_eq!(hi, Some(5));
    }

    #[test]
    fn test_progress_iter_vec() {
        // Works on a Vec's iterator.
        let v = vec!["a", "b", "c"];
        let result: Vec<&&str> = v.iter().progress("Vec").collect();
        assert_eq!(result, vec![&"a", &"b", &"c"]);
    }

    #[test]
    fn test_progress_iter_range() {
        // Works on a Range iterator.
        let result: Vec<u64> = (0u64..5).progress("Range").collect();
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_progress_iter_with_total() {
        // progress_with_total sets an explicit total on the task.
        let mut pi = (0..3).progress_with_total("Explicit", 3.0);
        pi.progress = Progress::new(Progress::default_columns()).with_disable(true);
        pi.task_id = pi.progress.add_task("Explicit", Some(3.0));
        while pi.next().is_some() {}
        let task = pi.progress.get_task(pi.task_id).unwrap();
        assert_eq!(task.total, Some(3.0));
        assert!((task.completed - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_iter_empty() {
        // An empty iterator yields nothing and doesn't panic.
        let result: Vec<i32> = std::iter::empty::<i32>().progress("Empty").collect();
        assert!(result.is_empty());
    }

    #[test]
    fn test_progress_iter_size_hint_range() {
        // Range size_hint is (n, Some(n)).
        let pi = (0..100).progress("Range hint");
        let (lo, hi) = pi.size_hint();
        assert_eq!(lo, 100);
        assert_eq!(hi, Some(100));
    }

    #[test]
    fn test_track_iterator_size_hint() {
        // TrackIterator should also delegate size_hint.
        let ti = track(0..50, "test", Some(50.0));
        let (lo, hi) = ti.size_hint();
        assert_eq!(lo, 50);
        assert_eq!(hi, Some(50));
    }

    // =======================================================================
    // reset / all_tasks_finished / print / log tests
    // =======================================================================

    #[test]
    fn test_reset_task() {
        let mut progress = Progress::new(Progress::default_columns())
            .with_disable(true)
            .with_get_time(|| 1.0);
        let tid = progress.add_task("resettable", Some(100.0));
        progress.advance(tid, 50.0);
        // Simulate finish
        progress.update(tid, Some(100.0), None, None, None, None);
        assert!(progress.get_task(tid).unwrap().finished());

        // Now reset
        progress.reset(tid);
        let task = progress.get_task(tid).unwrap();
        assert!((task.completed - 0.0).abs() < f64::EPSILON);
        assert!(task.start_time.is_some());
        assert!(task.stop_time.is_none());
        assert!(task.finished_time.is_none());
        assert!(task.finished_speed.is_none());
        assert!(!task.finished());
    }

    #[test]
    fn test_reset_nonexistent_task() {
        let mut progress = Progress::new(Progress::default_columns())
            .with_disable(true)
            .with_get_time(|| 1.0);
        // Should not panic
        progress.reset(999);
    }

    #[test]
    fn test_all_tasks_finished_true() {
        let mut progress = Progress::new(Progress::default_columns())
            .with_disable(true)
            .with_get_time(|| 1.0);
        let t1 = progress.add_task("a", Some(10.0));
        let t2 = progress.add_task("b", Some(20.0));
        progress.update(t1, Some(10.0), None, None, None, None);
        progress.update(t2, Some(20.0), None, None, None, None);
        assert!(progress.all_tasks_finished());
    }

    #[test]
    fn test_all_tasks_finished_false() {
        let mut progress = Progress::new(Progress::default_columns())
            .with_disable(true)
            .with_get_time(|| 1.0);
        let t1 = progress.add_task("a", Some(10.0));
        let _t2 = progress.add_task("b", Some(20.0));
        progress.update(t1, Some(10.0), None, None, None, None);
        // t2 is NOT finished
        assert!(!progress.all_tasks_finished());
    }

    #[test]
    fn test_all_tasks_finished_empty() {
        let progress = Progress::new(Progress::default_columns()).with_disable(true);
        assert!(progress.all_tasks_finished());
    }

    // =======================================================================
    // ProgressReader tests
    // =======================================================================

    #[test]
    fn test_progress_reader_counts_bytes() {
        use std::io::Read;
        let data = vec![0u8; 256];
        let mut reader = ProgressReader::new(data.as_slice(), |_| {});
        let mut buf = [0u8; 64];
        reader.read(&mut buf).unwrap();
        reader.read(&mut buf).unwrap();
        reader.read(&mut buf).unwrap();
        assert_eq!(reader.total_read(), 192);
    }

    #[test]
    fn test_progress_reader_calls_callback() {
        use std::cell::RefCell;
        use std::io::Read;
        use std::rc::Rc;

        let counts = Rc::new(RefCell::new(Vec::<usize>::new()));
        let counts_clone = Rc::clone(&counts);
        let data = vec![1u8; 100];
        let mut reader =
            ProgressReader::new(data.as_slice(), move |n| counts_clone.borrow_mut().push(n));
        let mut buf = [0u8; 30];
        reader.read(&mut buf).unwrap();
        reader.read(&mut buf).unwrap();
        reader.read(&mut buf).unwrap();
        reader.read(&mut buf).unwrap(); // last 10 bytes
        let recorded = counts.borrow();
        assert_eq!(recorded.len(), 4);
        assert_eq!(recorded[0], 30);
        assert_eq!(recorded[1], 30);
        assert_eq!(recorded[2], 30);
        assert_eq!(recorded[3], 10);
    }

    #[test]
    fn test_progress_reader_empty() {
        use std::io::Read;
        let data: Vec<u8> = vec![];
        let mut reader = ProgressReader::new(data.as_slice(), |_| {});
        let mut buf = [0u8; 64];
        let n = reader.read(&mut buf).unwrap();
        assert_eq!(n, 0);
        assert_eq!(reader.total_read(), 0);
    }
}
