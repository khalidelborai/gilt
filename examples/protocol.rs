//! Protocol utilities example -- demonstrates the `__gilt__` protocol and casting utilities.
//!
//! This example shows how to use gilt's protocol utilities to:
//! - Implement the `RichCast` trait for custom types
//! - Check if values are renderable
//! - Cast boxed values to concrete renderable types
//! - Convert custom types to renderable representations
//!
//! Run with: cargo run --example protocol

use gilt::box_chars::ROUNDED;
use gilt::console::Renderable;
use gilt::prelude::*;
use gilt::protocol;

// =============================================================================
// Custom Types Implementing RichCast
// =============================================================================

/// A user in the system
struct User {
    username: String,
    email: String,
    role: UserRole,
    active: bool,
}

#[derive(Debug, Clone, Copy)]
enum UserRole {
    Admin,
    Moderator,
    User,
    Guest,
}

impl UserRole {
    fn as_str(&self) -> &'static str {
        match self {
            UserRole::Admin => "Admin",
            UserRole::Moderator => "Moderator",
            UserRole::User => "User",
            UserRole::Guest => "Guest",
        }
    }

    fn style(&self) -> &'static str {
        match self {
            UserRole::Admin => "bold red",
            UserRole::Moderator => "bold yellow",
            UserRole::User => "green",
            UserRole::Guest => "dim",
        }
    }
}

impl protocol::RichCast for User {
    fn __gilt__(self) -> Box<dyn Renderable> {
        let status_icon = if self.active { "🟢" } else { "🔴" };
        let status_text = if self.active { "Active" } else { "Inactive" };

        let content = Text::from(format!(
            "👤 Username: {}\n📧 Email: {}\n🛡️  Role: [{}]{}[/]\n{} Status: {}",
            self.username,
            self.email,
            self.role.style(),
            self.role.as_str(),
            status_icon,
            status_text
        ));

        Box::new(
            Panel::new(content)
                .with_title("User Profile")
                .with_border_style(Style::parse("blue").unwrap()),
        )
    }
}

/// A system service with health status
struct Service {
    name: String,
    version: String,
    healthy: bool,
    latency_ms: u64,
    uptime_percent: f64,
}

impl protocol::RichCast for Service {
    fn __gilt__(self) -> Box<dyn Renderable> {
        let health_style = if self.healthy {
            "bold green"
        } else {
            "bold red"
        };
        let health_status = if self.healthy { "HEALTHY" } else { "UNHEALTHY" };

        let mut table = Table::new(&["Property", "Value"]);
        table.add_row(&["Name", &self.name]);
        table.add_row(&["Version", &self.version]);
        table.add_row(&["Status", &format!("[{}]{}[/]", health_style, health_status)]);
        table.add_row(&["Latency", &format!("{}ms", self.latency_ms)]);
        table.add_row(&["Uptime", &format!("{:.2}%", self.uptime_percent)]);

        let border_style = if self.healthy { "green" } else { "red" };

        Box::new(
            table
                .with_title(&format!("Service: {}", self.name))
                .with_border_style(border_style),
        )
    }
}

/// A log entry for display
struct LogEntry {
    timestamp: String,
    level: LogLevel,
    message: String,
    source: String,
}

#[derive(Debug, Clone, Copy)]
enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Critical => "CRIT",
        }
    }

    fn style(&self) -> &'static str {
        match self {
            LogLevel::Debug => "dim",
            LogLevel::Info => "cyan",
            LogLevel::Warning => "yellow",
            LogLevel::Error => "red",
            LogLevel::Critical => "bold red on white",
        }
    }
}

impl protocol::RichCast for LogEntry {
    fn __gilt__(self) -> Box<dyn Renderable> {
        let content = Text::from(format!(
            "[dim]{}[/] [[{}]{}[/]] [bold]{}[/]: {}",
            self.timestamp,
            self.level.style(),
            self.level.as_str(),
            self.source,
            self.message
        ));
        Box::new(content)
    }
}

/// A collection that can be displayed as a dashboard
struct ServiceDashboard {
    services: Vec<Service>,
}

