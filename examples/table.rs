//! Demonstrates gilt's Table widget — bordered tables with headers and grid layout.

use gilt::console::Console;
use gilt::rule::Rule;
use gilt::table::Table;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    // -- Star Wars Movies Table -----------------------------------------------

    console.print(&Rule::with_title("Star Wars Movies"));

    let mut table = Table::new(&["Episode", "Title", "Director", "Year", "Box Office"])
        .with_title("Star Wars Saga");

    table.add_row(&["IV", "A New Hope", "George Lucas", "1977", "$775M"]);
    table.add_row(&[
        "V",
        "The Empire Strikes Back",
        "Irvin Kershner",
        "1980",
        "$547M",
    ]);
    table.add_row(&[
        "VI",
        "Return of the Jedi",
        "Richard Marquand",
        "1983",
        "$475M",
    ]);
    table.add_row(&["I", "The Phantom Menace", "George Lucas", "1999", "$1.03B"]);
    table.add_row(&[
        "II",
        "Attack of the Clones",
        "George Lucas",
        "2002",
        "$653M",
    ]);
    table.add_row(&[
        "III",
        "Revenge of the Sith",
        "George Lucas",
        "2005",
        "$868M",
    ]);

    console.print(&table);

    // -- Compact Grid Layout --------------------------------------------------

    console.print(&Rule::with_title("Key-Value Grid"));

    let mut grid = Table::grid(&["Key", "Value"]);
    grid.padding = (0, 2, 0, 0);
    grid.add_row(&["Director:", "George Lucas"]);
    grid.add_row(&["Genre:", "Science Fiction"]);
    grid.add_row(&["Studio:", "Lucasfilm"]);
    grid.add_row(&["Music:", "John Williams"]);

    console.print(&grid);
}
