//! Dynamic progress — add tasks while progress is running.
//!
//! Run with: `cargo run --example dynamic_progress`
//!
//! Demonstrates adding new tasks dynamically as existing ones complete,
//! simulating a build system that discovers work at runtime.

use std::thread;
use std::time::Duration;

use gilt::console::Console;
use gilt::progress::{Progress, TaskId};

/// A work item that may spawn follow-up work when it completes.
struct Job {
    task_id: TaskId,
    total: f64,
    speed: f64,
}

fn main() {
    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    let mut progress = Progress::new(Progress::default_columns())
        .with_console(console)
        .with_auto_refresh(false);

    // Start with three initial compile tasks.
    let mut jobs: Vec<Job> = vec![
        Job {
            task_id: progress.add_task("Compiling core...", Some(100.0)),
            total: 100.0,
            speed: 5.0,
        },
        Job {
            task_id: progress.add_task("Compiling utils...", Some(60.0)),
            total: 60.0,
            speed: 8.0,
        },
        Job {
            task_id: progress.add_task("Compiling macros...", Some(40.0)),
            total: 40.0,
            speed: 4.0,
        },
    ];

    // Follow-up tasks to add as earlier tasks finish.
    let follow_ups: Vec<(&str, f64, f64)> = vec![
        ("Linking core...", 50.0, 10.0),
        ("Compiling app...", 80.0, 6.0),
        ("Running tests...", 120.0, 12.0),
        ("Generating docs...", 30.0, 3.0),
    ];
    let mut next_follow_up = 0;
    let mut finished_count = 0;

    progress.start();

    loop {
        // Advance all active jobs.
        for job in &jobs {
            progress.advance(job.task_id, job.speed);
        }

        progress.refresh();

        // Check for newly finished jobs.
        let mut new_finished = 0;
        for job in &jobs {
            if let Some(task) = progress.get_task(job.task_id) {
                if task.finished() && task.completed >= job.total {
                    new_finished += 1;
                }
            }
        }

        // Add follow-up tasks when a job finishes for the first time.
        while finished_count < new_finished && next_follow_up < follow_ups.len() {
            let (desc, total, speed) = follow_ups[next_follow_up];
            let tid = progress.add_task(desc, Some(total));
            jobs.push(Job {
                task_id: tid,
                total,
                speed,
            });
            next_follow_up += 1;
            finished_count += 1;
        }
        finished_count = new_finished;

        // Check if all jobs are done.
        let all_done = jobs.iter().all(|job| {
            progress
                .get_task(job.task_id)
                .is_none_or(|t| t.finished())
        });

        if all_done {
            break;
        }

        thread::sleep(Duration::from_millis(50));
    }

    progress.stop();

    println!("\nBuild pipeline complete! {} tasks executed.", jobs.len());
}
