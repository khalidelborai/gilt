//! Main progress tracking orchestrator.

use std::io::{self, Read};

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::live::Live;
use crate::progress::columns::{BarColumn, TaskProgressColumn, TextColumn, TimeRemainingColumn};
use crate::progress::task::{current_time_secs, Task, TaskId};
use crate::segment::Segment;
use crate::style::Style;
use crate::table::Table;
use crate::text::Text;
use crate::utils::filesize;

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
// DownloadColumn
// ---------------------------------------------------------------------------

/// A column that shows `downloaded/total` as human-readable file sizes.
///
/// By default, sizes are formatted with SI (base-1000) units using
/// [`filesize::decimal`]. Set `binary_units` to `true` to use IEC
/// (base-1024) units via [`filesize::binary`].
#[derive(Debug, Clone)]
pub struct DownloadColumn {
    /// When `true`, format sizes with binary (base-1024) units (KiB, MiB, ...).
    /// When `false` (default), use decimal (base-1000) units (kB, MB, ...).
    pub binary_units: bool,
}

impl DownloadColumn {
    /// Create a new `DownloadColumn` with SI decimal units (default).
    pub fn new() -> Self {
        Self {
            binary_units: false,
        }
    }

    /// Create a new `DownloadColumn` that uses IEC binary units.
    pub fn with_binary_units(mut self, binary: bool) -> Self {
        self.binary_units = binary;
        self
    }

    /// Format a byte count using the configured unit system.
    pub(crate) fn format_size(&self, size: u64) -> String {
        if self.binary_units {
            filesize::binary(size, 1, " ")
        } else {
            filesize::decimal(size, 1, " ")
        }
    }
}

impl Default for DownloadColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressColumn for DownloadColumn {
    fn render(&self, task: &Task) -> Text {
        let completed = self.format_size(task.completed as u64);
        let total = match task.total {
            Some(t) => self.format_size(t as u64),
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
///
/// By default, speeds are formatted with SI (base-1000) units using
/// [`filesize::decimal`]. Set `binary_units` to `true` to use IEC
/// (base-1024) units via [`filesize::binary`].
#[derive(Debug, Clone)]
pub struct TransferSpeedColumn {
    /// When `true`, format speeds with binary (base-1024) units (KiB, MiB, ...).
    /// When `false` (default), use decimal (base-1000) units (kB, MB, ...).
    pub binary_units: bool,
}

impl TransferSpeedColumn {
    /// Create a new `TransferSpeedColumn` with SI decimal units (default).
    pub fn new() -> Self {
        Self {
            binary_units: false,
        }
    }

    /// Create a new `TransferSpeedColumn` that uses IEC binary units.
    pub fn with_binary_units(mut self, binary: bool) -> Self {
        self.binary_units = binary;
        self
    }

    /// Format a byte count using the configured unit system.
    pub(crate) fn format_size(&self, size: u64) -> String {
        if self.binary_units {
            filesize::binary(size, 1, " ")
        } else {
            filesize::decimal(size, 1, " ")
        }
    }
}

impl Default for TransferSpeedColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressColumn for TransferSpeedColumn {
    fn render(&self, task: &Task) -> Text {
        let style = Style::parse("progress.data.speed").unwrap_or_else(|_| Style::null());
        match task.speed() {
            Some(speed) => {
                let formatted = self.format_size(speed as u64);
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
    fn gilt_console(&self, _console: &Console, _options: &ConsoleOptions) -> Vec<Segment> {
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
