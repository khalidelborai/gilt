//! Demonstrates the CsvTable widget -- CSV data rendered as a styled table.
//!
//! Run with: `cargo run --example csv_table`
//! For csv crate features: `cargo run --example csv_table --features csv`

use gilt::console::Console;
use gilt::csv_table::CsvTable;
use gilt::rule::Rule;
use gilt::style::Style;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    // -- Basic CSV --------------------------------------------------------
    console.print(&Rule::with_title("Basic CSV Data"));

    let csv_data = "\
Name,Age,City,Role
Alice,30,New York,Engineer
Bob,25,San Francisco,Designer
Carol,35,Chicago,Manager
Dave,28,Seattle,DevOps";

    let csv = CsvTable::from_csv_str(csv_data)
        .unwrap()
        .with_title("Team Directory")
        .with_header_style(Style::parse("bold cyan").unwrap());

    console.print(&csv);

    // -- CSV with quoted fields -------------------------------------------
    console.print(&Rule::with_title("CSV with Quoted Fields"));

    let quoted_data = "\
Product,Description,Price
Widget,\"A small, handy device\",9.99
Gadget,\"The latest \"\"must-have\"\" item\",29.99
Doohickey,\"Multi-purpose tool, v2.0\",14.50";

    let quoted = CsvTable::from_csv_str(quoted_data)
        .unwrap()
        .with_title("Product Catalog");

    console.print(&quoted);

    // -- Max rows limit ---------------------------------------------------
    console.print(&Rule::with_title("Max Rows (showing 3 of 5)"));

    let many_rows = "\
ID,Value,Status
1,100,Active
2,200,Pending
3,300,Active
4,400,Closed
5,500,Active";

    let limited = CsvTable::from_csv_str(many_rows)
        .unwrap()
        .with_max_rows(3)
        .with_title("Limited View");

    console.print(&limited);

    // -- Single column ----------------------------------------------------
    console.print(&Rule::with_title("Single Column CSV"));

    let single_col = "\
Language
Rust
Python
TypeScript
Go";

    let single = CsvTable::from_csv_str(single_col)
        .unwrap()
        .with_header_style(Style::parse("bold magenta").unwrap());

    console.print(&single);

    // -- Display trait ----------------------------------------------------
    console.print(&Rule::with_title("Display Trait (println!)"));

    let small = CsvTable::from_csv_str("X,Y\n1,2\n3,4").unwrap();
    println!("{}", small);
}
