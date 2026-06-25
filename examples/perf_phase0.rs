//! Phase 0 of the gilt 3.0 structural-refactor decision.
//!
//! See `.review/v3-directions-2026-06-25.md` and
//! `.review/v2-structural-decision-2026-06-06.md`. The ADR deferred the
//! SegmentBuf / StyleId / SoA refactor behind a benchmark-gated trigger:
//! implement it only when a realistic workload shows `Vec<Segment>` allocation
//! or `Style` copying is a *double-digit-percent* share of a large render.
//!
//! This harness measures that with a counting global allocator + timing:
//!   1. current type sizes (does the ADR baseline still hold after 2.0?)
//!   2. allocation profile of a large render (segment storage as % of memory)
//!   3. the ADR's named case: a giant recorded buffer + export
//!   4. copy isolation: clone Vec<Style> vs Vec<u32> (the interner's copy win)
//!
//! Run: cargo run --release --example perf_phase0 --all-features

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::time::{Duration, Instant};

use gilt::console::Console;
use gilt::segment::{ControlCode, Segment};
use gilt::style::Style;
use gilt::table::Table;
use gilt::text::Span;

// --- counting global allocator --------------------------------------------

struct Counting;
static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = System.alloc(layout);
        if !p.is_null() {
            ALLOCS.fetch_add(1, Relaxed);
            BYTES.fetch_add(layout.size(), Relaxed);
            let live = LIVE.fetch_add(layout.size(), Relaxed) + layout.size();
            let mut peak = PEAK.load(Relaxed);
            while live > peak {
                match PEAK.compare_exchange_weak(peak, live, Relaxed, Relaxed) {
                    Ok(_) => break,
                    Err(p) => peak = p,
                }
            }
        }
        p
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        LIVE.fetch_sub(layout.size(), Relaxed);
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

#[derive(Clone, Copy)]
struct Stats {
    dt: Duration,
    allocs: usize,
    bytes: usize,
    peak: usize,
}

/// Measure allocations + peak-live + time for one operation.
fn measure<T>(f: impl FnOnce() -> T) -> (T, Stats) {
    let live0 = LIVE.load(Relaxed);
    ALLOCS.store(0, Relaxed);
    BYTES.store(0, Relaxed);
    PEAK.store(live0, Relaxed);
    let t = Instant::now();
    let out = f();
    let dt = t.elapsed();
    let stats = Stats {
        dt,
        allocs: ALLOCS.load(Relaxed),
        bytes: BYTES.load(Relaxed),
        peak: PEAK.load(Relaxed).saturating_sub(live0),
    };
    (out, stats)
}

fn kb(n: usize) -> String {
    format!("{:.1} KiB", n as f64 / 1024.0)
}

fn big_table(rows: usize) -> Table {
    let mut t = Table::new(&["ID", "Name", "Score", "Status", "Note"]);
    for i in 0..rows {
        let id = i.to_string();
        let name = format!("user_{i}");
        let score = format!("{}", (i * 7) % 100);
        let status = if i % 2 == 0 {
            "[green]ok[/]"
        } else {
            "[red]fail[/]"
        };
        t.add_row(&[&id, &name, &score, status, "lorem ipsum dolor sit amet"]);
    }
    t
}

