//! Demonstrates gilt's segment pipeline — the atomic rendering unit.

use gilt::cells::cell_len;
use gilt::segment::Segment;
use gilt::style::Style;

fn main() {
    println!("=== Segment Basics ===\n");

    let seg = Segment::styled("Hello, World!", Style::parse("bold green").unwrap());
    println!("  text:        {:?}", seg.text);
    println!("  cell_length: {}", seg.cell_length());
    println!("  is_control:  {}", seg.is_control());

    println!("\n=== CJK Cell Width ===\n");

    let texts = ["Hello", "こんにちは", "Hello💩World", "├──┤"];
    for text in texts {
        println!("  {:>15} → {} cells", text, cell_len(text));
    }

    println!("\n=== Split Cells (double-width aware) ===\n");

    let seg = Segment::text("AB💩CD");
    println!("  original: {:?} ({} cells)", seg.text, seg.cell_length());
    for cut in 0..=seg.cell_length() {
        let (left, right) = seg.split_cells(cut);
        println!(
            "  cut@{}: {:?}({}) | {:?}({})",
            cut,
            left.text,
            left.cell_length(),
            right.text,
            right.cell_length()
        );
    }

    println!("\n=== Split Lines ===\n");

    let segments = vec![
        Segment::styled("Hello\n", Style::parse("bold").unwrap()),
        Segment::styled("World!", Style::parse("italic").unwrap()),
    ];
    let lines = Segment::split_lines(&segments);
    for (i, line) in lines.iter().enumerate() {
        let texts: Vec<&str> = line.iter().map(|s| s.text.as_str()).collect();
        println!("  line {}: {:?}", i, texts);
    }

    println!("\n=== Adjust Line Length ===\n");

    let line = vec![Segment::text("Hi")];
    let padded = Segment::adjust_line_length(&line, 10, &Style::null(), true);
    println!(
        "  pad 'Hi' to 10: {:?}",
        padded.iter().map(|s| &s.text).collect::<Vec<_>>()
    );

    let line = vec![Segment::text("Hello, World!")];
    let cropped = Segment::adjust_line_length(&line, 5, &Style::null(), false);
    println!(
        "  crop to 5:      {:?}",
        cropped.iter().map(|s| &s.text).collect::<Vec<_>>()
    );

    println!("\n=== Simplify (merge adjacent) ===\n");

    let segments = vec![
        Segment::text("He"),
        Segment::text("llo"),
        Segment::text(" "),
        Segment::text("World"),
    ];
    let simplified = Segment::simplify(&segments);
    println!(
        "  {} segments → {} segment: {:?}",
        segments.len(),
        simplified.len(),
        simplified[0].text
    );

    println!("\n=== Divide at Positions ===\n");

    let bold = Style::parse("bold").unwrap();
    let italic = Style::parse("italic").unwrap();
    let segments = vec![
        Segment::styled("Hello", bold),
        Segment::styled(" World!", italic),
    ];
    let divisions = Segment::divide(&segments, &[3, 8]);
    for (i, div) in divisions.iter().enumerate() {
        let texts: Vec<&str> = div.iter().map(|s| s.text.as_str()).collect();
        println!("  part {}: {:?}", i, texts);
    }

    println!("\n=== Shape ===\n");

    let lines = vec![
        vec![Segment::text("short")],
        vec![Segment::text("a longer line")],
        vec![Segment::text("mid")],
    ];
    let (width, height) = Segment::get_shape(&lines);
    println!("  shape: {}×{}", width, height);
}
