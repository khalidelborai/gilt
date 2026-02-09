//! Demonstrates the `#[derive(Inspect)]` proc-macro from gilt-derive.
//!
//! Run with:
//! ```sh
//! cargo run --example derive_inspect --features derive
//! ```

#[cfg(feature = "derive")]
fn main() {
    use gilt::prelude::*;
    use gilt::DeriveInspect;

    let mut console = Console::new();

    // ── 1. Basic inspect (just Debug) ────────────────────────────────────
    //
    // The simplest usage: derive Debug + Inspect with no extra attributes.
    // Inspect uses the struct's type name as the panel title.

    console.rule(Some("1. Basic Inspect"));

    #[derive(Debug, Clone, DeriveInspect)]
    #[allow(dead_code)]
    struct Sensor {
        id: u32,
        temperature: f64,
        online: bool,
    }

    let sensor = Sensor {
        id: 7,
        temperature: 23.5,
        online: true,
    };
    console.print(&sensor.to_inspect());

    // ── 2. Custom title and label ────────────────────────────────────────
    //
    // Use `#[inspect(title = "...", label = "...")]` to override the panel
    // title and add a label beneath it.

    console.rule(Some("2. Custom Title & Label"));

    #[derive(Debug, Clone, DeriveInspect)]
    #[inspect(title = "Cluster Node", label = "node-3a (primary)")]
    #[allow(dead_code)]
    struct ClusterNode {
        hostname: String,
        cpu_cores: u16,
        memory_gb: f64,
        healthy: bool,
    }

    let node = ClusterNode {
        hostname: "node-3a.internal".into(),
        cpu_cores: 16,
        memory_gb: 128.0,
        healthy: true,
    };
    console.print(&node.to_inspect());

    // ── 3. Doc annotation ────────────────────────────────────────────────
    //
    // Use `#[inspect(doc = "...")]` to add a documentation string that is
    // rendered above the debug output, providing context to the reader.

    console.rule(Some("3. Doc Annotation"));

    #[derive(Debug, Clone, DeriveInspect)]
    #[inspect(doc = "Pipeline stage configuration. Each stage runs sequentially.")]
    #[allow(dead_code)]
    struct PipelineStage {
        name: String,
        timeout_secs: u64,
        retries: u32,
        parallel: bool,
    }

    let stage = PipelineStage {
        name: "lint".into(),
        timeout_secs: 300,
        retries: 2,
        parallel: false,
    };
    console.print(&stage.to_inspect());

    // ── 4. Pretty printing enabled ───────────────────────────────────────
    //
    // Use `#[inspect(pretty)]` (or `pretty = true`) to enable pretty-printed
    // Debug output with indentation and syntax highlighting.

    console.rule(Some("4. Pretty Printing"));

    #[derive(Debug, Clone, DeriveInspect)]
    #[inspect(title = "Build Manifest", pretty)]
    #[allow(dead_code)]
    struct BuildManifest {
        name: String,
        version: String,
        features: Vec<String>,
        dependencies: Vec<String>,
    }

    let manifest = BuildManifest {
        name: "gilt".into(),
        version: "0.1.0".into(),
        features: vec!["derive".into(), "miette".into(), "eyre".into()],
        dependencies: vec!["serde".into(), "thiserror".into(), "unicode-width".into()],
    };
    console.print(&manifest.to_inspect());

    // ── 5. Complex nested struct inspection ──────────────────────────────
    //
    // Inspect handles nested structs naturally via the Debug trait. Combining
    // all attributes demonstrates the full power of the derive macro.

    console.rule(Some("5. Complex Nested Struct"));

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    struct Endpoint {
        path: String,
        method: String,
        rate_limit: u32,
    }

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    struct TlsConfig {
        cert_path: String,
        key_path: String,
        min_version: String,
    }

    #[derive(Debug, Clone, DeriveInspect)]
    #[inspect(
        title = "API Gateway",
        label = "production / us-east-1",
        doc = "Full gateway configuration with TLS and routing.",
        pretty
    )]
    #[allow(dead_code)]
    struct GatewayConfig {
        listen_addr: String,
        port: u16,
        tls: TlsConfig,
        endpoints: Vec<Endpoint>,
        max_connections: usize,
        idle_timeout_secs: u64,
    }

    let gateway = GatewayConfig {
        listen_addr: "0.0.0.0".into(),
        port: 443,
        tls: TlsConfig {
            cert_path: "/etc/ssl/certs/gateway.pem".into(),
            key_path: "/etc/ssl/private/gateway.key".into(),
            min_version: "TLSv1.3".into(),
        },
        endpoints: vec![
            Endpoint {
                path: "/api/v1/users".into(),
                method: "GET".into(),
                rate_limit: 1000,
            },
            Endpoint {
                path: "/api/v1/orders".into(),
                method: "POST".into(),
                rate_limit: 500,
            },
            Endpoint {
                path: "/health".into(),
                method: "GET".into(),
                rate_limit: 5000,
            },
        ],
        max_connections: 10_000,
        idle_timeout_secs: 120,
    };
    console.print(&gateway.to_inspect());
}

#[cfg(not(feature = "derive"))]
fn main() {
    eprintln!(
        "This example requires the 'derive' feature: \
         cargo run --example derive_inspect --features derive"
    );
}
