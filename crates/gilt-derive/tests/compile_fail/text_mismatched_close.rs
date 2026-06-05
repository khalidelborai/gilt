//! text! macro: `[/italic]` when only `[bold]` is open must produce a compile error.

fn main() {
    let _t = gilt::text!("[bold]x[/italic]");
}
