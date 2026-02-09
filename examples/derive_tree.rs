//! Demonstrates gilt's #[derive(Tree)] proc macro.
//! Run with: cargo run --example derive_tree --features derive

#[cfg(feature = "derive")]
fn main() {
    use gilt::prelude::*;
    use gilt::Tree as TreeDerive;

    let mut console = Console::new();

    // ── 1. File system tree — recursive struct with label and children ───
    console.print_text("\n[bold cyan]1. File system tree — basic recursive structure[/]");

    #[derive(TreeDerive)]
    struct FileEntry {
        #[tree(label)]
        name: String,
        #[tree(children)]
        entries: Vec<FileEntry>,
    }

    let root = FileEntry {
        name: "my-project/".into(),
        entries: vec![
            FileEntry {
                name: "src/".into(),
                entries: vec![
                    FileEntry {
                        name: "main.rs".into(),
                        entries: vec![],
                    },
                    FileEntry {
                        name: "lib.rs".into(),
                        entries: vec![],
                    },
                    FileEntry {
                        name: "utils/".into(),
                        entries: vec![
                            FileEntry {
                                name: "helpers.rs".into(),
                                entries: vec![],
                            },
                            FileEntry {
                                name: "config.rs".into(),
                                entries: vec![],
                            },
                        ],
                    },
                ],
            },
            FileEntry {
                name: "tests/".into(),
                entries: vec![
                    FileEntry {
                        name: "integration.rs".into(),
                        entries: vec![],
                    },
                    FileEntry {
                        name: "unit.rs".into(),
                        entries: vec![],
                    },
                ],
            },
            FileEntry {
                name: "Cargo.toml".into(),
                entries: vec![],
            },
            FileEntry {
                name: "README.md".into(),
                entries: vec![],
            },
        ],
    };
    console.print(&root.to_tree());

    // ── 2. With leaf details — file size shown as leaf nodes ────────────
    console.print_text("\n[bold cyan]2. With leaf details — file sizes as leaf nodes[/]");

    #[derive(TreeDerive)]
    struct SizedEntry {
        #[tree(label)]
        name: String,
        #[tree(leaf)]
        size_bytes: u64,
        #[tree(children)]
        children: Vec<SizedEntry>,
    }

    let project = SizedEntry {
        name: "dist/".into(),
        size_bytes: 0,
        children: vec![
            SizedEntry {
                name: "index.html".into(),
                size_bytes: 4_096,
                children: vec![],
            },
            SizedEntry {
                name: "assets/".into(),
                size_bytes: 0,
                children: vec![
                    SizedEntry {
                        name: "app.js".into(),
                        size_bytes: 128_512,
                        children: vec![],
                    },
                    SizedEntry {
                        name: "style.css".into(),
                        size_bytes: 24_320,
                        children: vec![],
                    },
                    SizedEntry {
                        name: "logo.png".into(),
                        size_bytes: 51_200,
                        children: vec![],
                    },
                ],
            },
            SizedEntry {
                name: "manifest.json".into(),
                size_bytes: 512,
                children: vec![],
            },
        ],
    };
    console.print(&project.to_tree());

    // ── 3. Styled tree — colored nodes and guide lines ──────────────────
    console.print_text("\n[bold cyan]3. Styled tree — colored output[/]");

    #[derive(TreeDerive)]
    #[tree(style = "bold green", guide_style = "dim cyan")]
    struct StyledEntry {
        #[tree(label)]
        name: String,
        #[tree(leaf)]
        description: String,
        #[tree(children)]
        items: Vec<StyledEntry>,
    }

    let menu = StyledEntry {
        name: "Application Menu".into(),
        description: "Top-level navigation".into(),
        items: vec![
            StyledEntry {
                name: "File".into(),
                description: "File operations".into(),
                items: vec![
                    StyledEntry {
                        name: "New".into(),
                        description: "Create new document".into(),
                        items: vec![],
                    },
                    StyledEntry {
                        name: "Open".into(),
                        description: "Open existing file".into(),
                        items: vec![],
                    },
                    StyledEntry {
                        name: "Save".into(),
                        description: "Save current file".into(),
                        items: vec![],
                    },
                ],
            },
            StyledEntry {
                name: "Edit".into(),
                description: "Edit operations".into(),
                items: vec![
                    StyledEntry {
                        name: "Undo".into(),
                        description: "Undo last action".into(),
                        items: vec![],
                    },
                    StyledEntry {
                        name: "Redo".into(),
                        description: "Redo last undo".into(),
                        items: vec![],
                    },
                ],
            },
            StyledEntry {
                name: "View".into(),
                description: "Display settings".into(),
                items: vec![],
            },
        ],
    };
    console.print(&menu.to_tree());

    // ── 4. Org chart — department / team / person hierarchy ─────────────
    console.print_text("\n[bold cyan]4. Org chart — department / team / person[/]");

    #[derive(TreeDerive)]
    #[tree(style = "bold", guide_style = "dim blue")]
    struct OrgNode {
        #[tree(label)]
        title: String,
        #[tree(leaf)]
        role: String,
        #[tree(children)]
        reports: Vec<OrgNode>,
    }

    let org = OrgNode {
        title: "Acme Corp".into(),
        role: "Organization".into(),
        reports: vec![
            OrgNode {
                title: "Engineering".into(),
                role: "Department".into(),
                reports: vec![
                    OrgNode {
                        title: "Platform Team".into(),
                        role: "Team".into(),
                        reports: vec![
                            OrgNode {
                                title: "Alice Chen".into(),
                                role: "Staff Engineer".into(),
                                reports: vec![],
                            },
                            OrgNode {
                                title: "Bob Martinez".into(),
                                role: "Senior Engineer".into(),
                                reports: vec![],
                            },
                        ],
                    },
                    OrgNode {
                        title: "Frontend Team".into(),
                        role: "Team".into(),
                        reports: vec![
                            OrgNode {
                                title: "Carol Washington".into(),
                                role: "Tech Lead".into(),
                                reports: vec![],
                            },
                            OrgNode {
                                title: "David Kim".into(),
                                role: "Engineer".into(),
                                reports: vec![],
                            },
                        ],
                    },
                ],
            },
            OrgNode {
                title: "Design".into(),
                role: "Department".into(),
                reports: vec![
                    OrgNode {
                        title: "UX Team".into(),
                        role: "Team".into(),
                        reports: vec![
                            OrgNode {
                                title: "Eve Johnson".into(),
                                role: "Senior Designer".into(),
                                reports: vec![],
                            },
                        ],
                    },
                ],
            },
            OrgNode {
                title: "Product".into(),
                role: "Department".into(),
                reports: vec![
                    OrgNode {
                        title: "Frank Lee".into(),
                        role: "Product Manager".into(),
                        reports: vec![],
                    },
                    OrgNode {
                        title: "Grace Patel".into(),
                        role: "Product Analyst".into(),
                        reports: vec![],
                    },
                ],
            },
        ],
    };
    console.print(&org.to_tree());
}

#[cfg(not(feature = "derive"))]
fn main() {
    eprintln!(
        "This example requires the 'derive' feature: cargo run --example derive_tree --features derive"
    );
}
