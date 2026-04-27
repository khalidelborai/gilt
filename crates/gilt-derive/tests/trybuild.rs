//! trybuild runner — exercises every derive macro across both
//! compile-pass (smoke tests) and compile-fail (error-message regression
//! guards) inputs.
//!
//! ## Regenerating expected stderr
//!
//! When stable rustc emits a different error format (rare), regenerate
//! the `.stderr` files alongside the same PR that bumps rustc:
//!
//! ```sh
//! TRYBUILD=overwrite cargo test -p gilt-derive --test trybuild
//! ```

#[test]
fn compile_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile_pass/*.rs");
}

#[test]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
