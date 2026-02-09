//! Port of rich's jobs.py — multi-job progress tracking with reset.
//!
//! Demonstrates adding an overall task plus a per-job task that gets
//! reset for each new job. Uses Progress.log() for status messages.

use std::thread;
use std::time::Duration;

use gilt::console::Console;
use gilt::panel::Panel;
use gilt::progress::Progress;
use gilt::style::Style;
use gilt::text::Text;

fn main() {
    let jobs: Vec<f64> = vec![100.0, 150.0, 25.0, 70.0, 110.0];

    let console = Console::builder().width(80).force_terminal(true).build();

    let mut progress = Progress::new(Progress::default_columns())
        .with_console(console)
        .with_auto_refresh(false);

    // Print intro panel via the progress console
    let intro = Panel::new(Text::new(
        "A demonstration of progress with a current task and overall progress.",
        Style::parse("bold blue").unwrap_or_else(|_| Style::null()),
    ));
    progress.print(&intro);

    // Add tasks: overall tracks total steps across all jobs
    let total: f64 = jobs.iter().sum();
    let master_task = progress.add_task("overall", Some(total));

    progress.start();

    for (job_no, &job_steps) in jobs.iter().enumerate() {
        progress.log(&format!("Starting job #{}", job_no));
        thread::sleep(Duration::from_millis(100));

        // Create a new task for each job (simulating reset behavior)
        let job_task = progress.add_task(&format!("job #{}", job_no), Some(job_steps));

        // Simulate work on this job
        let mut completed = 0.0;
        while completed < job_steps {
            progress.advance(job_task, 1.0);
            completed += 1.0;
            progress.refresh();
            thread::sleep(Duration::from_millis(5));
        }

        // Advance the master task by this job's total
        progress.advance(master_task, job_steps);
        progress.refresh();

        // Hide the completed job task
        progress.update(job_task, None, None, None, None, Some(false));

        progress.log(&format!("Job #{} is complete", job_no));
    }

    progress.stop();

    // Print completion panel
    let mut done_console = Console::new();
    let done = Panel::new(Text::new(
        "All done!",
        Style::parse("bold green").unwrap_or_else(|_| Style::null()),
    ));
    done_console.print(&done);
}
