//! Demonstrates human-readable file size formatting with SI (decimal) and IEC (binary) units.

use gilt::console::Console;
use gilt::filesize;
use gilt::prelude::*;

fn main() {
    let mut console = Console::builder()
        .width(90)
        .force_terminal(true)
        .no_color(false)
        .build();

    // -- Side-by-side comparison table ----------------------------------------

    console.print(&Rule::with_title("File Size Formatting"));

    let mut table = Table::new(&["Bytes", "Decimal (SI)", "Binary (IEC)"]);
    table.title = Some("Decimal vs Binary".to_string());

    let sizes: &[u64] = &[
        0,
        1,
        999,
        1_000,
        1_023,
        1_024,
        1_000_000,
        1_048_576,
        1_000_000_000,
        1_073_741_824,
        1_000_000_000_000,
    ];

    for &size in sizes {
        table.add_row(&[
            &format_bytes_column(size),
            &filesize::decimal(size, 1, " "),
            &filesize::binary(size, 1, " "),
        ]);
    }

    console.print(&table);

    // -- Precision comparison -------------------------------------------------

    console.print(&Rule::with_title("Precision Comparison"));

    let sample_size: u64 = 1_536_000; // ~1.5 MB / ~1.46 MiB
    let mut prec_table = Table::new(&["Precision", "Decimal (SI)", "Binary (IEC)"]);
    prec_table.title = Some(format!(
        "Sample: {} bytes",
        format_with_separator(sample_size)
    ));

    for precision in 0..=3 {
        prec_table.add_row(&[
            &format!("{}", precision),
            &filesize::decimal(sample_size, precision, " "),
            &filesize::binary(sample_size, precision, " "),
        ]);
    }

    console.print(&prec_table);

    // -- Separator styles -----------------------------------------------------

    console.print(&Rule::with_title("Separator Styles"));

    let demo_size: u64 = 1_500_000;
    let mut sep_table = Table::new(&["Separator", "Decimal (SI)", "Binary (IEC)"]);

    let separators: &[(&str, &str)] = &[
        ("Space (\" \")", " "),
        ("Empty (\"\")", ""),
        ("Dash (\"-\")", "-"),
    ];

    for (label, sep) in separators {
        sep_table.add_row(&[
            label,
            &filesize::decimal(demo_size, 1, sep),
            &filesize::binary(demo_size, 1, sep),
        ]);
    }

    console.print(&sep_table);
}

/// Format a byte count with thousands separators for the "Bytes" column.
fn format_with_separator(n: u64) -> String {
    let s = n.to_string();
    let len = s.len();
    if len <= 3 {
        return s;
    }
    let mut result = String::with_capacity(len + (len - 1) / 3);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result
}

fn format_bytes_column(n: u64) -> String {
    format_with_separator(n)
}
