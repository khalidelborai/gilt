//! text! macro: unknown style token `blod` must produce a compile error.

fn main() {
    let _t = gilt::text!("[blod]text[/]");
}