fn main() {
    println!("=== gilt 3.0 Phase 0 — structural-refactor trigger measurement ===\n");

    // -- 1. type sizes ------------------------------------------------------
    println!("[1] Type sizes (current 2.0 codebase)");
    let seg = std::mem::size_of::<Segment>();
    let sty = std::mem::size_of::<Style>();
    println!("    Segment                  = {seg} B");
    println!("    Style                    = {sty} B");
    println!(
        "    Option<Style>            = {} B",
        std::mem::size_of::<Option<Style>>()
    );
    println!(
        "    Span                     = {} B",
        std::mem::size_of::<Span>()
    );
    println!(
        "    ControlCode              = {} B",
        std::mem::size_of::<ControlCode>()
    );
    println!(
        "    Option<Vec<ControlCode>> = {} B",
        std::mem::size_of::<Option<Vec<ControlCode>>>()
    );
    let seg_with_styleid = seg - sty + 4; // replace inline Style with a u32 id
    println!(
        "    -> hypothetical Segment with StyleId(u32): ~{seg_with_styleid} B ({:.0}% smaller)\n",
        100.0 * (seg - seg_with_styleid) as f64 / seg as f64
    );

    // -- 2. large render allocation profile ---------------------------------
    println!("[2] Large render: console.render(&2000-row table)");
    let console = Console::builder().width(100).force_terminal(true).build();
    let table = big_table(2000);
    // warm any lazy statics so they don't pollute the measured region
    let _ = black_box(console.render(&Table::new(&["x"]), None));
    let (segs, s) = measure(|| console.render(black_box(&table), None));
    let n = segs.len();
    let seg_storage = n * seg;
    println!(
        "    time={:?}  allocs={}  alloc_bytes={}  peak_live={}",
        s.dt,
        s.allocs,
        kb(s.bytes),
        kb(s.peak)
    );
    println!(
        "    segments={n}  segment storage = {} = {:.1}% of peak_live, {:.1}% of alloc_bytes",
        kb(seg_storage),
        100.0 * seg_storage as f64 / s.peak.max(1) as f64,
        100.0 * seg_storage as f64 / s.bytes.max(1) as f64,
    );
    let style_savings = n * (seg - seg_with_styleid);
    println!(
        "    StyleId would shrink segment storage by {} ({:.1}% of peak_live)\n",
        kb(style_savings),
        100.0 * style_savings as f64 / s.peak.max(1) as f64
    );

    // -- 3. ADR's named case: giant recorded buffer + export ----------------
    println!("[3] Giant recorded buffer (the ADR's memory-pressure case): print 5000-row table, then export_html");
    let mut rec = Console::builder()
        .width(100)
        .record(true)
        .force_terminal(true)
        .build();
    let big = big_table(5000);
    let n5 = console.render(&big, None).len();
    // begin_capture redirects output away from stdout while record(true) still
    // populates the record buffer — so we measure a real retained buffer + export
    // without dumping 5000 rows to the terminal.
    rec.begin_capture();
    let (_, s_print) = measure(|| rec.print(black_box(&big)));
    let _captured = rec.end_capture();
    let (html, s_html) = measure(|| rec.export_html(None, false, true));
    let rec_seg_storage = n5 * seg;
    println!(
        "    print:  time={:?}  allocs={}  peak_live={}  (record buffer ≈ {} segments)",
        s_print.dt,
        s_print.allocs,
        kb(s_print.peak),
        n5
    );
    println!(
        "    segment storage in record buffer = {} = {:.1}% of print peak_live",
        kb(rec_seg_storage),
        100.0 * rec_seg_storage as f64 / s_print.peak.max(1) as f64
    );
    println!(
        "    export_html: time={:?}  allocs={}  alloc_bytes={}  -> html {} bytes\n",
        s_html.dt,
        s_html.allocs,
        kb(s_html.bytes),
        html.len()
    );

    // -- 4. copy isolation: Style vs StyleId --------------------------------
    println!(
        "[4] Copy isolation: clone Vec<Style> vs Vec<u32> (the interner's per-element copy win)"
    );
    let count = 500_000usize;
    let styles: Vec<Style> = std::iter::repeat_with(|| Style::parse("bold red"))
        .take(count)
        .collect();
    let ids: Vec<u32> = (0..count as u32).collect();
    let (c1, s_style) = measure(|| black_box(styles.clone()));
    let (c2, s_id) = measure(|| black_box(ids.clone()));
    let (c3, s_seg) = measure(|| black_box(segs.clone()));
    println!(
        "    clone {count} Style : time={:?}  alloc_bytes={}",
        s_style.dt,
        kb(s_style.bytes)
    );
    println!(
        "    clone {count} u32   : time={:?}  alloc_bytes={}",
        s_id.dt,
        kb(s_id.bytes)
    );
    let copy_win = s_style.dt.saturating_sub(s_id.dt);
    println!(
        "    -> copy win from interning the style field ≈ {:?} over {count} elements",
        copy_win
    );
    println!(
        "    clone {} rendered Segment (incl. text): time={:?}  alloc_bytes={}",
        n,
        s_seg.dt,
        kb(s_seg.bytes)
    );
    // express the copy win as % of render time, scaled to the render's segment count
    let per_elem_win_ns = copy_win.as_nanos() as f64 / count as f64;
    let render_scaled_win_ns = per_elem_win_ns * n as f64;
    println!(
        "    -> at the 2000-row render's {n} segments, the style-copy win ≈ {:.1} µs = {:.2}% of the {:?} render\n",
        render_scaled_win_ns / 1000.0,
        100.0 * render_scaled_win_ns / s.dt.as_nanos().max(1) as f64,
        s.dt
    );

    black_box((c1, c2, c3, segs, html));

    // -- verdict ------------------------------------------------------------
    println!("=== TRIGGER CHECK (ADR threshold: double-digit % share) ===");
    let mem_share = 100.0 * seg_storage as f64 / s.peak.max(1) as f64;
    let rec_share = 100.0 * rec_seg_storage as f64 / s_print.peak.max(1) as f64;
    let cpu_share = 100.0 * render_scaled_win_ns / s.dt.as_nanos().max(1) as f64;
    println!("    segment storage share of render memory   : {mem_share:.1}%");
    println!("    segment storage share of record buffer   : {rec_share:.1}%");
    println!("    style-copy CPU share of render time       : {cpu_share:.2}%");
    let fired = mem_share >= 10.0 || rec_share >= 10.0 || cpu_share >= 10.0;
    println!(
        "    => TRIGGER {}",
        if fired {
            "FIRES — structural refactor is justified"
        } else {
            "does NOT fire — keep the refactor deferred"
        }
    );
}
