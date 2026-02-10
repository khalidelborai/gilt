//! Align widget demonstration
//!
//! Demonstrates gilt's Align widget for positioning content horizontally
//! and vertically within available console space.
//!
//! Run: cargo run --example align_demo

use gilt::align_widget::{Align, HorizontalAlign, VerticalAlign};
use gilt::console::Console;
use gilt::panel::Panel;
use gilt::rule::Rule;
use gilt::style::Style;
use gilt::text::Text;

fn main() {
    let mut console = Console::builder()
        .width(70)
        .force_terminal(true)
        .no_color(false)
        .build();

    console.print(&Rule::with_title("Align Widget Demo"));

    // ========================================================================
    // 1. Horizontal Alignment Options
    // ========================================================================

    console.print(&Rule::with_title("1. Horizontal Alignment"));

    // Left align (default)
    console.print(&Rule::with_title("Left Align (default)").with_characters("-"));
    let left_text = Text::new("← This text is left-aligned", Style::null());
    let left_align = Align::left(left_text);
    console.print(&left_align);

    // Center align
    console.print(&Rule::with_title("Center Align").with_characters("-"));
    let center_text = Text::new("← This text is centered →", Style::null());
    let center_align = Align::center(center_text);
    console.print(&center_align);

    // Right align
    console.print(&Rule::with_title("Right Align").with_characters("-"));
    let right_text = Text::new("This text is right-aligned →", Style::null());
    let right_align = Align::right(right_text);
    console.print(&right_align);

    // ========================================================================
    // 2. Visual Comparison: All Three Horizontal Alignments
    // ========================================================================

    console.print(&Rule::with_title("2. Side-by-Side Comparison"));

    let sample_text = "Aligned";

    // Left
    console.print(&Rule::with_title("Left").with_characters("·"));
    let left = Align::left(Text::new(sample_text, Style::null()));
    console.print(&left);

    // Center
    console.print(&Rule::with_title("Center").with_characters("·"));
    let center = Align::center(Text::new(sample_text, Style::null()));
    console.print(&center);

    // Right
    console.print(&Rule::with_title("Right").with_characters("·"));
    let right = Align::right(Text::new(sample_text, Style::null()));
    console.print(&right);

    // ========================================================================
    // 3. Vertical Alignment
    // ========================================================================

    console.print(&Rule::with_title(
        "3. Vertical Alignment (with fixed height)",
    ));

    let v_content = Text::new("Vertical Content", Style::null());

    // Top alignment
    console.print(&Rule::with_title("Top (height=5)").with_characters("·"));
    let top_align = Align::new(
        v_content.clone(),
        HorizontalAlign::Center,
        None,
        Some(VerticalAlign::Top),
        true,
        None,
        Some(5),
    );
    console.print(&top_align);

    // Middle alignment
    console.print(&Rule::with_title("Middle (height=5)").with_characters("·"));
    let middle_align = Align::new(
        v_content.clone(),
        HorizontalAlign::Center,
        None,
        Some(VerticalAlign::Middle),
        true,
        None,
        Some(5),
    );
    console.print(&middle_align);

    // Bottom alignment
    console.print(&Rule::with_title("Bottom (height=5)").with_characters("·"));
    let bottom_align = Align::new(
        v_content,
        HorizontalAlign::Center,
        None,
        Some(VerticalAlign::Bottom),
        true,
        None,
        Some(5),
    );
    console.print(&bottom_align);

    // ========================================================================
    // 4. Fixed Width Alignment
    // ========================================================================

    console.print(&Rule::with_title("4. Fixed Width Alignment"));

    let fixed_content = Text::new("Centered in 30 chars", Style::null());
    let fixed_align = Align::new(
        fixed_content,
        HorizontalAlign::Center,
        None,
        None,
        true,
        Some(30),
        None,
    );
    console.print(&Rule::with_title("Width = 30").with_characters("·"));
    console.print(&fixed_align);

    // Demonstrate left align with fixed width
    let fixed_left = Text::new("Left in 25 chars", Style::null());
    let fixed_left_align = Align::new(
        fixed_left,
        HorizontalAlign::Left,
        None,
        None,
        true,
        Some(25),
        None,
    );
    console.print(&Rule::with_title("Left, Width = 25").with_characters("·"));
    console.print(&fixed_left_align);

    // ========================================================================
    // 5. Alignment with Multiple Lines
    // ========================================================================

    console.print(&Rule::with_title("5. Multi-Line Alignment"));

    let multi_line = Text::new(
        "First line of text\nSecond line here\nThird and final line",
        Style::null(),
    );

    console.print(&Rule::with_title("Left (multi-line)").with_characters("·"));
    let left_multi = Align::left(multi_line.clone());
    console.print(&left_multi);

    console.print(&Rule::with_title("Center (multi-line)").with_characters("·"));
    let center_multi = Align::center(multi_line.clone());
    console.print(&center_multi);

    console.print(&Rule::with_title("Right (multi-line)").with_characters("·"));
    let right_multi = Align::right(multi_line);
    console.print(&right_multi);

    // ========================================================================
    // 6. Alignment with Styled Content
    // ========================================================================

    console.print(&Rule::with_title("6. Styled Content Alignment"));

    // Center-aligned with styled text using markup
    console.print(&Rule::with_title("Styled Center").with_characters("·"));
    let styled_center = Align::center(
        Text::from_markup("[bold magenta]★[/bold magenta] [bold]Centered Star[/bold] [bold magenta]★[/bold magenta]").unwrap()
    );
    console.print(&styled_center);

    // Right-aligned with styled background
    console.print(&Rule::with_title("Styled Right with BG").with_characters("·"));
    let styled_right = Align::new(
        Text::from_markup("[green on black] Styled Right [/green on black]").unwrap(),
        HorizontalAlign::Right,
        Some(Style::parse("on black").unwrap()),
        None,
        true,
        None,
        None,
    );
    console.print(&styled_right);

    // ========================================================================
    // 7. Combined Horizontal + Vertical Alignment
    // ========================================================================

    console.print(&Rule::with_title("7. Combined H+V Alignment"));

    console.print(&Rule::with_title("Center + Middle").with_characters("·"));
    let combo_text = Text::new("Center-Center", Style::null());
    let center_both = Align::new(
        combo_text,
        HorizontalAlign::Center,
        None,
        Some(VerticalAlign::Middle),
        true,
        None,
        Some(5),
    );
    console.print(&center_both);

    console.print(&Rule::with_title("Right + Bottom").with_characters("·"));
    let right_bottom_text = Text::new("Right-Bottom", Style::null());
    let right_bottom = Align::new(
        right_bottom_text,
        HorizontalAlign::Right,
        None,
        Some(VerticalAlign::Bottom),
        true,
        None,
        Some(5),
    );
    console.print(&right_bottom);

    console.print(&Rule::with_title("Left + Top").with_characters("·"));
    let left_top_text = Text::new("Left-Top", Style::null());
    let left_top = Align::new(
        left_top_text,
        HorizontalAlign::Left,
        None,
        Some(VerticalAlign::Top),
        true,
        None,
        Some(5),
    );
    console.print(&left_top);

    // ========================================================================
    // 8. No Padding Mode
    // ========================================================================

    console.print(&Rule::with_title("8. Without Right Padding (pad=false)"));

    // With padding (default)
    console.print(&Rule::with_title("pad=true (default)").with_characters("·"));
    let with_pad = Align::center(Text::new("With padding (fills width)", Style::null()));
    console.print(&with_pad);

    // Without padding
    console.print(&Rule::with_title("pad=false").with_characters("·"));
    let no_pad = Align::new(
        Text::new("No right padding", Style::null()),
        HorizontalAlign::Center,
        None,
        None,
        false, // pad = false
        None,
        None,
    );
    console.print(&no_pad);

    // ========================================================================
    // 9. Alignment Within Panels (using Panel's title alignment)
    // ========================================================================

    console.print(&Rule::with_title("9. Alignment Within Panels"));

    // Panel with different title alignments to show alignment concept
    let panel_content = Text::new("Panel content here", Style::null());

    // Note: Panel title alignment uses HorizontalAlign
    let left_panel = Panel::new(panel_content.clone())
        .with_title(Text::new("Left Title", Style::null()))
        .with_title_align(HorizontalAlign::Left);
    console.print(&left_panel);

    let center_panel = Panel::new(panel_content.clone())
        .with_title(Text::new("Center Title", Style::null()))
        .with_title_align(HorizontalAlign::Center);
    console.print(&center_panel);

    let right_panel = Panel::new(panel_content)
        .with_title(Text::new("Right Title", Style::null()))
        .with_title_align(HorizontalAlign::Right);
    console.print(&right_panel);

    console.print(&Rule::with_title("End of Align Demo"));
}
