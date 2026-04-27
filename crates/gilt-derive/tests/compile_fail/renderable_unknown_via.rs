//! Renderable derive with unknown `via = "..."` value should error
//! with a span pointing at the offending literal.
//! Regression guard for v0.11.2's restructuring that eliminated the
//! `.unwrap()` in this code path.

use gilt::Renderable;

#[derive(Renderable)]
#[renderable(via = "made_up_widget")]
struct Bad {
    title: String,
}

fn main() {}
