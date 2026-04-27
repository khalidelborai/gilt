//! v0.11.3: the new `gilt::derives` namespace works alongside top-level
//! re-exports. Both should compile; the namespace form is preferred for
//! the colliding-name derives.

use gilt::derives::{Columns, Inspect, Rule, Table, Tree};

#[derive(Table)]
struct R {
    #[column(header = "X")]
    x: u32,
}

#[derive(Columns)]
struct C {
    name: String,
}

#[derive(Debug, Inspect)]
struct I {
    cpu: f64,
}

#[derive(Rule)]
struct S {
    #[rule(title)]
    heading: String,
}

#[derive(Tree)]
struct T {
    #[tree(label)]
    label: String,
    #[tree(children)]
    children: Vec<T>,
}

fn main() {
    let _ = R::to_table(&[R { x: 1 }]);
    let _ = C::to_columns(&[C { name: "a".into() }]);
    let _ = I { cpu: 1.0 }.to_inspect();
    let _ = S { heading: "h".into() }.to_rule();
    let _ = T { label: "root".into(), children: vec![] }.to_tree();
}
