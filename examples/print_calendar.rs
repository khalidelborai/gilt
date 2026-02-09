//! Builds a full-year calendar layout using Tables and Columns.
//!
//! Port of Python rich's `print_calendar.py`. Renders 12 month tables
//! arranged in columns, with weekend days in blue and today highlighted.
//!
//! Usage:
//!     cargo run --example print_calendar [YEAR]
//! Example:
//!     cargo run --example print_calendar 2026

use gilt::box_chars::SIMPLE_HEAVY;
use gilt::columns::Columns;
use gilt::console::Console;
use gilt::style::Style;
use gilt::table::{ColumnOptions, Table};
use gilt::text::{JustifyMethod, Text};

// ---------------------------------------------------------------------------
// Date calculation helpers (no external crate)
// ---------------------------------------------------------------------------

/// Returns true if `year` is a leap year.
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Number of days in a given month (1-based).
fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 => 31,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        3 => 31,
        4 => 30,
        5 => 31,
        6 => 30,
        7 => 31,
        8 => 31,
        9 => 30,
        10 => 31,
        11 => 30,
        12 => 31,
        _ => 0,
    }
}

/// Day of week using Tomohiko Sakamoto's algorithm.
/// Returns 0 = Monday, 1 = Tuesday, ..., 6 = Sunday.
fn day_of_week(year: i32, month: u32, day: u32) -> u32 {
    // Sakamoto's algorithm returns 0=Sunday..6=Saturday
    let t = [0_i32, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if month < 3 { year - 1 } else { year };
    let dow = (y + y / 4 - y / 100 + y / 400 + t[(month - 1) as usize] + day as i32) % 7;
    // Convert: 0=Sun -> 6, 1=Mon -> 0, 2=Tue -> 1, ...
    ((dow + 6) % 7) as u32
}

/// Build a calendar grid for a month: each inner Vec is a week (Mon..Sun),
/// with 0 meaning "no day" in that slot.
fn month_weeks(year: i32, month: u32) -> Vec<[u32; 7]> {
    let num_days = days_in_month(year, month);
    let first_dow = day_of_week(year, month, 1); // 0=Mon
    let mut weeks: Vec<[u32; 7]> = Vec::new();
    let mut week = [0u32; 7];
    let mut col = first_dow as usize;

    for day in 1..=num_days {
        week[col] = day;
        col += 1;
        if col == 7 {
            weeks.push(week);
            week = [0u32; 7];
            col = 0;
        }
    }
    // Push final partial week if any
    if col > 0 {
        weeks.push(week);
    }
    weeks
}

/// Month name from 1-based index.
fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "",
    }
}

// ---------------------------------------------------------------------------
// Get today's date from the system clock (no chrono)
// ---------------------------------------------------------------------------

/// Returns (year, month, day) for today using libc's localtime.
fn today() -> (i32, u32, u32) {
    // Fallback: default to 2026-02-09 if something goes wrong.
    // We use std::time to get epoch seconds, then do simple date math.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    // Civil date from Unix timestamp (algorithm from Howard Hinnant)
    let z = secs / 86400 + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    (year as i32, m, d)
}

// ---------------------------------------------------------------------------
// Calendar rendering
// ---------------------------------------------------------------------------

/// Build a Table for one month of the calendar.
fn build_month_table(
    year: i32,
    month: u32,
    today_day: u32,
    today_month: u32,
    today_year: i32,
) -> Table {
    let day_names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

    let mut table = Table::new(&[]);
    table.title = Some(format!("{} {}", month_name(month), year));
    table.style = "green".to_string();
    table.box_chars = Some(&SIMPLE_HEAVY);
    table.padding = (0, 0, 0, 0);

    // Add 7 columns (Mon-Sun), right-justified
    for &day_name in &day_names {
        table.add_column(
            day_name,
            "",
            ColumnOptions {
                justify: Some(JustifyMethod::Right),
                ..Default::default()
            },
        );
    }

    let weeks = month_weeks(year, month);
    for week in &weeks {
        let mut cells: Vec<Text> = Vec::with_capacity(7);
        for (col, &day) in week.iter().enumerate() {
            let label = if day == 0 {
                String::new()
            } else {
                format!("{:>2}", day)
            };

            let mut style = Style::parse("magenta").unwrap_or_else(|_| Style::null());

            // Weekend days (Sat=5, Sun=6) in blue
            if col >= 5 {
                style = Style::parse("blue").unwrap_or_else(|_| Style::null());
            }

            // Highlight today
            if day > 0 && day == today_day && month == today_month && year == today_year {
                style = Style::parse("white on dark_red").unwrap_or_else(|_| Style::null());
            }

            cells.push(Text::new(&label, style));
        }
        table.add_row_text(&cells);
    }

    table
}

fn print_calendar(year: i32) {
    let mut console = Console::builder()
        .force_terminal(true)
        .no_color(false)
        .build();

    let (today_year, today_month, today_day) = today();

    // Build all 12 month tables, render each to a plain string for Columns
    let mut rendered_months: Vec<String> = Vec::new();
    for month in 1..=12 {
        let table = build_month_table(year, month, today_day, today_month, today_year);
        // Capture the table's rendered output as a string
        let mut capture_console = Console::builder()
            .width(24)
            .force_terminal(true)
            .no_color(true)
            .markup(false)
            .build();
        capture_console.begin_capture();
        capture_console.print(&table);
        let output = capture_console.end_capture();
        rendered_months.push(output);
    }

    // Arrange months in Columns
    let mut columns = Columns::new().with_padding((1, 1, 1, 1)).with_expand(true);
    for rendered in &rendered_months {
        columns.add_renderable(rendered);
    }

    // Print with rules
    console.rule(Some(&format!("{}", year)));
    console.line(1);
    console.print(&columns);
    console.rule(Some(&format!("{}", year)));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let year = if args.len() > 1 {
        args[1].parse::<i32>().unwrap_or(2026)
    } else {
        let (y, _, _) = today();
        y
    };

    print_calendar(year);
}
