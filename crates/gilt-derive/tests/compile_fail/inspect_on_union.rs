//! Inspect derive on a union should error gracefully (not panic).

use gilt::DeriveInspect;

#[derive(DeriveInspect)]
union NotSupported {
    a: u32,
    b: f32,
}

fn main() {}
