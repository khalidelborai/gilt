//! Live, real-time charts — an animated terminal dashboard.
//!
//! Run it:  cargo run --example live_charts
//!
//! Everything here is a plain `Renderable` swapped into a lock-free [`Live`]
//! display each frame: scrolling `Sparkline`s with min/max markers, an
//! oscillating `BarChart`, and a flowing `Heatmap`, all wrapped in a `Panel`.
//! No event loop, no alternate screen — it scrolls into any terminal.

use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

use gilt::barchart::BarChart;
use gilt::color::Color;
use gilt::console::Console;
use gilt::gradient::Gradient;
use gilt::group::Group;
use gilt::heatmap::Heatmap;
use gilt::live::Live;
use gilt::panel::Panel;
use gilt::sparkline::Sparkline;
use gilt::style::Style;
use gilt::text::Text;

const W: usize = 56; // chart width in cells
const SERVICES: [&str; 5] = ["api", "auth", "search", "cache", "queue"];

/// Build the whole dashboard for the current frame (a single `Renderable`).
fn dashboard(cpu: &[f64], mem: &[f64], reqs: &[f64], net: &[Vec<f64>], frame: usize) -> Panel {
    let mut g = Group::new(vec![]);

    // Gradient banner.
    g.push(Gradient::new(
        "●  gilt · live dashboard",
        vec![
            Color::from_rgb(189, 147, 249),
            Color::from_rgb(255, 121, 198),
            Color::from_rgb(139, 233, 253),
        ],
    ));
    g.push(Text::new("", Style::null()));

    // CPU sparkline (green) with min/max markers — the v2.2 feature on show.
    g.push(
        Text::from_markup(&format!(
            "[bold #50fa7b]CPU   {:>3.0}%[/]   [dim]▼ min  ▲ max[/]",
            cpu.last().copied().unwrap_or(0.0)
        ))
        .unwrap(),
    );
    g.push(
        Sparkline::new(cpu)
            .with_width(W)
            .with_min(0.0)
            .with_max(100.0)
            .with_style(Style::parse("#50fa7b"))
            .with_min_max_markers(true)
            .with_min_style(Style::parse("#6272a4"))
            .with_max_style(Style::parse("bold #ff5555")),
    );

    // MEM sparkline (cyan).
    g.push(Text::new(
        &format!("MEM   {:>3.0}%", mem.last().copied().unwrap_or(0.0)),
        Style::parse("bold #8be9fd"),
    ));
    g.push(
        Sparkline::new(mem)
            .with_width(W)
            .with_min(0.0)
            .with_max(100.0)
            .with_style(Style::parse("#8be9fd"))
            .with_min_max_markers(true)
            .with_min_style(Style::parse("#6272a4"))
            .with_max_style(Style::parse("bold #ffb86c")),
    );
    g.push(Text::new("", Style::null()));

    // Requests/sec bar chart (oscillating), pink bars.
    g.push(Text::new("requests / sec", Style::parse("bold #f8f8f2")));
    let mut bars = BarChart::new()
        .with_width(W)
        .with_max(100.0)
        .with_bar_style(Style::parse("#ff79c6"))
        .with_label_style(Style::parse("#bd93f9"))
        .with_value_style(Style::parse("dim #f8f8f2"));
    for (name, &v) in SERVICES.iter().zip(reqs) {
        bars = bars.with_bar(*name, v);
    }
    g.push(bars);
    g.push(Text::new("", Style::null()));

    // Network heatmap (flowing left), cool→hot gradient.
    g.push(Text::new("network throughput", Style::parse("bold #f8f8f2")));
    g.push(
        Heatmap::new(net.to_vec())
            .with_min(0.0)
            .with_max(1.0)
            .with_cell_width(1)
            .with_gradient(vec![
                Color::from_rgb(40, 42, 54),
                Color::from_rgb(98, 114, 164),
                Color::from_rgb(139, 233, 253),
                Color::from_rgb(241, 250, 140),
                Color::from_rgb(255, 85, 85),
            ]),
    );
    g.push(Text::new("", Style::null()));

    // Footer — brag a little.
    g.push(
        Text::from_markup(&format!(
            "[#50fa7b]●[/] [dim]live · lock-free[/] [bold #bd93f9]ArcSwap[/] [dim]refresh · ~14 fps · frame[/] [bold #f8f8f2]{frame}[/]"
        ))
        .unwrap(),
    );

    Panel::new(g)
        .with_title(Text::new(" gilt · live charts ", Style::parse("bold #282a36 on #bd93f9")))
        .with_subtitle(Text::new("Sparkline · BarChart · Heatmap, animated", Style::parse("italic #6272a4")))
        .with_border_style(Style::parse("bold #bd93f9"))
}

fn main() {
    let frames = 110usize;
    let win = W; // sparkline window length

    // Seeded rolling windows.
    let mut cpu: VecDeque<f64> = VecDeque::from(vec![50.0; win]);
    let mut mem: VecDeque<f64> = VecDeque::from(vec![45.0; win]);
    // Heatmap: 6 rows × 48 cols, flowing.
    let (rows, cols) = (6usize, 48usize);
    let mut net: Vec<Vec<f64>> = vec![vec![0.0; cols]; rows];

    let mut live = Live::new(Text::new("starting live charts…", Style::parse("dim")))
        .with_console(Console::new())
        .with_auto_refresh(false);
    live.start();

    for f in 0..frames {
        let t = f as f64;

        // CPU: smooth wave + a faster ripple, clamped.
        let cpu_v = (52.0 + 40.0 * (t * 0.13).sin() + 7.0 * (t * 0.6).sin()).clamp(2.0, 99.0);
        cpu.push_back(cpu_v);
        if cpu.len() > win {
            cpu.pop_front();
        }
        // MEM: slower drift.
        let mem_v = (50.0 + 30.0 * (t * 0.08 + 1.0).sin() + 5.0 * (t * 0.4).cos()).clamp(2.0, 99.0);
        mem.push_back(mem_v);
        if mem.len() > win {
            mem.pop_front();
        }
        // Requests: per-service oscillation.
        let reqs: Vec<f64> = (0..SERVICES.len())
            .map(|i| {
                let phase = i as f64 * 1.1;
                (55.0 + 42.0 * (t * 0.17 + phase).sin()).clamp(1.0, 100.0)
            })
            .collect();
        // Heatmap: shift each row left, append a new value (row-dependent wave).
        for (r, row) in net.iter_mut().enumerate() {
            row.remove(0);
            let v = 0.5 + 0.5 * ((t * 0.25) + (r as f64) * 0.9).sin();
            row.push(v.clamp(0.0, 1.0));
        }

        let cpu_v: Vec<f64> = cpu.iter().copied().collect();
        let mem_v: Vec<f64> = mem.iter().copied().collect();
        live.set(dashboard(&cpu_v, &mem_v, &reqs, &net, f));
        live.refresh();
        thread::sleep(Duration::from_millis(70));
    }

    live.stop();
    println!("done — that was {frames} frames of pure Renderable, swapped through one Live.");
}
