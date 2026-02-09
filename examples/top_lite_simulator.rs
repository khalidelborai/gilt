//! Lite simulation of the `top` Linux command with a live-updating process table.
//!
//! Port of Python rich's `top_lite_simulator.py`. Generates random process data
//! and displays it in a styled table that refreshes every 500ms using gilt's
//! Live display.
//!
//! Run with: `cargo run --example top_lite_simulator`
//!
//! The display runs for approximately 10 seconds, then exits.

use std::thread;
use std::time::{Duration, Instant};

use gilt::console::Console;
use gilt::live::Live;
use gilt::style::Style;
use gilt::text::Text;

// ---------------------------------------------------------------------------
// Simple LCG pseudo-random number generator (no external crate)
// ---------------------------------------------------------------------------

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        SimpleRng { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state >> 33
    }

    fn next_f64(&mut self) -> f64 {
        self.next_u64() as f64 / u32::MAX as f64
    }

    fn next_range(&mut self, min: u64, max: u64) -> u64 {
        min + self.next_u64() % (max - min + 1)
    }
}

// ---------------------------------------------------------------------------
// Process data
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Process {
    pid: u32,
    command: String,
    cpu_percent: f64,
    memory: u64,
    start_secs_ago: u64,
    thread_count: u32,
    state: &'static str,
}

impl Process {
    fn memory_str(&self) -> String {
        if self.memory > 1_000_000 {
            format!("{}M", self.memory / 1_000_000)
        } else if self.memory > 1_000 {
            format!("{}K", self.memory / 1_000)
        } else {
            format!("{}B", self.memory)
        }
    }

    fn time_str(&self) -> String {
        let secs = self.start_secs_ago;
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        let s = secs % 60;
        format!("{:02}:{:02}:{:02}", hours, mins, s)
    }
}

static COMMANDS: &[&str] = &[
    "systemd",
    "kworker/0:1",
    "sshd",
    "bash",
    "cargo",
    "rustc",
    "node",
    "python3",
    "nginx",
    "postgres",
    "redis-server",
    "dockerd",
    "containerd",
    "Xorg",
    "pulseaudio",
    "dbus-daemon",
    "NetworkManager",
    "vim",
    "top",
    "htop",
    "git",
    "rsync",
    "tar",
    "wget",
    "curl",
    "journald",
    "cron",
    "syslog-ng",
    "snapd",
    "code-server",
];

fn generate_processes(rng: &mut SimpleRng, count: usize) -> Vec<Process> {
    let mut processes: Vec<Process> = Vec::with_capacity(count);
    for i in 0..count {
        let cmd_idx = rng.next_range(0, COMMANDS.len() as u64 - 1) as usize;
        let cpu = rng.next_f64() * 25.0;
        let mem_base = rng.next_range(10, 200);
        let memory = mem_base * mem_base * mem_base; // 1K - 8M range
        let start_secs = rng.next_range(0, 500) * rng.next_range(0, 500);
        let threads = rng.next_range(1, 32) as u32;
        let state = if rng.next_range(0, 10) < 8 {
            "running"
        } else {
            "sleeping"
        };

        processes.push(Process {
            pid: (i as u32 + 1) * 100 + rng.next_range(1, 99) as u32,
            command: COMMANDS[cmd_idx].to_string(),
            cpu_percent: cpu,
            memory,
            start_secs_ago: start_secs,
            thread_count: threads,
            state,
        });
    }

    // Sort by CPU% descending
    processes.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap());
    processes
}

// ---------------------------------------------------------------------------
// Build styled text table for the Live display
// ---------------------------------------------------------------------------

fn build_process_display(processes: &[Process]) -> Text {
    let header_style = Style::parse("bold white on blue").unwrap_or_else(|_| Style::null());
    let dim_style = Style::parse("dim").unwrap_or_else(|_| Style::null());

    let mut text = Text::empty();

    // Header
    let header = format!(
        " {:>6}  {:<18} {:>6}  {:>8}  {:>10}  {:>4}  {:<10}\n",
        "PID", "Command", "CPU %", "Memory", "Time", "THR", "State"
    );
    text.append_str(&header, Some(header_style));

    // Separator line
    text.append_str(&format!(" {}\n", "-".repeat(72)), Some(dim_style.clone()));

    // Process rows
    for (i, proc) in processes.iter().enumerate() {
        let cpu_style = if proc.cpu_percent > 15.0 {
            Style::parse("bold red").unwrap_or_else(|_| Style::null())
        } else if proc.cpu_percent > 8.0 {
            Style::parse("yellow").unwrap_or_else(|_| Style::null())
        } else {
            Style::parse("green").unwrap_or_else(|_| Style::null())
        };

        let state_style = if proc.state == "running" {
            Style::parse("bold green").unwrap_or_else(|_| Style::null())
        } else {
            Style::parse("dim").unwrap_or_else(|_| Style::null())
        };

        // Alternate row dimming
        let row_base = if i % 2 == 1 {
            Style::parse("dim").ok()
        } else {
            None
        };

        // PID
        let pid_str = format!(" {:>6}  ", proc.pid);
        text.append_str(&pid_str, row_base.clone());

        // Command
        let cmd_str = format!("{:<18} ", proc.command);
        text.append_str(&cmd_str, row_base.clone());

        // CPU%
        let cpu_str = format!("{:>5.1}%  ", proc.cpu_percent);
        text.append_str(&cpu_str, Some(cpu_style));

        // Memory
        let mem_str = format!("{:>8}  ", proc.memory_str());
        text.append_str(&mem_str, row_base.clone());

        // Time
        let time_str = format!("{:>10}  ", proc.time_str());
        text.append_str(&time_str, row_base.clone());

        // Threads
        let thr_str = format!("{:>4}  ", proc.thread_count);
        text.append_str(&thr_str, row_base);

        // State
        let state_str = format!("{:<10}\n", proc.state);
        text.append_str(&state_str, Some(state_style));
    }

    // Footer
    text.append_str(
        "\n Press Ctrl+C to exit (auto-exits after ~10 seconds)",
        Some(dim_style),
    );

    text
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let console = Console::builder()
        .force_terminal(true)
        .no_color(false)
        .build();

    let mut rng = SimpleRng::new(42);
    let process_count = 20;

    let initial_processes = generate_processes(&mut rng, process_count);
    let initial_display = build_process_display(&initial_processes);

    let mut live = Live::new(initial_display)
        .with_console(console)
        .with_auto_refresh(false)
        .with_transient(true);

    live.start();

    let start_time = Instant::now();
    let duration = Duration::from_secs(10);

    while start_time.elapsed() < duration {
        let processes = generate_processes(&mut rng, process_count);
        let display = build_process_display(&processes);
        live.update(display, true);
        thread::sleep(Duration::from_millis(500));
    }

    live.stop();

    println!("\nProcess monitor ended.");
}
