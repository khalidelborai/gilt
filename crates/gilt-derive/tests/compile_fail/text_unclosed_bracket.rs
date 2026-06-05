//! text! macro: unclosed `[` must produce a compile error.

fn main() {
    let _t = gilt::text!("[bold");
}
