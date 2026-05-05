//! Idiomatic-gilt counterpart to `top_lite_simulator.rs`.
//!
//! Same simulator, but built with the `Table` widget instead of hand-formatted
//! `Text` rows. Uses v1.0 `Live::from_renderable` to live-update a non-Text
//! widget without the manual capture roundtrip.
//!
//! Run with: `cargo run --example top_lite_table`

use gilt::live::Live;
use gilt::style::Style;
use gilt::table::{ColumnOptions, Table};
use gilt::text::{JustifyMethod, Text};
use std::thread;
use std::time::{Duration, Instant};

struct SimpleRng(u64);
impl SimpleRng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn f(&mut self) -> f64 {
        self.next_u64() as f64 / u32::MAX as f64
    }
    fn r(&mut self, lo: u64, hi: u64) -> u64 {
        lo + self.next_u64() % (hi - lo + 1)
    }
}

const COMMANDS: &[&str] = &[
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
];

fn build_table(rng: &mut SimpleRng) -> Table {
    let mut t = Table::new(&[]);
    t.title = Some("Top Lite (gilt::Table)".into());
    t.title_style = "bold cyan".into();
    t.header_style = "bold white on blue".into();
    let right = ColumnOptions {
        justify: Some(JustifyMethod::Right),
        ..Default::default()
    };
    let left_min = ColumnOptions {
        min_width: Some(18),
        ..Default::default()
    };
    t.add_column("PID", "", right.clone());
    t.add_column("Command", "", left_min);
    t.add_column("CPU %", "", right.clone());
    t.add_column("Memory", "", right.clone());
    t.add_column("THR", "", right);
    t.add_column("State", "", ColumnOptions::default());

    let mut procs: Vec<(u32, &str, f64, u64, u32, &str)> = (0..20)
        .map(|i| {
            let cpu = rng.f() * 25.0;
            let mem = rng.r(10, 200).pow(3);
            let cmd = COMMANDS[(rng.r(0, COMMANDS.len() as u64 - 1)) as usize];
            let state = if rng.r(0, 10) < 8 {
                "running"
            } else {
                "sleeping"
            };
            (
                (i as u32 + 1) * 100 + rng.r(1, 99) as u32,
                cmd,
                cpu,
                mem,
                rng.r(1, 32) as u32,
                state,
            )
        })
        .collect();
    procs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

    for (pid, cmd, cpu, mem, thr, state) in procs {
        let cpu_style = if cpu > 15.0 {
            "bold red"
        } else if cpu > 8.0 {
            "yellow"
        } else {
            "green"
        };
        let state_style = if state == "running" {
            "bold green"
        } else {
            "dim"
        };
        let mem_str = if mem > 1_000_000 {
            format!("{}M", mem / 1_000_000)
        } else if mem > 1_000 {
            format!("{}K", mem / 1_000)
        } else {
            format!("{mem}B")
        };
        t.add_row_text(&[
            Text::from(pid.to_string()),
            Text::from(cmd.to_string()),
            Text::styled_with(format!("{cpu:.1}%"), Style::parse(cpu_style)),
            Text::from(mem_str),
            Text::from(thr.to_string()),
            Text::styled_with(state, Style::parse(state_style)),
        ]);
    }
    t
}

fn main() {
    let mut rng = SimpleRng(42);
    let live = Live::from_renderable(&build_table(&mut rng))
        .with_auto_refresh(false)
        .with_transient(true);
    let mut live = live;
    live.start();

    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(10) {
        live.set_renderable_widget(&build_table(&mut rng));
        thread::sleep(Duration::from_millis(500));
    }
    drop(live);
    println!("\nProcess monitor ended.");
}
