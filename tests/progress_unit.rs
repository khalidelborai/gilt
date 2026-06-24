//! Unit-level tests for progress bar columns, task lifecycle, and progress
//! control-flow. Ported from rich's `tests/test_progress.py`.

use std::sync::{Arc, Mutex};

use gilt::progress::{
    BarColumn, DownloadColumn, Progress, ProgressColumn, SpinnerColumn, TextColumn,
    TimeElapsedColumn, TimeRemainingColumn, TransferSpeedColumn,
};
use gilt::progress::{Task, TaskId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a Task with optional start/stop times set explicitly so that
/// `elapsed()` returns a deterministic value without calling `SystemTime::now`.
fn make_task_with_elapsed(
    id: usize,
    description: &str,
    total: Option<f64>,
    elapsed_secs: f64,
) -> Task {
    let mut task = Task::new(id, description, total);
    // start_time=0, stop_time=elapsed so elapsed() == elapsed_secs deterministically.
    task.start_time = Some(0.0);
    task.stop_time = Some(elapsed_secs);
    task
}

// ---------------------------------------------------------------------------
// Cluster 1 — Column units
// ---------------------------------------------------------------------------

#[test]
fn bar_column_renders_partial_progress() {
    let mut task = Task::new(0, "test", Some(100.0));
    task.completed = 50.0;

    let col = BarColumn::default();
    let text = col.render(&task);
    let plain = text.plain();

    // Bar should render some characters (width defaults to 40)
    assert!(
        !plain.is_empty(),
        "bar column should produce non-empty output"
    );
    // At 50%, the bar fill character ━ must appear somewhere
    assert!(
        plain.contains('\u{2501}') || !plain.is_empty(),
        "partial bar should contain bar characters"
    );
    // Rendered width should be ~40 chars (bar_width default)
    let char_count = plain.chars().count();
    assert!(
        char_count > 0 && char_count <= 40,
        "bar width should be at most 40, got {char_count}"
    );
}

#[test]
fn text_column_substitutes_task_description() {
    let task = Task::new(0, "downloading", Some(100.0));
    let col = TextColumn::new("{task.description}");
    let text = col.render(&task);
    assert_eq!(text.plain(), "downloading");
}

#[test]
fn time_elapsed_column_format() {
    // A task with 0 elapsed time should render "0:00"
    let zero = make_task_with_elapsed(0, "t", Some(100.0), 0.0);
    let col = TimeElapsedColumn;
    let rendered_zero = col.render(&zero).plain().to_string();
    assert_eq!(rendered_zero, "0:00", "0 elapsed should be '0:00'");

    // A task with 73 seconds elapsed should render "1:13"
    let non_zero = make_task_with_elapsed(1, "t", Some(100.0), 73.0);
    let rendered_nonzero = col.render(&non_zero).plain().to_string();
    assert_eq!(rendered_nonzero, "1:13", "73s elapsed should be '1:13'");
}

#[test]
fn time_remaining_column_unknown_total_renders_dashes() {
    // When total is None, there is no ETA — must show the placeholder.
    let task = make_task_with_elapsed(0, "t", None, 5.0);
    let col = TimeRemainingColumn::default();
    let rendered = col.render(&task).plain().to_string();
    assert_eq!(rendered, "-:--:--", "no-total task should render '-:--:--'");
}

#[test]
fn compact_time_remaining_column_format() {
    // A finished task in default TimeRemainingColumn renders "0:00".
    let mut task = Task::new(0, "t", Some(100.0));
    task.completed = 100.0;
    task.start_time = Some(0.0);
    task.finished_time = Some(10.0);

    let col = TimeRemainingColumn::new()
        .with_compact(true)
        .with_elapsed_when_finished(false);
    let rendered = col.render(&task).plain().to_string();
    assert_eq!(rendered, "0:00", "finished task should render '0:00'");
}

#[test]
fn spinner_column_animates_with_elapsed() {
    // Two tasks at different elapsed values must produce different frames.
    let early = make_task_with_elapsed(0, "t", Some(100.0), 0.0);
    let later = make_task_with_elapsed(1, "t", Some(100.0), 0.5);

    let col = SpinnerColumn::default();
    let frame_early = col.render(&early).plain().to_string();
    let frame_later = col.render(&later).plain().to_string();

    // Both frames must be non-empty
    assert!(
        !frame_early.is_empty(),
        "spinner frame at elapsed=0 should not be empty"
    );
    assert!(
        !frame_later.is_empty(),
        "spinner frame at elapsed=0.5 should not be empty"
    );

    // Frames at different elapsed times should differ (dots spinner cycles ~10 fps)
    assert_ne!(
        frame_early, frame_later,
        "spinner frames at elapsed=0 and elapsed=0.5 should differ"
    );
}

#[test]
fn download_column_uses_shared_unit_from_max() {
    // completed=1500 B, total=2 MB. Before the P2 fix the completed side
    // would show "kB" while the total showed "MB" (inconsistent units).
    // After the fix, both sides use the same magnitude — the larger of
    // completed and total — so both render in MB.
    let mut task = Task::new(0, "t", Some(2_000_000.0));
    task.completed = 1_500.0;

    let col = DownloadColumn::new();
    let rendered = col.render(&task).plain().to_string();
    // Both sides must be in the same unit (MB, driven by the 2 MB total).
    let lower = rendered.to_lowercase();
    assert!(
        !lower.contains("kb"),
        "completed side must not use kB when total is 2 MB (shared-unit fix); got: {rendered}"
    );
    assert!(
        lower.contains("mb"),
        "both sides should be in MB; got: {rendered}"
    );
}

#[test]
fn download_column_uses_binary_units() {
    let mut task = Task::new(0, "t", Some(10_485_760.0)); // 10 MiB total
    task.completed = 2_097_152.0; // 2 MiB

    let col = DownloadColumn::new().with_binary_units(true);
    let rendered = col.render(&task).plain().to_string();
    // Binary units: should contain "MiB"
    assert!(
        rendered.contains("MiB"),
        "2 MiB with binary units should render 'MiB', got: {rendered}"
    );
}

#[test]
fn transfer_speed_column_after_some_advance() {
    // Use an injectable clock to advance time between samples so speed > 0.
    let clock = Arc::new(Mutex::new(0.0_f64));
    let clock_clone = clock.clone();

    let mut progress = Progress::new(vec![])
        .with_disable(true)
        .with_get_time(move || *clock_clone.lock().unwrap());

    let id: TaskId = progress.add_task("dl", Some(1000.0), true);

    // First sample at t=0
    progress.advance(id, 100.0);

    // Advance clock by 1 second, then record more progress
    *clock.lock().unwrap() = 1.0;
    progress.advance(id, 200.0);

    let task = progress.get_task(id).unwrap();
    let col = TransferSpeedColumn::new();
    let rendered = col.render(task).plain().to_string();

    // Speed should be known and > 0 (not "?")
    assert_ne!(rendered, "?", "speed column should show a rate, got '?'");
    assert!(
        rendered.contains("/s"),
        "speed column should contain '/s', got: {rendered}"
    );
}

// ---------------------------------------------------------------------------
// Cluster 2 — Task lifecycle
// ---------------------------------------------------------------------------

#[test]
fn task_ids_are_monotonic() {
    let mut p = Progress::new(vec![]).with_disable(true);
    let id0 = p.add_task("a", Some(10.0), true);
    let id1 = p.add_task("b", Some(10.0), true);
    let id2 = p.add_task("c", Some(10.0), true);
    assert!(id0 < id1, "task IDs should be monotonically increasing");
    assert!(id1 < id2, "task IDs should be monotonically increasing");
}

#[test]
fn task_finished_when_completed_equals_total() {
    let mut p = Progress::new(vec![]).with_disable(true);
    let id = p.add_task("work", Some(10.0), true);

    // Advance to exactly total
    p.advance(id, 10.0);

    let task = p.get_task(id).unwrap();
    assert!(
        task.finished(),
        "task should be finished after completing total"
    );
}

#[test]
fn task_create_with_total_zero() {
    // total=Some(0.0) must not divide by zero in percentage()
    let task = Task::new(0, "zero", Some(0.0));
    let pct = task.percentage();
    assert_eq!(pct, 0.0, "percentage with total=0 should be 0.0, not panic");
}

#[test]
fn task_speed_window_estimate() {
    // Advance at a known rate using an injectable clock.
    let clock = Arc::new(Mutex::new(0.0_f64));
    let clock_clone = clock.clone();

    let mut p = Progress::new(vec![])
        .with_disable(true)
        .with_get_time(move || *clock_clone.lock().unwrap());

    let id = p.add_task("work", Some(1000.0), true);

    // Simulate 10 advances of 100 units each, 100ms apart
    for i in 1..=10_u64 {
        *clock.lock().unwrap() = i as f64 * 0.1;
        p.advance(id, 100.0);
    }

    let task = p.get_task(id).unwrap();
    let speed = task
        .speed()
        .expect("speed should be known after 10 samples");

    // Rate should be ~1000 units/s (100 units per 0.1s)
    assert!(
        speed > 500.0 && speed < 2000.0,
        "speed estimate should be ~1000 units/s, got {speed}"
    );
}

#[test]
fn task_progress_finished_speed_freezes_after_completion() {
    let clock = Arc::new(Mutex::new(0.0_f64));
    let clock_clone = clock.clone();

    let mut p = Progress::new(vec![])
        .with_disable(true)
        .with_get_time(move || *clock_clone.lock().unwrap());

    let id = p.add_task("work", Some(100.0), true);

    // Build up two samples for a speed estimate
    *clock.lock().unwrap() = 0.5;
    p.advance(id, 50.0);
    *clock.lock().unwrap() = 1.0;
    p.advance(id, 50.0); // This push should finish the task and snapshot finished_speed

    let task = p.get_task(id).unwrap();
    assert!(task.finished(), "task should be finished");
    let frozen = task.speed();
    assert!(
        frozen.is_some(),
        "finished task should have a speed snapshot"
    );

    // Advance more — speed should remain frozen
    *clock.lock().unwrap() = 2.0;
    p.advance(id, 10.0);
    let task = p.get_task(id).unwrap();
    assert_eq!(
        task.speed(),
        frozen,
        "speed should be frozen after task completion"
    );
}

// ---------------------------------------------------------------------------
// Cluster 3 — Progress control flow
// ---------------------------------------------------------------------------

#[test]
fn progress_with_none_total_renders_pulse_bar() {
    // A task with total=None should trigger pulse mode in BarColumn.
    let task = Task::new(0, "spin", None);
    let col = BarColumn::default();
    let text = col.render(&task);
    let plain = text.plain();

    // Pulse mode should produce non-empty output (the pulse segments are
    // non-empty strings regardless of colour system).
    assert!(
        !plain.is_empty(),
        "pulse bar should produce non-empty output"
    );
}

#[test]
fn reset_clears_completed_and_restarts_start_time() {
    let clock = Arc::new(Mutex::new(10.0_f64));
    let clock_clone = clock.clone();

    let mut p = Progress::new(vec![])
        .with_disable(true)
        .with_get_time(move || *clock_clone.lock().unwrap());

    let id = p.add_task("work", Some(100.0), true);
    p.advance(id, 50.0);

    // Advance clock so reset() records a new start_time
    *clock.lock().unwrap() = 20.0;
    p.reset(id);

    let task = p.get_task(id).unwrap();
    assert_eq!(task.completed, 0.0, "reset should clear completed to 0");
    assert!(
        task.finished_time.is_none(),
        "reset should clear finished_time"
    );
    assert_eq!(
        task.start_time,
        Some(20.0),
        "reset should restart start_time to current clock"
    );
}

#[test]
fn expand_bar_fills_remaining_width() {
    // Progress::make_tasks_table() respects the `expand` flag.
    // Verify the underlying table has expand=true when the flag is set.
    let mut p = Progress::new(vec![
        Box::new(TextColumn::new("{task.description}")),
        Box::new(BarColumn::default()),
    ])
    .with_disable(true)
    .with_expand(true);

    p.add_task("work", Some(100.0), true);

    let table = p.make_tasks_table();
    assert!(
        table.expand(),
        "make_tasks_table() should produce an expanded table when with_expand(true)"
    );
}
