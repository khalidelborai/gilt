//! Demonstrates gilt's #[derive(Table)] proc macro with all attribute options.
//! Run with: cargo run --example derive_table --features derive

#[cfg(feature = "derive")]
fn main() {
    use gilt::prelude::*;
    use gilt::Table;

    let mut console = Console::new();

    // ── 1. Basic derive (no attributes) ──────────────────────────────────
    console.print_text("\n[bold cyan]1. Basic derive — fields become Title Case headers[/]");

    #[derive(Table)]
    struct Planet {
        name: String,
        mass_kg: f64,
        moons: u32,
    }

    let planets = vec![
        Planet {
            name: "Mercury".into(),
            mass_kg: 3.3e23,
            moons: 0,
        },
        Planet {
            name: "Venus".into(),
            mass_kg: 4.87e24,
            moons: 0,
        },
        Planet {
            name: "Earth".into(),
            mass_kg: 5.97e24,
            moons: 1,
        },
        Planet {
            name: "Mars".into(),
            mass_kg: 6.42e23,
            moons: 2,
        },
    ];
    console.print(&Planet::to_table(&planets));

    // ── 2. Table-level attributes ────────────────────────────────────────
    console.print_text("\n[bold cyan]2. Table-level — title, caption, box_style, styles[/]");

    #[derive(Table)]
    #[table(
        title = "Q4 Sales Report",
        caption = "All figures in USD",
        box_style = "ROUNDED",
        header_style = "bold white on blue",
        border_style = "bright_blue",
        title_style = "bold magenta",
        caption_style = "dim italic",
        show_lines = true,
        pad_edge = true
    )]
    struct Sale {
        region: String,
        product: String,
        revenue: f64,
        units: u32,
    }

    let sales = vec![
        Sale {
            region: "North".into(),
            product: "Widget A".into(),
            revenue: 15420.50,
            units: 342,
        },
        Sale {
            region: "South".into(),
            product: "Widget B".into(),
            revenue: 8930.00,
            units: 178,
        },
        Sale {
            region: "East".into(),
            product: "Widget A".into(),
            revenue: 22100.75,
            units: 491,
        },
        Sale {
            region: "West".into(),
            product: "Widget C".into(),
            revenue: 6750.25,
            units: 135,
        },
    ];
    console.print(&Sale::to_table(&sales));

    // ── 3. Column-level attributes ───────────────────────────────────────
    console.print_text("\n[bold cyan]3. Column-level — rename, justify, style, width[/]");

    #[derive(Table)]
    #[table(title = "Server Status", box_style = "HEAVY", header_style = "bold")]
    struct Server {
        #[column(header = "Hostname", style = "bold cyan", min_width = 15)]
        name: String,
        #[column(header = "CPU %", justify = "right", style = "yellow")]
        cpu: f32,
        #[column(header = "Mem %", justify = "right", style = "green")]
        memory: f32,
        #[column(header = "Status", justify = "center")]
        status: String,
        #[column(skip)]
        #[allow(dead_code)]
        internal_id: u64,
    }

    let servers = vec![
        Server {
            name: "web-prod-01".into(),
            cpu: 45.2,
            memory: 62.1,
            status: "OK".into(),
            internal_id: 1001,
        },
        Server {
            name: "web-prod-02".into(),
            cpu: 78.9,
            memory: 85.3,
            status: "WARN".into(),
            internal_id: 1002,
        },
        Server {
            name: "db-primary".into(),
            cpu: 23.1,
            memory: 41.7,
            status: "OK".into(),
            internal_id: 2001,
        },
        Server {
            name: "cache-01".into(),
            cpu: 12.5,
            memory: 28.9,
            status: "OK".into(),
            internal_id: 3001,
        },
    ];
    console.print(&Server::to_table(&servers));

    // ── 4. Row styles (alternating) ──────────────────────────────────────
    console.print_text("\n[bold cyan]4. Alternating row styles[/]");

    #[derive(Table)]
    #[table(
        title = "Inventory",
        box_style = "SIMPLE",
        row_styles = "on grey11, ",
        header_style = "bold underline"
    )]
    struct Item {
        #[column(header = "SKU", style = "dim")]
        sku: String,
        #[column(header = "Product")]
        name: String,
        #[column(header = "Qty", justify = "right")]
        quantity: u32,
        #[column(header = "Price", justify = "right", style = "green")]
        price: String,
    }

    let items = vec![
        Item {
            sku: "A-001".into(),
            name: "Mechanical Keyboard".into(),
            quantity: 45,
            price: "$89.99".into(),
        },
        Item {
            sku: "A-002".into(),
            name: "Wireless Mouse".into(),
            quantity: 120,
            price: "$34.50".into(),
        },
        Item {
            sku: "B-010".into(),
            name: "USB-C Hub".into(),
            quantity: 67,
            price: "$49.99".into(),
        },
        Item {
            sku: "B-011".into(),
            name: "Monitor Arm".into(),
            quantity: 23,
            price: "$129.00".into(),
        },
        Item {
            sku: "C-100".into(),
            name: "Webcam HD".into(),
            quantity: 89,
            price: "$59.95".into(),
        },
        Item {
            sku: "C-101".into(),
            name: "Headset Pro".into(),
            quantity: 34,
            price: "$149.00".into(),
        },
    ];
    console.print(&Item::to_table(&items));

    // ── 5. Expanded table with width ratios ──────────────────────────────
    console.print_text("\n[bold cyan]5. Expanded table with column ratios[/]");

    #[derive(Table)]
    #[table(
        title = "Task Board",
        box_style = "DOUBLE",
        expand = true,
        header_style = "bold white on dark_green",
        border_style = "green"
    )]
    struct Task {
        #[column(header = "ID", justify = "center", ratio = 1)]
        id: u32,
        #[column(header = "Task", ratio = 4)]
        description: String,
        #[column(header = "Owner", ratio = 2)]
        assignee: String,
        #[column(header = "Priority", justify = "center", ratio = 1, style = "bold")]
        priority: String,
    }

    let tasks = vec![
        Task {
            id: 1,
            description: "Implement auth middleware".into(),
            assignee: "Alice".into(),
            priority: "High".into(),
        },
        Task {
            id: 2,
            description: "Fix pagination bug on /users".into(),
            assignee: "Bob".into(),
            priority: "Med".into(),
        },
        Task {
            id: 3,
            description: "Add CSV export endpoint".into(),
            assignee: "Charlie".into(),
            priority: "Low".into(),
        },
        Task {
            id: 4,
            description: "Update dependencies".into(),
            assignee: "Diana".into(),
            priority: "Med".into(),
        },
    ];
    console.print(&Task::to_table(&tasks));

    // ── 6. Minimal style (no edge, no header) ────────────────────────────
    console.print_text("\n[bold cyan]6. Minimal — no edge, hidden header[/]");

    #[derive(Table)]
    #[table(show_header = false, show_edge = false, box_style = "SIMPLE")]
    struct LogEntry {
        #[column(style = "dim")]
        timestamp: String,
        #[column(style = "bold yellow")]
        level: String,
        message: String,
    }

    let logs = vec![
        LogEntry {
            timestamp: "12:01:03".into(),
            level: "INFO".into(),
            message: "Server started on :8080".into(),
        },
        LogEntry {
            timestamp: "12:01:05".into(),
            level: "WARN".into(),
            message: "Config file not found, using defaults".into(),
        },
        LogEntry {
            timestamp: "12:01:12".into(),
            level: "INFO".into(),
            message: "Connected to database".into(),
        },
        LogEntry {
            timestamp: "12:02:45".into(),
            level: "ERR".into(),
            message: "Failed to reach payment API".into(),
        },
    ];
    console.print(&LogEntry::to_table(&logs));

    // ── 7. All box styles showcase ───────────────────────────────────────
    console.print_text("\n[bold cyan]7. Box style showcase[/]");

    macro_rules! box_demo {
        ($name:ident, $style:literal) => {
            #[derive(Table)]
            #[table(title = $style, box_style = $style)]
            struct $name {
                a: String,
                b: String,
            }
            let data = vec![$name {
                a: "Hello".into(),
                b: "World".into(),
            }];
            console.print(&$name::to_table(&data));
        };
    }

    box_demo!(BoxAscii, "ASCII");
    box_demo!(BoxSquare, "SQUARE");
    box_demo!(BoxRounded, "ROUNDED");
    box_demo!(BoxHeavy, "HEAVY");
    box_demo!(BoxDouble, "DOUBLE");
    box_demo!(BoxMinimal, "MINIMAL");
    box_demo!(BoxMarkdown, "MARKDOWN");

    // ── 8. Column constraints (max_width, no_wrap) ─────────────────────
    console.print_text("\n[bold cyan]8. Column constraints — max_width and no_wrap[/]");

    #[derive(Table)]
    #[table(title = "Error Log", box_style = "ROUNDED", header_style = "bold")]
    struct ErrorEntry {
        #[column(header = "Time", no_wrap, style = "dim")]
        timestamp: String,
        #[column(header = "Code", justify = "center", width = 6)]
        code: u16,
        #[column(header = "Message", max_width = 40)]
        message: String,
        #[column(header = "Source", max_width = 20, style = "cyan")]
        source: String,
    }

    let errors = vec![
        ErrorEntry {
            timestamp: "2026-02-09 14:23:01".into(),
            code: 500,
            message: "Connection refused: upstream server at 10.0.0.5:8080 is not responding to health checks".into(),
            source: "api-gateway/proxy.rs:142".into(),
        },
        ErrorEntry {
            timestamp: "2026-02-09 14:23:03".into(),
            code: 429,
            message: "Rate limit exceeded for client_id=abc123, retry after 30s".into(),
            source: "middleware/rate_limit.rs:88".into(),
        },
        ErrorEntry {
            timestamp: "2026-02-09 14:23:07".into(),
            code: 404,
            message: "Not found".into(),
            source: "router/dispatch.rs:201".into(),
        },
    ];
    console.print(&ErrorEntry::to_table(&errors));

    // ── 9. Per-column header styles ────────────────────────────────────
    console.print_text("\n[bold cyan]9. Per-column header styles[/]");

    #[derive(Table)]
    #[table(
        title = "Deployment Matrix",
        box_style = "HEAVY_HEAD",
        border_style = "dim"
    )]
    struct Deployment {
        #[column(
            header = "Service",
            header_style = "bold white on dark_blue",
            min_width = 14
        )]
        service: String,
        #[column(
            header = "Staging",
            header_style = "bold white on dark_orange",
            justify = "center"
        )]
        staging: String,
        #[column(
            header = "Production",
            header_style = "bold white on dark_red",
            justify = "center"
        )]
        production: String,
        #[column(
            header = "Version",
            header_style = "bold white on dark_green",
            justify = "right"
        )]
        version: String,
    }

    let deploys = vec![
        Deployment {
            service: "auth-service".into(),
            staging: "v2.4.1".into(),
            production: "v2.3.0".into(),
            version: "behind".into(),
        },
        Deployment {
            service: "user-api".into(),
            staging: "v1.12.0".into(),
            production: "v1.12.0".into(),
            version: "synced".into(),
        },
        Deployment {
            service: "billing".into(),
            staging: "v3.0.0-rc1".into(),
            production: "v2.9.8".into(),
            version: "behind".into(),
        },
        Deployment {
            service: "notifications".into(),
            staging: "v1.5.2".into(),
            production: "v1.5.2".into(),
            version: "synced".into(),
        },
    ];
    console.print(&Deployment::to_table(&deploys));

    // ── 10. Kitchen sink — combining everything ────────────────────────
    console.print_text("\n[bold cyan]10. Kitchen sink — all attributes combined[/]");

    #[derive(Table)]
    #[table(
        title = "Employee Directory",
        caption = "Confidential — Internal Use Only",
        box_style = "DOUBLE_EDGE",
        header_style = "bold white on dark_green",
        border_style = "green",
        title_style = "bold underline",
        caption_style = "dim red italic",
        show_lines = true,
        pad_edge = true,
        row_styles = "on grey7, "
    )]
    struct Employee {
        #[column(header = "ID", justify = "right", style = "dim", width = 5)]
        id: u32,
        #[column(header = "Name", style = "bold", min_width = 16)]
        name: String,
        #[column(header = "Role", max_width = 24)]
        role: String,
        #[column(header = "Dept", header_style = "bold yellow", justify = "center")]
        department: String,
        #[column(header = "Salary", justify = "right", style = "green", no_wrap)]
        salary: String,
        #[column(skip)]
        #[allow(dead_code)]
        ssn: String,
    }

    let employees = vec![
        Employee {
            id: 1,
            name: "Alice Chen".into(),
            role: "Staff Engineer".into(),
            department: "Platform".into(),
            salary: "$185,000".into(),
            ssn: "xxx-xx-1234".into(),
        },
        Employee {
            id: 2,
            name: "Bob Martinez".into(),
            role: "Engineering Manager".into(),
            department: "Backend".into(),
            salary: "$195,000".into(),
            ssn: "xxx-xx-5678".into(),
        },
        Employee {
            id: 3,
            name: "Carol Washington".into(),
            role: "Senior Product Designer".into(),
            department: "Design".into(),
            salary: "$165,000".into(),
            ssn: "xxx-xx-9012".into(),
        },
        Employee {
            id: 4,
            name: "David Kim".into(),
            role: "DevOps Lead".into(),
            department: "Platform".into(),
            salary: "$175,000".into(),
            ssn: "xxx-xx-3456".into(),
        },
    ];
    console.print(&Employee::to_table(&employees));
}

#[cfg(not(feature = "derive"))]
fn main() {
    eprintln!("This example requires the 'derive' feature: cargo run --example derive_table --features derive");
}
