//! Main progress tracking orchestrator.

use std::io::{self, Read, Seek, SeekFrom};

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::live::Live;
use crate::progress::columns::{BarColumn, TaskProgressColumn, TextColumn, TimeRemainingColumn};
use crate::progress::task::{current_time_secs, Task, TaskId};
use crate::segment::{Segment, TaskbarState};
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
        let style = Style::parse("progress.download");
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
        let style = Style::parse("progress.data.speed");
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
    /// When `true`, emit OSC 9;4 taskbar progress updates on each refresh.
    ///
    /// Default: `false`. Enable with [`Progress::with_taskbar`].
    taskbar: bool,
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
            taskbar: false,
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

    /// Enable or disable OSC 9;4 taskbar progress updates (builder pattern).
    ///
    /// When enabled, `Progress` emits ConEmu/Windows Terminal taskbar progress
    /// (Normal with overall percent) on each refresh, and removes it on stop.
    /// Default: `false`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gilt::progress::Progress;
    ///
    /// let mut progress = Progress::new(Progress::default_columns())
    ///     .with_taskbar(true);
    /// progress.start();
    /// progress.add_task("demo", Some(100.0));
    /// progress.stop();
    /// ```
    #[must_use]
    pub fn with_taskbar(mut self, enabled: bool) -> Self {
        self.taskbar = enabled;
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
    ///
    /// Refreshes the live display after the state mutation so the new
    /// values appear without waiting for the next auto-refresh tick.
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
        let mut changed = false;
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            if let Some(desc) = description {
                task.description = desc.to_string();
                changed = true;
            }
            if let Some(t) = total {
                // When the total changes the existing speed samples become
                // meaningless (they measured progress towards a different
                // goal).  Clear both the sliding window and the history.
                if task.total != Some(t) {
                    task.samples.clear();
                    task.progress.clear();
                }
                task.total = Some(t);
                changed = true;
            }
            if let Some(c) = completed {
                task.completed = c;
                changed = true;
            }
            if let Some(a) = advance {
                task.completed += a;
                changed = true;
            }
            if let Some(v) = visible {
                task.visible = v;
                changed = true;
            }

            // Record a speed sample only when something actually changed —
            // recording on no-op calls would corrupt the speed estimate.
            if changed && task.started() && !task.finished() {
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
        if changed {
            self.mark_dirty();
        }
    }

    /// Advance a task's completed count by the given amount.
    ///
    /// Triggers a live-display refresh through [`update`](Self::update).
    pub fn advance(&mut self, task_id: TaskId, advance: f64) {
        self.update(task_id, None, None, Some(advance), None, None);
    }

    /// Mark a task as started (set start_time to now).
    pub fn start_task(&mut self, task_id: TaskId) {
        let now = (self.get_time)();
        let mut changed = false;
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            if task.start_time.is_none() {
                task.start_time = Some(now);
                changed = true;
            }
        }
        if changed {
            self.mark_dirty();
        }
    }

    /// Mark a task as stopped (set stop_time to now).
    pub fn stop_task(&mut self, task_id: TaskId) {
        let now = (self.get_time)();
        let mut changed = false;
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.stop_time = Some(now);
            changed = true;
        }
        if changed {
            self.mark_dirty();
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

    // -- File helpers -------------------------------------------------------

    /// Open a file for reading with a progress task automatically attached.
    ///
    /// Computes the file's length from its metadata so the bar shows a
    /// known total and ETA. The returned reader, when read, advances the
    /// task; when dropped, the task is left in place (call
    /// [`remove_task`](Self::remove_task) or [`stop_task`](Self::stop_task)
    /// explicitly if you want it gone before [`stop`](Self::stop)).
    ///
    /// The returned `ProgressReader<'_, File>` borrows `self` mutably for
    /// its entire lifetime, so no other `&mut Progress` methods may be called
    /// while the reader is live — the same constraint as [`track`](Self::track).
    ///
    /// # Errors
    ///
    /// Returns [`std::io::Error`] if the file cannot be opened or its length
    /// cannot be determined.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gilt::progress::Progress;
    ///
    /// let mut progress = Progress::new(Progress::default_columns());
    /// let mut reader = progress.open_file("file.bin", "Reading").unwrap();
    /// // Use `reader` as any `std::io::Read` impl.
    /// ```
    pub fn open_file(
        &mut self,
        path: impl AsRef<std::path::Path>,
        description: &str,
    ) -> io::Result<ProgressReader<'_, std::fs::File>> {
        let file = std::fs::File::open(path)?;
        let len = file.metadata()?.len();
        let task_id = self.add_task(description, Some(len as f64));
        // Sound: the callback captures `&mut *self` (a re-borrow) with the
        // explicit `'_` lifetime that ties the returned ProgressReader to
        // this `&mut self` borrow.  The borrow checker therefore prevents any
        // concurrent `&mut Progress` use while the reader is alive.
        Ok(ProgressReader::new(file, move |n| {
            self.advance(task_id, n as f64);
        }))
    }

    /// Wrap an arbitrary `Read + Seek` impl in a progress-tracking reader,
    /// auto-creating a task with the seekable stream length as total.
    ///
    /// Uses `SeekFrom::End(0)` to determine the stream length, then rewinds
    /// to the current beginning before wrapping.
    ///
    /// The returned `ProgressReader<'_, R>` borrows `self` mutably for
    /// its entire lifetime — same constraint as [`open_file`](Self::open_file).
    ///
    /// # Errors
    ///
    /// Returns [`std::io::Error`] if the seek operations fail.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use gilt::progress::Progress;
    ///
    /// let data = b"hello world";
    /// let cursor = Cursor::new(data.to_vec());
    /// let mut progress = Progress::new(Progress::default_columns())
    ///     .with_disable(true);
    /// let _reader = progress.wrap_file(cursor, "Processing").unwrap();
    /// ```
    pub fn wrap_file<R: Read + Seek>(
        &mut self,
        mut reader: R,
        description: &str,
    ) -> io::Result<ProgressReader<'_, R>> {
        let len = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(0))?;
        let task_id = self.add_task(description, Some(len as f64));
        // Sound: same lifetime-borrow approach as `open_file`.
        Ok(ProgressReader::new(reader, move |n| {
            self.advance(task_id, n as f64);
        }))
    }

    // -- Taskbar helpers ----------------------------------------------------

    /// Compute the overall progress percentage across all visible tasks.
    ///
    /// Returns `None` if no tasks have a known total.  Otherwise returns the
    /// ratio of total completed to total work across all visible tasks,
    /// clamped to 0–100.
    fn overall_percent(&self) -> Option<u8> {
        let (mut total_completed, mut total_work) = (0.0f64, 0.0f64);
        let mut has_total = false;
        for task in &self.tasks {
            if !task.visible {
                continue;
            }
            if let Some(t) = task.total {
                total_work += t;
                total_completed += task.completed.min(t);
                has_total = true;
            }
        }
        if !has_total || total_work <= 0.0 {
            return None;
        }
        let pct = ((total_completed / total_work) * 100.0).clamp(0.0, 100.0) as u8;
        Some(pct)
    }

    /// Emit a taskbar progress update when `taskbar` is enabled.
    fn emit_taskbar_progress(&mut self, state: TaskbarState, percent: u8) {
        if !self.taskbar {
            return;
        }
        self.live.console_mut().set_taskbar_progress(state, percent);
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
    ///
    /// When `with_taskbar(true)` was set, emits the Remove taskbar state
    /// before stopping the live display.
    pub fn stop(&mut self) {
        if self.disable {
            return;
        }
        // Emit taskbar Remove before the live display clears.
        self.emit_taskbar_progress(TaskbarState::Remove, 0);
        self.live.stop();
    }

    /// Refresh the live display with current task state and force an
    /// immediate paint.
    ///
    /// State-mutating helpers (`update`, `advance`, `start_task`,
    /// `stop_task`) call an internal `mark_dirty` instead of this —
    /// they rebuild the stored renderable without forcing a paint, so the
    /// auto-refresh thread paints at the configured rate (default 10 Hz).
    /// Tight `advance()` loops therefore generate at most one paint per
    /// refresh-tick interval rather than one per call.
    ///
    /// When `with_taskbar(true)` is set, also emits a Normal taskbar
    /// progress update with the overall completion percentage.
    pub fn refresh(&mut self) {
        if self.disable {
            return;
        }
        let table_text = self.render_tasks_text();
        self.live.update_renderable(table_text, true);
        // Emit taskbar progress update if enabled.
        if self.taskbar {
            let pct = self.overall_percent().unwrap_or(0);
            self.emit_taskbar_progress(TaskbarState::Normal, pct);
        }
    }

    /// Re-render the task table and store it on the live display, but do
    /// **not** trigger an immediate paint. The auto-refresh thread will
    /// pick up the updated renderable on its next tick.
    fn mark_dirty(&mut self) {
        if self.disable {
            return;
        }
        let table_text = self.render_tasks_text();
        // refresh = false: just update s.renderable so the next tick paints
        // the fresh content; do not synchronously call write_segments.
        self.live.update_renderable(table_text, false);
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

        // Add a row for each visible task, preserving column styling.
        for task in &self.tasks {
            if !task.visible {
                continue;
            }
            let cells: Vec<Text> = self.columns.iter().map(|col| col.render(task)).collect();
            table.add_row_text(&cells);
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
    fn gilt_console(&self, console: &Console, _options: &ConsoleOptions) -> Vec<Segment> {
        let text = self.render_tasks_text();
        text.render_themed(console)
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
/// # Lifetime
///
/// The lifetime parameter `'p` ties this reader to the `Progress` borrow that
/// backs it (when created via [`Progress::open_file`] or
/// [`Progress::wrap_file`]).  For standalone use with a `'static` callback
/// simply omit the lifetime or use `ProgressReader<'static, R>`.
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
pub struct ProgressReader<'p, R> {
    inner: R,
    callback: Box<dyn FnMut(usize) + 'p>,
    total_read: usize,
}

impl<'p, R> ProgressReader<'p, R> {
    /// Wrap a reader with a progress callback.
    ///
    /// The `callback` is invoked after every successful read with the
    /// number of bytes that were read.
    pub fn new(inner: R, callback: impl FnMut(usize) + 'p) -> Self {
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

impl<R: Read> Read for ProgressReader<'_, R> {
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
    use crate::console::Console;
    use std::io::{Cursor, Read};

    fn make_progress() -> Progress {
        Progress::new(Progress::default_columns()).with_disable(true)
    }

    // -- open_file tests ----------------------------------------------------

    #[test]
    fn open_file_creates_task_with_file_length() {
        let content = b"hello, progress world!";
        let path = std::env::temp_dir().join("gilt_test_open_file_task.bin");
        std::fs::write(&path, content).unwrap();

        let mut progress = make_progress();
        // Drop the reader immediately after creating it; we only care about
        // the task metadata.  The borrow-based API requires the reader to be
        // dropped before `progress` is accessed again.
        {
            let _reader = progress.open_file(&path, "Reading").unwrap();
        }

        let tasks = progress.tasks();
        assert_eq!(tasks.len(), 1);
        assert_eq!(
            tasks[0].total,
            Some(content.len() as f64),
            "task total should equal file byte length"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn open_file_advances_task_on_read() {
        let content = b"advance me please";
        let path = std::env::temp_dir().join("gilt_test_open_file_advance.bin");
        std::fs::write(&path, content).unwrap();

        let mut progress = make_progress();

        // Read all bytes through the progress reader, then drop it so we can
        // inspect `progress` again (the borrow-based API requires this).
        let total_read = {
            let mut reader = progress.open_file(&path, "Reading").unwrap();
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf).unwrap();
            reader.total_read()
        };

        assert_eq!(
            total_read,
            content.len(),
            "ProgressReader.total_read should equal bytes read"
        );
        // The task's completed counter is advanced through the borrow closure.
        let task = &progress.tasks()[0];
        assert_eq!(
            task.completed,
            content.len() as f64,
            "task.completed should equal bytes read"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn open_file_returns_error_for_missing_path() {
        let mut progress = make_progress();
        let result = progress.open_file("/nonexistent/path/gilt_test.bin", "Reading");
        assert!(result.is_err(), "should error for nonexistent path");
    }

    // -- wrap_file tests ----------------------------------------------------

    #[test]
    fn wrap_file_uses_seek_to_compute_total() {
        let content = b"seekable data here";
        let cursor = Cursor::new(content.to_vec());

        let mut progress = make_progress();
        // Drop the reader before inspecting `progress`.
        {
            let _reader = progress.wrap_file(cursor, "Processing").unwrap();
        }

        let tasks = progress.tasks();
        assert_eq!(tasks.len(), 1);
        assert_eq!(
            tasks[0].total,
            Some(content.len() as f64),
            "task total should equal cursor length determined via seek"
        );
    }

    #[test]
    fn wrap_file_advances_task_on_read() {
        let content = b"wrap and advance";
        let cursor = Cursor::new(content.to_vec());

        let mut progress = make_progress();
        let read_data: Vec<u8> = {
            let mut reader = progress.wrap_file(cursor, "Processing").unwrap();
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf).unwrap();
            buf
        };

        assert_eq!(read_data.as_slice(), content as &[u8]);
        let task = &progress.tasks()[0];
        assert_eq!(
            task.completed,
            content.len() as f64,
            "task.completed should equal bytes read"
        );
    }

    // -- with_taskbar tests -------------------------------------------------

    /// `Progress::with_taskbar` builder sets the flag and the overall_percent
    /// helper returns the correct proportion when tasks have a known total.
    #[test]
    fn test_progress_overall_percent_basic() {
        let mut p = make_progress();
        let t = p.add_task("demo", Some(100.0));
        p.update(t, Some(50.0), None, None, None, None);
        let pct = p.overall_percent();
        assert_eq!(pct, Some(50), "50/100 should give 50%");
    }

    #[test]
    fn test_progress_overall_percent_no_total() {
        let mut p = make_progress();
        p.add_task("indeterminate", None);
        assert_eq!(
            p.overall_percent(),
            None,
            "task with no total should give None"
        );
    }

    /// `with_taskbar(true)` builder method sets the flag and the Normal state
    /// OSC sequence is produced via the underlying console when the taskbar is
    /// enabled and `refresh()` is called.
    ///
    /// We verify by wiring a recording console and inspecting raw escape bytes.
    #[test]
    fn test_progress_with_taskbar_emits_normal() {
        // Build a recording console so we can inspect what is emitted.
        let recording_console = Console::builder()
            .force_terminal(true)
            .no_color(true)
            .record(true)
            .build();

        let mut p = Progress::new(Progress::default_columns())
            .with_disable(false) // enable rendering
            .with_taskbar(true)
            .with_console(recording_console)
            .with_auto_refresh(false); // manual refresh only

        let t = p.add_task("demo", Some(100.0));
        p.update(t, Some(50.0), None, None, None, None);
        // refresh() should emit Normal state + 50%.
        p.refresh();

        let output = p.live.console_mut().export_text(false, true);
        assert!(
            output.contains("\x1b]9;4;1;"),
            "taskbar normal state should appear in output; got {:?}",
            output
        );
    }
}
