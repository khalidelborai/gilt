//! Inspect derive on a union should error gracefully (not panic).

use gilt::derives::Inspect;

#[derive(Inspect)]
union NotSupported {
    a: u32,
    b: f32,
}

fn main() {}
