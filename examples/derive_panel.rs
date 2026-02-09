//! Demonstrates gilt's #[derive(Panel)] proc macro.
//! Run with: cargo run --example derive_panel --features derive

#[cfg(feature = "derive")]
fn main() {
    use gilt::prelude::*;
    use gilt::Panel as PanelDerive;

    let mut console = Console::new();

    // ── 1. Basic panel — defaults to struct name title, ROUNDED box ─────
    console.print_text("\n[bold cyan]1. Basic panel — no attributes (struct name as title)[/]");

    #[derive(PanelDerive)]
    struct AppConfig {
        version: String,
        debug: bool,
        workers: u32,
    }

    let config = AppConfig {
        version: "1.4.2".into(),
        debug: true,
        workers: 8,
    };
    console.print(&config.to_panel());

    // ── 2. Styled panel — title, box_style, border_style, title_style ───
    console.print_text(
        "\n[bold cyan]2. Styled panel — custom title, box, border, and field styles[/]",
    );

    #[derive(PanelDerive)]
    #[panel(
        title = "Build Info",
        box_style = "HEAVY",
        border_style = "bright_blue",
        title_style = "bold magenta"
    )]
    struct BuildInfo {
        #[field(label = "Compiler", style = "bold cyan")]
        compiler: String,
        #[field(label = "Target", style = "yellow")]
        target: String,
        #[field(label = "Opt Level", style = "green")]
        opt_level: String,
        #[field(label = "Features")]
        features: String,
    }

    let build = BuildInfo {
        compiler: "rustc 1.82.0 (f6e511eec 2024-10-15)".into(),
        target: "x86_64-unknown-linux-gnu".into(),
        opt_level: "3 (release)".into(),
        features: "default, json, syntax, derive".into(),
    };
    console.print(&build.to_panel());

    // ── 3. Skip fields — hide sensitive data with #[field(skip)] ────────
    console.print_text("\n[bold cyan]3. Skip fields — passwords and tokens hidden[/]");

    #[derive(PanelDerive)]
    #[panel(title = "User Account", box_style = "DOUBLE", border_style = "red")]
    struct UserAccount {
        username: String,
        email: String,
        role: String,
        #[field(skip)]
        #[allow(dead_code)]
        password_hash: String,
        #[field(skip)]
        #[allow(dead_code)]
        api_token: String,
    }

    let user = UserAccount {
        username: "alice".into(),
        email: "alice@example.com".into(),
        role: "admin".into(),
        password_hash: "$2b$12$LJ3m4ys...REDACTED".into(),
        api_token: "sk-live-abc123...REDACTED".into(),
    };
    console.print(&user.to_panel());

    // ── 4. Custom labels — override field names with #[field(label)] ────
    console.print_text("\n[bold cyan]4. Custom labels — friendly names for fields[/]");

    #[derive(PanelDerive)]
    #[panel(
        title = "Network Interface",
        box_style = "ROUNDED",
        border_style = "green"
    )]
    struct NetworkInterface {
        #[field(label = "Interface")]
        if_name: String,
        #[field(label = "IPv4 Address", style = "bold")]
        ipv4_addr: String,
        #[field(label = "Subnet Mask")]
        subnet: String,
        #[field(label = "MAC Address", style = "dim")]
        hw_addr: String,
        #[field(label = "Link Speed")]
        speed_mbps: String,
        #[field(label = "Packets RX/TX")]
        packets: String,
    }

    let iface = NetworkInterface {
        if_name: "eth0".into(),
        ipv4_addr: "192.168.1.42".into(),
        subnet: "255.255.255.0".into(),
        hw_addr: "aa:bb:cc:dd:ee:ff".into(),
        speed_mbps: "1000 Mbps".into(),
        packets: "1,234,567 / 987,654".into(),
    };
    console.print(&iface.to_panel());

    // ── 5. Server dashboard — real-world example ────────────────────────
    console.print_text("\n[bold cyan]5. Server dashboard — real-world monitoring panel[/]");

    #[derive(PanelDerive)]
    #[panel(
        title = "Server Dashboard",
        subtitle = "prod-web-01",
        box_style = "HEAVY",
        border_style = "bright_green",
        title_style = "bold white"
    )]
    struct ServerDashboard {
        #[field(label = "Hostname", style = "bold cyan")]
        hostname: String,
        #[field(label = "CPU Usage", style = "yellow")]
        cpu: String,
        #[field(label = "Memory", style = "green")]
        memory: String,
        #[field(label = "Disk", style = "blue")]
        disk: String,
        #[field(label = "Uptime")]
        uptime: String,
        #[field(label = "Status", style = "bold green")]
        status: String,
        #[field(label = "Load Avg")]
        load_avg: String,
        #[field(label = "Open Connections")]
        connections: String,
        #[field(skip)]
        #[allow(dead_code)]
        internal_ip: String,
    }

    let server = ServerDashboard {
        hostname: "prod-web-01.us-east-1.compute.internal".into(),
        cpu: "42.3% (8 cores)".into(),
        memory: "12.4 / 32.0 GB (38.8%)".into(),
        disk: "156 / 500 GB (31.2%)".into(),
        uptime: "47 days, 13:22:09".into(),
        status: "HEALTHY".into(),
        load_avg: "1.23  0.98  0.87".into(),
        connections: "1,247 active".into(),
        internal_ip: "10.0.3.42".into(),
    };
    console.print(&server.to_panel());
}

#[cfg(not(feature = "derive"))]
fn main() {
    eprintln!(
        "This example requires the 'derive' feature: cargo run --example derive_panel --features derive"
    );
}
