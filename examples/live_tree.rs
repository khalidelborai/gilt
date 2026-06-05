//! Live-updating `Tree` — a build/deploy pipeline that fills in as steps run.
//!
//! Demonstrates v1.5.2 `Live` holding a hierarchical renderable (`Tree`),
//! re-rendered through the live console each frame so guides and labels reflow
//! at the real terminal width.
//!
//! Run with: `cargo run --example live_tree`

use gilt::live::Live;
use gilt::style::Style;
use gilt::text::Text;
use gilt::tree::Tree;
use std::thread;
use std::time::Duration;

/// Build a status label: `done` → green check, `run` → yellow dot, else dim.
fn label(name: &str, state: State) -> Text {
    let (marker, style) = match state {
        State::Done => ("\u{2713}", "green"), // ✓
        State::Run => ("\u{25cf}", "yellow"), // ●
        State::Todo => ("\u{25cb}", "dim"),   // ○
    };
    Text::styled(format!("{marker} {name}"), style)
}

#[derive(Clone, Copy)]
enum State {
    Done,
    Run,
    Todo,
}

fn state_for(completed: usize, index: usize) -> State {
    if completed > index {
        State::Done
    } else if completed == index {
        State::Run
    } else {
        State::Todo
    }
}

fn main() {
    // (group, leaf steps)
    let pipeline: &[(&str, &[&str])] = &[
        ("Build", &["compile", "link", "strip"]),
        ("Test", &["unit", "integration", "doctest"]),
        ("Package", &["bundle", "sign"]),
        ("Deploy", &["upload", "activate"]),
    ];
    let total: usize = pipeline.iter().map(|(_, leaves)| leaves.len()).sum();

    let mut live = Live::from_renderable(Text::new("", Style::null())).with_auto_refresh(false);
    live.start();

    for completed in 0..=total {
        // Rebuild the tree each frame reflecting current progress.
        let mut root =
            Tree::new(Text::styled("pipeline", "bold")).with_guide_style(Style::parse("dim"));

        let mut index = 0;
        for (group, leaves) in pipeline {
            let start = index;
            let end = index + leaves.len();
            let group_state = if completed >= end {
                State::Done
            } else if completed > start {
                State::Run
            } else {
                State::Todo
            };

            let branch = root.add(label(group, group_state));
            for leaf in *leaves {
                branch.add(label(leaf, state_for(completed, index)));
                index += 1;
            }
        }

        live.update_renderable(root, true);
        thread::sleep(Duration::from_millis(250));
    }

    thread::sleep(Duration::from_millis(800));
    live.stop();
}
