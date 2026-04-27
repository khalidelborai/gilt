//! Table derive on an enum should produce an actionable error
//! (not a panic from the proc-macro).

use gilt::Table;

#[derive(Table)]
enum NotAStruct {
    A,
    B,
}

fn main() {}
