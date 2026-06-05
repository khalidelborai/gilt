//! text! macro: `[/]` with no open tag must produce a compile error.

fn main() {
    let _t = gilt::text!("foo[/]");
}
