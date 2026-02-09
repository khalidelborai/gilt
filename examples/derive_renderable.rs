//! Demonstrates gilt's #[derive(Renderable)] proc macro.
//! Run with: cargo run --example derive_renderable --features derive
//!
//! The Renderable derive implements `gilt::console::Renderable` for a struct by
//! delegating to a widget conversion method. By default it calls `to_panel()`,
//! so the struct must also derive Panel. Use `#[renderable(via = "tree")]` to
//! delegate through `to_tree()` instead (requires Tree derive).
//!
//! Once a struct implements Renderable, it can be passed directly to
//! `console.print()` without manual conversion.

#[cfg(feature = "derive")]
fn main() {
    use gilt::prelude::*;
    use gilt::Panel as PanelDerive;
    use gilt::Renderable as RenderableDerive;
    use gilt::Tree as TreeDerive;

    let mut console = Console::builder()
        .width(100)
        .force_terminal(true)
        .no_color(false)
        .build();

    // ── 1. Basic: Panel-backed Renderable ────────────────────────────────
    console.rule(Some("1. Basic Panel-backed Renderable"));

    #[derive(PanelDerive, RenderableDerive)]
    #[panel(title = "App Config", box_style = "ROUNDED", border_style = "cyan")]
    struct AppConfig {
        #[field(label = "Version", style = "bold")]
        version: String,
        #[field(label = "Debug Mode")]
        debug: bool,
        #[field(label = "Workers")]
        workers: u32,
        #[field(label = "Log Level", style = "yellow")]
        log_level: String,
    }

    let config = AppConfig {
        version: "2.1.0".into(),
        debug: false,
        workers: 8,
        log_level: "info".into(),
    };

    // Because AppConfig implements Renderable, we can print it directly:
    console.print(&config);

    // ── 2. Tree-backed Renderable ────────────────────────────────────────
    console.rule(Some("2. Tree-backed Renderable (via = \"tree\")"));

    #[derive(TreeDerive, RenderableDerive)]
    #[renderable(via = "tree")]
    struct Department {
        #[tree(label)]
        name: String,
        #[tree(children)]
        teams: Vec<Department>,
    }

    let org = Department {
        name: "Engineering".into(),
        teams: vec![
            Department {
                name: "Platform".into(),
                teams: vec![
                    Department {
                        name: "Infrastructure".into(),
                        teams: vec![],
                    },
                    Department {
                        name: "Developer Tools".into(),
                        teams: vec![],
                    },
                ],
            },
            Department {
                name: "Product".into(),
                teams: vec![
                    Department {
                        name: "Frontend".into(),
                        teams: vec![],
                    },
                    Department {
                        name: "Backend".into(),
                        teams: vec![],
                    },
                    Department {
                        name: "Mobile".into(),
                        teams: vec![],
                    },
                ],
            },
            Department {
                name: "Data".into(),
                teams: vec![
                    Department {
                        name: "ML Pipeline".into(),
                        teams: vec![],
                    },
                    Department {
                        name: "Analytics".into(),
                        teams: vec![],
                    },
                ],
            },
        ],
    };

    // Print directly — Renderable delegates through to_tree():
    console.print(&org);

    // ── 3. Styled Panel Renderable — a custom "widget" type ──────────────
    console.rule(Some("3. Custom widget: styled health check panel"));

    #[derive(PanelDerive, RenderableDerive)]
    #[panel(
        title = "Health Check",
        box_style = "HEAVY",
        border_style = "green",
        title_style = "bold white"
    )]
    struct HealthCheck {
        #[field(label = "Service", style = "bold cyan")]
        service: String,
        #[field(label = "Status", style = "bold green")]
        status: String,
        #[field(label = "Latency", style = "yellow")]
        latency: String,
        #[field(label = "Uptime")]
        uptime: String,
        #[field(label = "Last Checked", style = "dim")]
        last_checked: String,
        #[field(skip)]
        #[allow(dead_code)]
        internal_endpoint: String,
    }

    let checks = vec![
        HealthCheck {
            service: "auth-service".into(),
            status: "HEALTHY".into(),
            latency: "12ms".into(),
            uptime: "99.99%".into(),
            last_checked: "2026-02-09 14:30:00 UTC".into(),
            internal_endpoint: "http://10.0.1.5:8080/health".into(),
        },
        HealthCheck {
            service: "billing-api".into(),
            status: "DEGRADED".into(),
            latency: "340ms".into(),
            uptime: "98.7%".into(),
            last_checked: "2026-02-09 14:30:01 UTC".into(),
            internal_endpoint: "http://10.0.2.8:8080/health".into(),
        },
        HealthCheck {
            service: "notification-worker".into(),
            status: "HEALTHY".into(),
            latency: "5ms".into(),
            uptime: "100%".into(),
            last_checked: "2026-02-09 14:30:00 UTC".into(),
            internal_endpoint: "http://10.0.3.2:8080/health".into(),
        },
    ];

    // Each HealthCheck implements Renderable, so we can print them directly:
    for check in &checks {
        console.print(check);
    }

    // ── 4. Practical: Renderable in a collection context ─────────────────
    console.rule(Some("4. Renderable structs composed with other widgets"));

    #[derive(PanelDerive, RenderableDerive)]
    #[panel(
        title = "Deployment",
        box_style = "DOUBLE",
        border_style = "bright_blue",
        title_style = "bold"
    )]
    struct DeploymentInfo {
        #[field(label = "Environment", style = "bold")]
        environment: String,
        #[field(label = "Image")]
        image: String,
        #[field(label = "Replicas")]
        replicas: String,
        #[field(label = "CPU Request")]
        cpu: String,
        #[field(label = "Memory Limit")]
        memory: String,
    }

    let staging = DeploymentInfo {
        environment: "staging".into(),
        image: "app:v2.4.1-rc3".into(),
        replicas: "2 / 2 ready".into(),
        cpu: "250m".into(),
        memory: "512Mi".into(),
    };

    let production = DeploymentInfo {
        environment: "production".into(),
        image: "app:v2.3.0".into(),
        replicas: "6 / 6 ready".into(),
        cpu: "500m".into(),
        memory: "1Gi".into(),
    };

    // Because they implement Renderable, they work with console.print() directly:
    console.print(&staging);
    console.print(&production);
}

#[cfg(not(feature = "derive"))]
fn main() {
    eprintln!(
        "This example requires the 'derive' feature: cargo run --example derive_renderable --features derive"
    );
}
