//! Demonstrates gilt's console.log() — timestamped log output.
//!
//! Each line is prefixed with a `[HH:MM:SS]` timestamp styled with the
//! `log.time` theme style. The body text supports markup for inline styling.

use gilt::console::Console;
use gilt::rule::Rule;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    console.print(&Rule::with_title("Console Logging Demo"));

    // Basic log messages
    console.log("Server starting up...");
    console.log("[bold green]Application initialized[/bold green]");
    console.log("Listening on [bold cyan]0.0.0.0:8080[/bold cyan]");

    // Simulated request log entries
    console.log("[bold]GET[/bold] /api/v1/users [green]200 OK[/green] (12ms)");
    console.log("[bold]POST[/bold] /api/v1/login [green]200 OK[/green] (45ms)");
    console.log("[bold]GET[/bold] /api/v1/orders [green]200 OK[/green] (8ms)");
    console.log("[bold]DELETE[/bold] /api/v1/sessions/42 [yellow]204 No Content[/yellow] (3ms)");
    console.log("[bold]GET[/bold] /api/v1/missing [bold red]404 Not Found[/bold red] (2ms)");
    console.log(
        "[bold]POST[/bold] /api/v1/upload [bold red]500 Internal Server Error[/bold red] (120ms)",
    );

    // Status updates
    console.log("[yellow]Warning:[/yellow] Connection pool at 80% capacity");
    console.log("[red]Error:[/red] Database connection timeout after 30s");
    console.log("[green]Info:[/green] Cache hit ratio: [bold]94.2%[/bold]");
    console.log("[dim]Debug:[/dim] Request headers: Content-Type=application/json");

    // Shutdown sequence
    console.log("Graceful shutdown initiated...");
    console.log("[bold green]Server stopped cleanly[/bold green]");
}