impl protocol::RichCast for ServiceDashboard {
    fn __gilt__(self) -> Box<dyn Renderable> {
        let mut table = Table::new(&["Service", "Version", "Status", "Latency", "Uptime"]);

        for service in self.services {
            let status = if service.healthy {
                "[green]✓ HEALTHY[/]"
            } else {
                "[red]✗ UNHEALTHY[/]"
            };

            table.add_row(&[
                &service.name,
                &service.version,
                status,
                &format!("{}ms", service.latency_ms),
                &format!("{:.1}%", service.uptime_percent),
            ]);
        }

        Box::new(
            table
                .with_title("Service Dashboard")
                .with_box_chars(Some(&ROUNDED))
                .with_border_style("cyan"),
        )
    }
}

/// Quick status struct using the macro for RichCast
struct QuickStatus {
    label: &'static str,
    value: i32,
}

gilt::rich_cast_impl! { QuickStatus => |s|
    Box::new(Panel::new(Text::from(format!("{}: {}", s.label, s.value)))
        .with_title("Quick Status"))
}

// =============================================================================
// Main Example
// =============================================================================

fn main() {
    let mut console = Console::builder()
        .width(100)
        .force_terminal(true)
        .no_color(false)
        .build();

    // ── 1. RichCast - Custom User Type ──────────────────────────────────────
    console.rule(Some("1. RichCast - Custom User Type"));
    println!();

    let user = User {
        username: "alice_dev".into(),
        email: "alice@example.com".into(),
        role: UserRole::Admin,
        active: true,
    };

    // Convert the user to a renderable using RichCast
    let user_renderable = protocol::IntoRenderable::into_renderable(user);
    console.print(&*user_renderable);
    println!();

    // Another user with different role
    let user2 = User {
        username: "bob_user".into(),
        email: "bob@example.com".into(),
        role: UserRole::User,
        active: false,
    };
    console.print(&*protocol::IntoRenderable::into_renderable(user2));
    println!();

    // ── 2. Service Status Display ───────────────────────────────────────────
    console.rule(Some("2. Service Status with RichCast"));
    println!();

    let auth_service = Service {
        name: "auth-service".into(),
        version: "v2.3.1".into(),
        healthy: true,
        latency_ms: 12,
        uptime_percent: 99.99,
    };

    let billing_service = Service {
        name: "billing-api".into(),
        version: "v1.8.0".into(),
        healthy: false,
        latency_ms: 2450,
        uptime_percent: 97.45,
    };

    console.print(&*protocol::IntoRenderable::into_renderable(auth_service));
    println!();
    console.print(&*protocol::IntoRenderable::into_renderable(billing_service));
    println!();

    // ── 3. Log Entries ──────────────────────────────────────────────────────
    console.rule(Some("3. Log Entries"));
    println!();

    let logs = vec![
        LogEntry {
            timestamp: "2026-02-09 14:32:01".into(),
            level: LogLevel::Info,
            message: "Application started successfully".into(),
            source: "main".into(),
        },
        LogEntry {
            timestamp: "2026-02-09 14:32:15".into(),
            level: LogLevel::Debug,
            message: "Connected to database pool (10 connections)".into(),
            source: "db".into(),
        },
        LogEntry {
            timestamp: "2026-02-09 14:35:42".into(),
            level: LogLevel::Warning,
            message: "High memory usage detected (85%)".into(),
            source: "monitor".into(),
        },
        LogEntry {
            timestamp: "2026-02-09 14:36:10".into(),
            level: LogLevel::Error,
            message: "Failed to connect to cache server".into(),
            source: "cache".into(),
        },
    ];

    for log in logs {
        console.print(&*protocol::IntoRenderable::into_renderable(log));
    }
    println!();

    // ── 4. Service Dashboard ────────────────────────────────────────────────
    console.rule(Some("4. Service Dashboard"));
    println!();

    let dashboard = ServiceDashboard {
        services: vec![
            Service {
                name: "api-gateway".into(),
                version: "v3.0.2".into(),
                healthy: true,
                latency_ms: 5,
                uptime_percent: 99.99,
            },
            Service {
                name: "user-service".into(),
                version: "v2.1.0".into(),
                healthy: true,
                latency_ms: 18,
                uptime_percent: 99.95,
            },
            Service {
                name: "payment-service".into(),
                version: "v1.5.3".into(),
                healthy: true,
                latency_ms: 45,
                uptime_percent: 99.99,
            },
            Service {
                name: "notification-worker".into(),
                version: "v1.2.1".into(),
                healthy: false,
                latency_ms: 3200,
                uptime_percent: 94.20,
            },
        ],
    };

    console.print(&*protocol::IntoRenderable::into_renderable(dashboard));
    println!();

    // ── 5. Type Casting with rich_cast ──────────────────────────────────────
    console.rule(Some("5. Type Casting with rich_cast"));
    println!();

    // Create a boxed Any containing a Text
    let original_text = Text::from("This is some important text content");
    let boxed: Box<dyn std::any::Any> = Box::new(original_text);

    // Try to cast it back to Text
    match protocol::rich_cast::<Text>(boxed) {
        Some(text) => {
            println!("✓ Successfully cast back to Text!");
            console.print(&*text);
        }
        None => {
            println!("✗ Failed to cast to Text");
        }
    }
    println!();

    // Demonstrate failed cast
    let text2 = Text::from("Another text");
    let boxed2: Box<dyn std::any::Any> = Box::new(text2);

    match protocol::rich_cast::<Panel>(boxed2) {
        Some(_) => println!("✓ Unexpectedly succeeded casting Text to Panel"),
        None => println!("✓ Correctly failed to cast Text to Panel (expected)"),
    }
    println!();

    // ── 6. Collection of Renderables ────────────────────────────────────────
    console.rule(Some("6. Working with Collections"));
    println!();

    // Create a vector of users and convert them all to renderables
    let users: Vec<User> = vec![
        User {
            username: "carol_m".into(),
            email: "carol@example.com".into(),
            role: UserRole::Moderator,
            active: true,
        },
        User {
            username: "dave_guest".into(),
            email: "dave@example.com".into(),
            role: UserRole::Guest,
            active: true,
        },
    ];

    let renderables: Vec<Box<dyn Renderable>> = users
        .into_iter()
        .map(|u| protocol::IntoRenderable::into_renderable(u))
        .collect();

    println!(
        "Printing {} user profiles from collection:\n",
        renderables.len()
    );
    for (i, renderable) in renderables.iter().enumerate() {
        println!("User {}:", i + 1);
        console.print(&**renderable);
        println!();
    }

    // ── 7. RenderableBox for Type-Erased Storage ────────────────────────────
    console.rule(Some("7. Type-Erased Storage with RenderableBox"));
    println!();

    // Create a heterogeneous collection of renderables
    let items: Vec<protocol::RenderableBox> = vec![
        protocol::RenderableBox::new(Text::from("Simple text item")),
        protocol::RenderableBox::new(Panel::new(Text::from("Panel item"))),
        protocol::RenderableBox::new(Rule::with_title("Rule item")),
    ];

    println!(
        "Storing {} different renderable types in one collection:\n",
        items.len()
    );
    for (i, item) in items.iter().enumerate() {
        println!("Item {}:", i + 1);
        console.print(item);
        println!();
    }

    // ── 8. Using RenderableExt ──────────────────────────────────────────────
    console.rule(Some("8. RenderableExt - Converting to RenderableBox"));
    println!();

    let text = Text::from("This text is wrapped in a RenderableBox");
    let boxed = protocol::RenderableExt::into_boxed_renderable(text);
    console.print(&boxed);
    println!();

    // ── 9. Using rich_cast_impl Macro ───────────────────────────────────────
    console.rule(Some("9. Using rich_cast_impl Macro"));
    println!();

    let quick = QuickStatus {
        label: "Count",
        value: 42,
    };
    console.print(&*protocol::IntoRenderable::into_renderable(quick));
    println!();

    // ── 10. Python __gilt__ Protocol Documentation ──────────────────────────
    console.rule(Some("10. Python __gilt__ Protocol Equivalent"));
    println!();

    let explanation = Text::from(
        "The RichCast trait is Rust's equivalent of Python's __gilt__ protocol.\n\n\
         Python Rich:\n\
           class MyClass:\n\
               def __gilt__(self):\n\
                   return Panel(str(self))\n\n\
         Rust gilt:\n\
           impl RichCast for MyClass {\n\
               fn __gilt__(self) -> Box<dyn Renderable> {\n\
                   Box::new(Panel::new(Text::from(...)))\n\
               }\n\
           }\n\n\
         Both protocols allow custom types to define their visual representation\n\
         when rendered to the console.",
    );

    console.print(&Panel::new(explanation).with_title("Protocol Comparison"));
    println!();

    console.rule(None);
    println!("\n✨ Example complete! The protocol utilities enable seamless integration\n   of custom types with gilt's rendering system.");
}
