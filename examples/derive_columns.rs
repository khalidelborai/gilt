//! Demonstrates gilt's #[derive(Columns)] proc macro.
//! Run with: cargo run --example derive_columns --features derive
//!
//! The Columns derive generates two methods:
//!   - `to_card(&self) -> Panel`  — renders the struct as a labeled key-value panel
//!   - `to_columns(items: &[Self]) -> Columns` — lays out a slice of items in auto-fit columns
//!
//! Struct-level `#[columns(...)]` controls the column layout (count, padding, equal, expand, title).
//! Field-level `#[field(...)]` controls labels, styles, and skip behavior.

#[cfg(feature = "derive")]
fn main() {
    use gilt::prelude::*;
    use gilt::DeriveColumns;

    let mut console = Console::builder()
        .width(100)
        .force_terminal(true)
        .no_color(false)
        .build();

    // ── 1. Basic struct — fields become Title Case labels in each card ────
    console.rule(Some("1. Basic derive — fields as columns of cards"));

    #[derive(DeriveColumns)]
    struct Language {
        name: String,
        paradigm: String,
        year: u32,
    }

    let langs = vec![
        Language {
            name: "Rust".into(),
            paradigm: "Systems".into(),
            year: 2010,
        },
        Language {
            name: "Haskell".into(),
            paradigm: "Functional".into(),
            year: 1990,
        },
        Language {
            name: "Python".into(),
            paradigm: "Multi-paradigm".into(),
            year: 1991,
        },
        Language {
            name: "Go".into(),
            paradigm: "Concurrent".into(),
            year: 2009,
        },
        Language {
            name: "Zig".into(),
            paradigm: "Systems".into(),
            year: 2016,
        },
        Language {
            name: "Elixir".into(),
            paradigm: "Functional".into(),
            year: 2011,
        },
    ];
    console.print(&Language::to_columns(&langs));

    // ── 2. Title and layout options ──────────────────────────────────────
    console.rule(Some("2. Title, equal columns, and padding"));

    #[derive(DeriveColumns)]
    #[columns(title = "Cloud Regions", equal, expand, padding = 2)]
    struct Region {
        #[field(label = "Region", style = "bold cyan")]
        name: String,
        #[field(label = "AZs")]
        availability_zones: u32,
        #[field(label = "Status", style = "green")]
        status: String,
    }

    let regions = vec![
        Region {
            name: "us-east-1".into(),
            availability_zones: 6,
            status: "Healthy".into(),
        },
        Region {
            name: "eu-west-1".into(),
            availability_zones: 3,
            status: "Healthy".into(),
        },
        Region {
            name: "ap-south-1".into(),
            availability_zones: 3,
            status: "Degraded".into(),
        },
        Region {
            name: "us-west-2".into(),
            availability_zones: 4,
            status: "Healthy".into(),
        },
    ];
    console.print(&Region::to_columns(&regions));

    // ── 3. Styled fields with skip ───────────────────────────────────────
    console.rule(Some("3. Styled fields and #[field(skip)]"));

    #[derive(DeriveColumns)]
    #[columns(column_count = 3, padding = 1)]
    struct Dependency {
        #[field(label = "Crate", style = "bold yellow")]
        name: String,
        #[field(label = "Version", style = "dim")]
        version: String,
        #[field(label = "Features")]
        features: String,
        #[field(skip)]
        #[allow(dead_code)]
        checksum: String,
    }

    let deps = vec![
        Dependency {
            name: "serde".into(),
            version: "1.0.210".into(),
            features: "derive, std".into(),
            checksum: "abc123".into(),
        },
        Dependency {
            name: "tokio".into(),
            version: "1.40.0".into(),
            features: "full".into(),
            checksum: "def456".into(),
        },
        Dependency {
            name: "regex".into(),
            version: "1.10.6".into(),
            features: "default".into(),
            checksum: "ghi789".into(),
        },
        Dependency {
            name: "clap".into(),
            version: "4.5.17".into(),
            features: "derive, env".into(),
            checksum: "jkl012".into(),
        },
        Dependency {
            name: "tracing".into(),
            version: "0.1.40".into(),
            features: "log".into(),
            checksum: "mno345".into(),
        },
        Dependency {
            name: "anyhow".into(),
            version: "1.0.89".into(),
            features: "default".into(),
            checksum: "pqr678".into(),
        },
    ];
    console.print(&Dependency::to_columns(&deps));

    // ── 4. Practical example: team roster ────────────────────────────────
    console.rule(Some("4. Team roster — practical use case"));

    #[derive(DeriveColumns)]
    #[columns(title = "Engineering Team", equal, expand, padding = 2)]
    struct TeamMember {
        #[field(label = "Name", style = "bold")]
        name: String,
        #[field(label = "Role", style = "cyan")]
        role: String,
        #[field(label = "Squad")]
        squad: String,
        #[field(label = "Location", style = "dim")]
        location: String,
        #[field(skip)]
        #[allow(dead_code)]
        employee_id: u64,
    }

    let team = vec![
        TeamMember {
            name: "Alice Chen".into(),
            role: "Staff Engineer".into(),
            squad: "Platform".into(),
            location: "San Francisco".into(),
            employee_id: 1001,
        },
        TeamMember {
            name: "Bob Martinez".into(),
            role: "Senior Engineer".into(),
            squad: "Backend".into(),
            location: "Austin".into(),
            employee_id: 1002,
        },
        TeamMember {
            name: "Carol Kim".into(),
            role: "Tech Lead".into(),
            squad: "Frontend".into(),
            location: "New York".into(),
            employee_id: 1003,
        },
        TeamMember {
            name: "David Singh".into(),
            role: "Engineer".into(),
            squad: "Platform".into(),
            location: "London".into(),
            employee_id: 1004,
        },
        TeamMember {
            name: "Eva Novak".into(),
            role: "Senior Engineer".into(),
            squad: "Data".into(),
            location: "Berlin".into(),
            employee_id: 1005,
        },
        TeamMember {
            name: "Frank Osei".into(),
            role: "Engineer".into(),
            squad: "Backend".into(),
            location: "Toronto".into(),
            employee_id: 1006,
        },
    ];
    console.print(&TeamMember::to_columns(&team));

    // ── 5. Individual card rendering ─────────────────────────────────────
    console.rule(Some("5. Single card via to_card()"));

    let alice = &team[0];
    console.print(&alice.to_card());
}

#[cfg(not(feature = "derive"))]
fn main() {
    eprintln!(
        "This example requires the 'derive' feature: cargo run --example derive_columns --features derive"
    );
}
