//! Padding widget demonstration
//!
//! Demonstrates various padding configurations with different content types.
//!
//! Run: cargo run --example padding_demo

use gilt::padding::{Padding, PaddingDimensions};
use gilt::panel::Panel;
use gilt::prelude::*;
use gilt::table::Table;

fn main() {
    let mut console = Console::builder()
        .width(70)
        .force_terminal(true)
        .no_color(false)
        .build();

    // ========================================================================
    // Section 1: Basic Padding Configurations
    // ========================================================================

    console.print(&Rule::with_title("Basic Padding Configurations"));

    // -- 1.1 Default/Uniform Padding -----------------------------------------
    console.print_text("[bold]1. Uniform Padding (2 cells all sides)[/bold]");
    let content = Text::new("Content with uniform padding", Style::null());
    let padded = Padding::new(
        content,
        PaddingDimensions::Uniform(2),
        Style::parse("on blue").unwrap(),
        true,
    );
    console.print(&padded);
    console.print_text("");

    // -- 1.2 Horizontal-only Padding -----------------------------------------
    console.print_text("[bold]2. Horizontal-only Padding (left=4, right=4)[/bold]");
    let content = Text::new("Horizontal padding only", Style::null());
    let padded = Padding::new(
        content,
        PaddingDimensions::Pair(0, 4), // (vertical=0, horizontal=4)
        Style::parse("on green").unwrap(),
        true,
    );
    console.print(&padded);
    console.print_text("");

    // -- 1.3 Vertical-only Padding -------------------------------------------
    console.print_text("[bold]3. Vertical-only Padding (top=2, bottom=2)[/bold]");
    let content = Text::new("Vertical padding only", Style::null());
    let padded = Padding::new(
        content,
        PaddingDimensions::Pair(2, 0), // (vertical=2, horizontal=0)
        Style::parse("on yellow").unwrap(),
        true,
    );
    console.print(&padded);
    console.print_text("");

    // -- 1.4 Asymmetric Padding ----------------------------------------------
    console.print_text("[bold]4. Asymmetric Padding (top=1, right=3, bottom=1, left=6)[/bold]");
    let content = Text::new("Asymmetric padding on each side", Style::null());
    let padded = Padding::new(
        content,
        PaddingDimensions::Full(1, 3, 1, 6), // (top, right, bottom, left)
        Style::parse("on magenta").unwrap(),
        true,
    );
    console.print(&padded);
    console.print_text("");

    // -- 1.5 Indent (Left Padding Only) --------------------------------------
    console.print_text("[bold]5. Indent (left=10 only)[/bold]");
    let content = Text::from_markup("[italic]Indented text using Padding::indent()[/italic]").unwrap();
    let indented = Padding::indent(content, 10);
    console.print(&indented);
    console.print_text("");

    // ========================================================================
    // Section 2: Padding with Different Content Types
    // ========================================================================

    console.print(&Rule::with_title("Padding with Different Content Types"));

    // -- 2.1 Padding with Styled Text ----------------------------------------
    console.print_text("[bold]1. Padding with Styled Text[/bold]");
    let styled_text = Text::from_markup(
        "[bold cyan]Important:[/bold cyan] This is a [italic]styled[/italic] message!",
    )
    .unwrap();
    let padded = Padding::new(
        styled_text,
        PaddingDimensions::Uniform(1),
        Style::parse("on bright_black").unwrap(),
        true,
    );
    console.print(&padded);
    console.print_text("");

    // -- 2.2 Padding with Panel ----------------------------------------------
    console.print_text("[bold]2. Padding wrapping a Panel[/bold]");
    let panel_content = Text::new("Panel inside padding", Style::null());
    let panel = Panel::new(panel_content)
        .with_title("Inner Panel")
        .with_border_style(Style::parse("cyan").unwrap());
    // Convert panel to text for padding (Padding takes Text content)
    console.print_text("   [dim]→ Panel wrapped in 2-cell padding:[/dim]");
    let panel_str = format!("{}", panel);
    let panel_text = Text::new(&panel_str, Style::null());
    let padded_panel = Padding::new(
        panel_text,
        PaddingDimensions::Uniform(2),
        Style::parse("on dark_blue").unwrap(),
        true,
    );
    console.print(&padded_panel);
    console.print_text("");

    // -- 2.3 Padding with Table Content --------------------------------------
    console.print_text("[bold]3. Padding with Table[/bold]");
    let mut table = Table::new(&["Name", "Role", "Status"]);
    table.add_row(&["Alice", "Developer", "Active"]);
    table.add_row(&["Bob", "Designer", "Away"]);
    table.add_row(&["Carol", "Manager", "Active"]);
    let table_str = format!("{}", table);
    let table_text = Text::new(&table_str, Style::null());
    let padded_table = Padding::new(
        table_text,
        PaddingDimensions::Uniform(1),
        Style::parse("on dark_green").unwrap(),
        true,
    );
    console.print(&padded_table);
    console.print_text("");

    // ========================================================================
    // Section 3: Styled Padding
    // ========================================================================

    console.print(&Rule::with_title("Styled Padding (Dim Style for Padding Area)"));

    // -- 3.1 Dim Padding Style -----------------------------------------------
    console.print_text("[bold]Padding with dim style:[/bold]");
    let content = Text::new("Content with dim padding background", Style::null());
    let padded = Padding::new(
        content,
        PaddingDimensions::Uniform(3),
        Style::parse("dim on black").unwrap(),
        true,
    );
    console.print(&padded);
    console.print_text("");

    // -- 3.2 Styled Padding with Color ---------------------------------------
    console.print_text("[bold]Padding with colored background:[/bold]");
    let content = Text::from_markup("[bold white]Alert:[/bold white] Check your settings").unwrap();
    let padded = Padding::new(
        content,
        PaddingDimensions::Pair(1, 4),
        Style::parse("on red").unwrap(),
        true,
    );
    console.print(&padded);
    console.print_text("");

    // -- 3.3 Gradient-style Padding (using dim and background) ---------------
    console.print_text("[bold]Padding with subtle styling:[/bold]");
    let content = Text::new("Subtle padded content area", Style::null());
    let padded = Padding::new(
        content,
        PaddingDimensions::Full(2, 4, 2, 4),
        Style::parse("dim on bright_black").unwrap(),
        true,
    );
    console.print(&padded);
    console.print_text("");

    // ========================================================================
    // Section 4: Visual Comparison
    // ========================================================================

    console.print(&Rule::with_title("Visual Comparison: Same Content, Different Padding"));

    let sample_text = Text::from_markup("[bold]Sample Content[/bold]\nThis demonstrates padding effects.").unwrap();

    // Create a comparison table using multiple panels
    console.print_text("[bold]Comparing padding configurations side by side:[/bold]\n");

    // No padding
    console.print_text("[dim]1. No padding (PaddingDimensions::Uniform(0)):[/dim]");
    let no_pad = Padding::new(
        sample_text.clone(),
        PaddingDimensions::Uniform(0),
        Style::null(),
        false,
    );
    console.print(&no_pad);
    console.print_text("");

    // Small padding (1 cell)
    console.print_text("[dim]2. Small padding (Uniform(1)):[/dim]");
    let small_pad = Padding::new(
        sample_text.clone(),
        PaddingDimensions::Uniform(1),
        Style::parse("on bright_black").unwrap(),
        false,
    );
    console.print(&small_pad);
    console.print_text("");

    // Medium padding (2 cells)
    console.print_text("[dim]3. Medium padding (Uniform(2)):[/dim]");
    let medium_pad = Padding::new(
        sample_text.clone(),
        PaddingDimensions::Uniform(2),
        Style::parse("on bright_black").unwrap(),
        false,
    );
    console.print(&medium_pad);
    console.print_text("");

    // Large horizontal, small vertical
    console.print_text("[dim]4. Wide horizontal padding (Pair(1, 6)):[/dim]");
    let wide_pad = Padding::new(
        sample_text.clone(),
        PaddingDimensions::Pair(1, 6),
        Style::parse("on bright_black").unwrap(),
        false,
    );
    console.print(&wide_pad);
    console.print_text("");

    // Asymmetric
    console.print_text("[dim]5. Asymmetric padding (Full(2, 4, 1, 8)):[/dim]");
    let asym_pad = Padding::new(
        sample_text,
        PaddingDimensions::Full(2, 4, 1, 8),
        Style::parse("on bright_black").unwrap(),
        false,
    );
    console.print(&asym_pad);
    console.print_text("");

    // ========================================================================
    // Section 5: Practical Use Cases
    // ========================================================================

    console.print(&Rule::with_title("Practical Use Cases"));

    // -- 5.1 Card-like Display -----------------------------------------------
    console.print_text("[bold]1. Card-like Display with Padding + Panel[/bold]");
    let card_content = Text::from_markup(
        "[bold green]Success![/bold green]\nYour changes have been saved.",
    )
    .unwrap();
    let card = Panel::new(card_content)
        .with_title("Notification")
        .with_padding(PaddingDimensions::Uniform(2))
        .with_border_style(Style::parse("green").unwrap());
    console.print(&card);
    console.print_text("");

    // -- 5.2 Quote/Blockquote ------------------------------------------------
    console.print_text("[bold]2. Blockquote Style with Left Indent[/bold]");
    let quote = Text::from_markup(
        "[italic]The only limit to our realization of tomorrow will be our doubts of today.[/italic]\n[dim]— Franklin D. Roosevelt[/dim]",
    )
    .unwrap();
    let blockquote = Padding::new(
        quote,
        PaddingDimensions::Full(1, 2, 1, 4),
        Style::parse("on bright_black dim").unwrap(),
        true,
    );
    console.print(&blockquote);
    console.print_text("");

    // -- 5.3 Header with Spacing ---------------------------------------------
    console.print_text("[bold]3. Section Header with Vertical Spacing[/bold]");
    let header = Text::from_markup("[bold cyan]§ Configuration[/bold cyan]").unwrap();
    let spaced_header = Padding::new(
        header,
        PaddingDimensions::Pair(1, 2),
        Style::parse("on dark_blue").unwrap(),
        true,
    );
    console.print(&spaced_header);
    console.print_text("   Settings go here...\n");

    // -- 5.4 Nested Padding Effect -------------------------------------------
    console.print_text("[bold]4. Multiple Padded Elements[/bold]");
    for i in 1..=3 {
        let item = Text::from_markup(&format!("[bold]Item {}[/bold]\nDescription here", i)).unwrap();
        let padded_item = Padding::new(
            item,
            PaddingDimensions::Uniform(1),
            Style::parse("on bright_black").unwrap(),
            true,
        );
        console.print(&padded_item);
        console.print_text("");
    }

    console.print(&Rule::with_title("End of Demo"));
}
