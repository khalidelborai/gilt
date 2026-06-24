//! Renderable and Display implementations for Table.

use crate::console::{Console, ConsoleOptions, ConsoleOptionsUpdates, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;
use crate::widgets::table::Table;

impl Renderable for Table {
    fn gilt_measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        self.measure(console, options)
    }

    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        if self.columns.is_empty() {
            return vec![Segment::line()];
        }

        let mut max_width = options.max_width;
        if let Some(w) = self.width {
            max_width = w;
        }

        let extra_width = self.extra_width();
        // T9: build column cells once, share them between the
        // measurement (calculate_column_widths) and render (render_table)
        // passes. Was previously rebuilding via get_cells in each path.
        let column_cells: Vec<Vec<crate::widgets::table::core::CellInfo>> = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| self.get_cells(console, i, col))
            .collect();
        let widths = self.calculate_column_widths_with_cells(
            console,
            &options.update_width(max_width.saturating_sub(extra_width)),
            &column_cells,
        );
        let table_width: usize = widths.iter().sum::<usize>() + extra_width;

        let render_options = options.with_updates(&ConsoleOptionsUpdates {
            width: Some(table_width),
            highlight: Some(Some(self.highlight)),
            height: Some(None),
            ..Default::default()
        });

        let mut segments = Vec::new();

        // Title
        if let Some(ref title) = self.title {
            let title_style_str = if self.title_style.is_empty() {
                "table.title"
            } else {
                &self.title_style
            };
            let title_style = console
                .get_style(title_style_str)
                .unwrap_or_else(|_| Style::null());
            let mut title_text =
                console.render_str(title, Some(&title_style.to_string()), None, None);
            title_text.justify = Some(self.title_justify);

            let title_opts = render_options.with_updates(&ConsoleOptionsUpdates {
                justify: Some(Some(self.title_justify)),
                ..Default::default()
            });

            let title_segs = title_text.gilt_console(console, &title_opts);
            segments.extend(title_segs);
            // Ensure title ends with a newline
            if segments
                .last()
                .map(|s| !s.text.ends_with('\n'))
                .unwrap_or(false)
            {
                segments.push(Segment::line());
            }
        }

        // Render table body — pass the cached cells so render_table doesn't
        // rebuild them.
        segments.extend(self.render_table_with_cells(
            console,
            &render_options,
            &widths,
            column_cells,
        ));

        // Caption
        if let Some(ref caption) = self.caption {
            let caption_style_str = if self.caption_style.is_empty() {
                "table.caption"
            } else {
                &self.caption_style
            };
            let caption_style = console
                .get_style(caption_style_str)
                .unwrap_or_else(|_| Style::null());
            let mut caption_text =
                console.render_str(caption, Some(&caption_style.to_string()), None, None);
            caption_text.justify = Some(self.caption_justify);

            let caption_opts = render_options.with_updates(&ConsoleOptionsUpdates {
                justify: Some(Some(self.caption_justify)),
                ..Default::default()
            });

            let caption_segs = caption_text.gilt_console(console, &caption_opts);
            segments.extend(caption_segs);
            if segments
                .last()
                .map(|s| !s.text.ends_with('\n'))
                .unwrap_or(false)
            {
                segments.push(Segment::line());
            }
        }

        segments
    }
}

impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut console = Console::builder()
            .width(f.width().unwrap_or(80))
            .force_terminal(true)
            .no_color(true)
            .build();
        console.begin_capture();
        console.print(self);
        let output = console.end_capture();
        write!(f, "{}", output.trim_end_matches('\n'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::table::Table;

    fn make_console(width: usize) -> Console {
        Console::builder()
            .width(width)
            .force_terminal(true)
            .no_color(true)
            .markup(false)
            .build()
    }

    #[test]
    fn table_gilt_measure_matches_standalone() {
        let console = make_console(80);
        let opts = console.options();
        let mut table = Table::new(&["Name", "Age"]);
        table.add_row(&["Alice", "30"]);
        assert_eq!(
            table.gilt_measure(&console, &opts),
            table.measure(&console, &opts),
            "Table::gilt_measure must delegate to Table::measure"
        );
    }

    #[test]
    fn table_gilt_measure_empty_matches_standalone() {
        let console = make_console(80);
        let opts = console.options();
        let table = Table::new(&[]);
        assert_eq!(
            table.gilt_measure(&console, &opts),
            table.measure(&console, &opts),
            "Table::gilt_measure empty table must delegate to Table::measure"
        );
    }
}
