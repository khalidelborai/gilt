//! Demonstrates gilt's Table widget — bordered tables with headers.

use gilt::console::Console;
use gilt::table::Table;

fn main() {
    let mut table = Table::new(&["Released", "Title", "Box Office"]).with_title("Star Wars Movies");
    table.add_row(&[
        "Dec 20, 2019",
        "Star Wars: The Rise of Skywalker",
        "$952,110,690",
    ]);
    table.add_row(&["May 25, 2018", "Solo: A Star Wars Story", "$393,151,347"]);
    table.add_row(&[
        "Dec 15, 2017",
        "Star Wars Ep. VIII: The Last Jedi",
        "$1,332,539,889",
    ]);
    table.add_row(&[
        "Dec 16, 2016",
        "Rogue One: A Star Wars Story",
        "$1,332,439,889",
    ]);
    Console::default().print(&table);
}
