//! T8: contention bench for `Live` under shared multi-thread updates.
//!
//! Measures aggregate throughput when N writer threads call
//! `update_renderable` against a shared `Arc<Live>` while the main thread
//! drives `refresh()` in a tight loop. The hypothesis (per
//! `.review/V0_11_DESIGN.md` §6) is that the single internal
//! `Mutex<SharedState>` causes writers to serialize even though they
//! mutate disjoint logical state from the renderer.
//!
//! If the throughput-vs-N curve flattens early or regresses past N>1, the
//! contention is real and the v0.10.6 lock-free split (`ArcSwap` for the
//! hot read, `parking_lot::Mutex` for cold config) is justified. If it
//! scales linearly, ship the bench as documentation and skip the rewrite.

use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use gilt::console::ConsoleBuilder;
use gilt::live::Live;
use gilt::style::Style;
use gilt::text::Text;

/// Build a Live configured for benchmarking: no auto-refresh thread (we
/// drive refresh manually), capture-style console (no tty writes).
fn make_live() -> Live {
    // `quiet(true)` swallows writes so the renderer thread doesn't dump
    // ANSI to stdout while the bench runs.
    let console = ConsoleBuilder::new()
        .width(80)
        .height(25)
        .quiet(true)
        .markup(false)
        .no_color(true)
        .force_terminal(true)
        .build();
    Live::new(Text::new("initial", Style::null()))
        .with_console(console)
        .with_auto_refresh(false)
}

/// Spawn `n_writers` threads, each calling `update_renderable` in a tight
/// loop for `iterations` rounds. Returns total elapsed wall time. The
/// payload `Text` is built outside the timed loop so the bench measures
/// the update path itself, not Text construction.
fn run_writers(live: Arc<Live>, n_writers: usize, iterations: usize, payload: Text) -> Duration {
    let barrier = Arc::new(Barrier::new(n_writers + 1));
    let payload = Arc::new(payload);
    let mut handles = Vec::with_capacity(n_writers);

    for _tid in 0..n_writers {
        let live = Arc::clone(&live);
        let barrier = Arc::clone(&barrier);
        let payload = Arc::clone(&payload);
        handles.push(thread::spawn(move || {
            barrier.wait();
            for _ in 0..iterations {
                // refresh=false so writers do not also drive the render
                // path — the bench measures pure update contention.
                live.update_renderable(black_box((*payload).clone()), false);
            }
        }));
    }

    barrier.wait();
    let start = Instant::now();
    for h in handles {
        h.join().unwrap();
    }
    start.elapsed()
}

/// A short payload (~20 chars) — represents Progress label updates.
fn small_payload() -> Text {
    Text::new("downloading file 42", Style::null())
}

/// A larger payload (~2 KB) — represents a fully-rendered table or panel
/// being pushed each frame, which is the realistic Progress workload.
fn large_payload() -> Text {
    let line = "│ task 42 │ 67% ████████░░░░░░ │ ETA 0:00:42 │ 14.2 MB/s │\n";
    Text::new(&line.repeat(32), Style::null())
}

fn bench_update_contention(c: &mut Criterion) {
    const ITERATIONS: usize = 200;

    for (label, payload_fn) in [
        ("small", small_payload as fn() -> Text),
        ("large", large_payload as fn() -> Text),
    ] {
        let mut group = c.benchmark_group(format!("live_threaded/update_only_{label}"));
        for n_writers in [1usize, 2, 4, 8] {
            group.throughput(Throughput::Elements((n_writers * ITERATIONS) as u64));
            group.bench_with_input(
                BenchmarkId::from_parameter(n_writers),
                &n_writers,
                |b, &n| {
                    b.iter_custom(|reps| {
                        let mut total = Duration::ZERO;
                        for _ in 0..reps {
                            let live = Arc::new(make_live());
                            total += run_writers(live, n, ITERATIONS, payload_fn());
                        }
                        total
                    });
                },
            );
        }
        group.finish();
    }
}

/// Same as above but with a renderer thread also calling `refresh()` in a
/// tight loop, simulating the auto-refresh thread that real `Live` users
/// have running. This is the bench that exercises the full contention
/// surface (writer mutex + renderer mutex on the same SharedState lock).
fn bench_update_with_renderer(c: &mut Criterion) {
    let mut group = c.benchmark_group("live_threaded/update_plus_render");
    const ITERATIONS: usize = 200;

    for n_writers in [1usize, 2, 4, 8] {
        group.throughput(Throughput::Elements((n_writers * ITERATIONS) as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(n_writers),
            &n_writers,
            |b, &n| {
                b.iter_custom(|reps| {
                    let mut total = Duration::ZERO;
                    for _ in 0..reps {
                        let live = Arc::new(make_live());
                        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
                        let render_live = Arc::clone(&live);
                        let render_stop = Arc::clone(&stop);
                        let renderer = thread::spawn(move || {
                            while !render_stop.load(std::sync::atomic::Ordering::Relaxed) {
                                render_live.refresh();
                            }
                        });
                        total += run_writers(live, n, ITERATIONS, large_payload());
                        stop.store(true, std::sync::atomic::Ordering::Relaxed);
                        renderer.join().unwrap();
                    }
                    total
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_update_contention, bench_update_with_renderer);
criterion_main!(benches);
