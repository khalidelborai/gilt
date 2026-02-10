//! Progress tracking system -- configurable progress bars with live display.
//!
//! Port of Python's `rich/progress.py`. Provides task tracking with
//! completion percentages, configurable columns (text, bar, spinner,
//! time, speed), live-updating display, and iterator wrapping.

mod core;
mod task;

pub mod columns;

// Re-export all public types from submodules for backward compatibility
pub use core::{
    track, DownloadColumn, Progress, ProgressColumn, ProgressIter, ProgressIteratorExt,
    ProgressReader, ProgressTracker, RenderableColumn, TrackIterator, TransferSpeedColumn,
};
pub use task::{format_time, ProgressSample, Task, TaskId};

// Re-export column types
pub use columns::{
    BarColumn, FileSizeColumn, MofNCompleteColumn, SpinnerColumn, TaskProgressColumn, TextColumn,
    TimeElapsedColumn, TimeRemainingColumn, TotalFileSizeColumn,
};
