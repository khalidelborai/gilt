//! Table derive — minimal valid input must compile.

use gilt::Table;

#[derive(Table)]
#[table(title = "Rows")]
struct Row {
    #[column(header = "ID")]
    id: u32,
    #[column(header = "Name", style = "bold")]
    name: String,
}

fn main() {
    let _table = Row::to_table(&[Row { id: 1, name: "alice".into() }]);
}
