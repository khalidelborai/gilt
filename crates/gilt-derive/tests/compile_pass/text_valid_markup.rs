//! text! macro — compile-pass tests: many valid markup strings.

use gilt::text;

fn main() {
    // Basic bold/italic
    let _a = text!("[bold]hello world[/bold]");
    let _b = text!("[italic]slanted[/italic]");

    // Combined attributes
    let _c = text!("[bold red]Error:[/] file not found");

    // Nested tags
    let _d = text!("[green]X[blue]Y[/blue]Z[/green]");

    // Implicit close
    let _e = text!("[bold]text[/]");

    // Colors: hex, color(N), rgb(R,G,B)
    let _f = text!("[#ff0000]red hex[/]");
    let _g = text!("[color(200)]indexed[/]");
    let _h = text!("[rgb(255,128,0)]orange[/]");

    // on <color>
    let _i = text!("[on blue]blue background[/]");

    // not <attr>
    let _j = text!("[not bold]normal weight[/]");

    // link
    let _k = text!("[link https://example.com]click[/]");
    let _l = text!("[link=https://rust-lang.org]rust[/]");

    // meta tags
    let _m = text!("[@click]button[/]");
    let _n = text!("[@key=value]annotated[/]");

    // Underline styles
    let _o = text!("[curly]wavy underline[/]");
    let _p = text!("[dotted]dotted[/]");
    let _q = text!("[dashed]dashed[/]");

    // Escaped bracket
    let _r = text!(r"\[not a tag] plain text");

    // Unclosed tag (valid in gilt)
    let _s = text!("[bold]rest of text");

    // No tags
    let _t = text!("plain text only");

    // Empty
    let _u = text!("");

    // Bright colors
    let _v = text!("[bright_blue]bright[/bright_blue]");
    let _w = text!("[on bright_red]error background[/]");

    // default color
    let _x = text!("[default]reset color[/]");
}
