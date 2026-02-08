//! Demonstrates gilt's #[derive(Table)] proc macro.
//! Run with: cargo run --example derive_table --features derive

#[cfg(feature = "derive")]
fn main() {
    use gilt::prelude::*;
    use gilt::Table;

    #[derive(Table)]
    struct Employee {
        name: String,
        department: String,
        years: u32,
    }

    let employees = vec![
        Employee { name: "Alice".into(), department: "Engineering".into(), years: 5 },
        Employee { name: "Bob".into(), department: "Marketing".into(), years: 3 },
        Employee { name: "Charlie".into(), department: "Engineering".into(), years: 8 },
        Employee { name: "Diana".into(), department: "Sales".into(), years: 2 },
    ];

    let table = Employee::to_table(&employees);
    let mut console = Console::new();
    console.print(&table);
}

#[cfg(not(feature = "derive"))]
fn main() {
    eprintln!("This example requires the 'derive' feature: cargo run --example derive_table --features derive");
}
