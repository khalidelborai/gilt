# gilt development & release tasks

# Run all checks (tests, clippy, fmt, docs)
check:
    cargo fmt --check
    cargo clippy --all-features
    cargo test --lib
    cargo test --doc
    cargo doc --no-deps
    RUSTFLAGS="-W missing-docs" cargo check

# Test with no default features (minimal build)
check-minimal:
    cargo test --lib --no-default-features

# Test all feature combinations
check-all:
    cargo test --lib
    cargo test --lib --no-default-features
    cargo test --lib --all-features
    cargo test --doc
    cargo clippy --all-features

# Run tests
test:
    cargo test --lib

# Run full test suite including doc tests
test-all:
    cargo test --lib
    cargo test --doc

# Check formatting
fmt-check:
    cargo fmt --check

# Apply formatting
fmt:
    cargo fmt

# Run clippy
lint:
    cargo clippy --all-features

# Build docs and open in browser
docs:
    cargo doc --no-deps --open

# Check missing docs
docs-coverage:
    RUSTFLAGS="-W missing-docs" cargo check

# Verify package contents (dry run)
package-check:
    cargo package --list --allow-dirty

# Release: bump version, commit, tag, push, publish
# Usage: just release 0.3.0
release version:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "==> Releasing gilt v{{version}}"

    # Validate version format
    if ! echo "{{version}}" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
        echo "Error: version must be semver (e.g. 0.3.0)"
        exit 1
    fi

    # Run all checks first
    echo "==> Running checks..."
    cargo fmt --check
    cargo clippy --all-features
    cargo test --lib
    cargo test --doc
    RUSTFLAGS="-W missing-docs" cargo check

    # Bump version in Cargo.toml
    echo "==> Bumping version to {{version}}..."
    sed -i 's/^version = ".*"/version = "{{version}}"/' Cargo.toml

    # Commit, tag, push
    echo "==> Committing and tagging..."
    git add Cargo.toml
    git commit -m "Release v{{version}}"
    git tag "v{{version}}"
    git push origin main
    git push origin "v{{version}}"

    # Create GitHub release
    echo "==> Creating GitHub release..."
    gh release create "v{{version}}" --title "v{{version}}" --generate-notes

    # Publish to crates.io
    echo "==> Publishing to crates.io..."
    cargo publish --allow-dirty

    echo "==> gilt v{{version}} released!"
    echo "    https://crates.io/crates/gilt/{{version}}"
    echo "    https://github.com/khalidelborai/gilt/releases/tag/v{{version}}"
