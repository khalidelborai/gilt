# gilt-derive

[![Crates.io](https://img.shields.io/crates/v/gilt-derive.svg)](https://crates.io/crates/gilt-derive)
[![Documentation](https://docs.rs/gilt-derive/badge.svg)](https://docs.rs/gilt-derive)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

Procedural macros for the [gilt](https://crates.io/crates/gilt) terminal
formatting library.

This crate is not useful standalone — enable it via the `derive` feature
on `gilt`:

```toml
[dependencies]
gilt = { version = "0.11", features = ["derive"] }
```

## Provided derives

| Derive          | Generates                                          | Use case                                       |
|-----------------|----------------------------------------------------|------------------------------------------------|
| `Table`         | `to_table(items: &[Self]) -> gilt::table::Table`   | Render a slice of structs as a styled table    |
| `Panel`         | `to_panel(&self) -> gilt::panel::Panel`            | Render a struct as a labelled panel            |
| `Tree`          | `to_tree(&self) -> gilt::tree::Tree`               | Render a struct as a hierarchical tree         |
| `Columns`       | `to_columns(items: &[Self]) -> gilt::columns::Columns` | Multi-column flow layout                  |
| `Rule`          | `to_rule(&self) -> gilt::rule::Rule`               | Section divider with title                     |
| `Inspect`       | `to_inspect(&self) -> impl Renderable`             | Debug-style introspection widget               |
| `Renderable`    | `impl gilt::console::Renderable for Self`          | Wire any struct into the rendering pipeline    |

## Quick example

```rust
use gilt::Table;

#[derive(Table)]
#[table(title = "Employees", box_style = "ROUNDED", header_style = "bold cyan")]
struct Employee {
    #[column(header = "Full Name", style = "bold")]
    name: String,
    #[column(justify = "right")]
    age: u32,
    #[column(skip)]
    internal_id: u64,
}

let employees = vec![/* ... */];
let table = Employee::to_table(&employees);
```

See [docs.rs/gilt-derive](https://docs.rs/gilt-derive) for the full
attribute schemas and [examples in the main repo](https://github.com/khalidelborai/gilt/tree/main/examples)
(`derive_*.rs`).

## Robustness

As of v0.11.2, all gilt-derive macros emit `compile_error!` (never panic)
on malformed input. Errors carry source spans so IDEs highlight the
offending attribute, not the derive macro itself.

## License

MIT — see [LICENSE](../../LICENSE).
