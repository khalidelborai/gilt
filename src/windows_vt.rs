//! Windows Virtual Terminal (VT) processing enablement.
//!
//! On Windows, terminals need
//! [`ENABLE_VIRTUAL_TERMINAL_PROCESSING`](https://docs.microsoft.com/en-us/windows/console/setconsolemode)
//! set on the stdout handle before ANSI escape sequences are rendered rather
//! than printed as raw text.
//!
//! This module provides `enable_windows_vt()`, gated behind the `windows-vt`
//! feature flag.  The function is deliberately **opt-in** (not in `default`)
//! so that:
//!
//! 1. The default dependency tree stays dep-light (no `windows-sys` for most
//!    users).
//! 2. Non-Windows and WASM builds never pull the dependency at all — the
//!    `[target.'cfg(windows)'.dependencies]` declaration in `Cargo.toml` plus
//!    the `cfg(windows)` gate on the real implementation mean the linker never
//!    sees `windows-sys` on non-Windows hosts.
//!
//! # Usage
//!
//! ```toml
//! gilt = { version = "1.8", features = ["windows-vt"] }
//! ```
//!
//! ```rust
//! gilt::windows_vt::enable_windows_vt();
//! ```
//!
//! On non-Windows platforms the function is a harmless no-op.  It is called
//! automatically from `Console::new()` when the feature is enabled, so most
//! callers never need to invoke it directly.

// ---------------------------------------------------------------------------
// Real Windows implementation (only compiled on Windows with windows-vt feature)
// ---------------------------------------------------------------------------

#[cfg(all(feature = "windows-vt", target_os = "windows"))]
mod inner {
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
    use windows_sys::Win32::System::Console::{
        GetConsoleMode, GetStdHandle, SetConsoleMode, ENABLE_PROCESSED_OUTPUT,
        ENABLE_VIRTUAL_TERMINAL_PROCESSING, STD_OUTPUT_HANDLE,
    };

    /// Enable Windows Virtual Terminal processing on stdout.
    ///
    /// Calls `SetConsoleMode` to OR in `ENABLE_VIRTUAL_TERMINAL_PROCESSING`
    /// and `ENABLE_PROCESSED_OUTPUT` on the stdout handle.  Errors are silently
    /// ignored — best-effort, idempotent.
    pub fn enable_windows_vt() {
        // SAFETY: GetStdHandle and GetConsoleMode are simple Win32 API calls
        // with well-defined behaviour; the handle is not used after this fn.
        unsafe {
            let handle = GetStdHandle(STD_OUTPUT_HANDLE);
            if handle == INVALID_HANDLE_VALUE || handle.is_null() {
                return;
            }
            let mut mode: u32 = 0;
            if GetConsoleMode(handle, &mut mode) == 0 {
                return; // not a console (e.g. piped) — ignore
            }
            let new_mode = mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING | ENABLE_PROCESSED_OUTPUT;
            // Best-effort: ignore the return value.
            SetConsoleMode(handle, new_mode);
        }
    }
}

// ---------------------------------------------------------------------------
// No-op stub for all other targets (non-Windows, or windows-vt feature off)
// ---------------------------------------------------------------------------

#[cfg(not(all(feature = "windows-vt", target_os = "windows")))]
mod inner {
    /// No-op stub for non-Windows targets or when the `windows-vt` feature is
    /// disabled.  The CSI escape sequences gilt emits are already handled by
    /// the terminal on Linux/macOS/WASM without any extra mode setup.
    #[inline(always)]
    pub fn enable_windows_vt() {}
}

// Re-export from the right inner module.
pub use inner::enable_windows_vt;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// `enable_windows_vt()` must be callable without panicking on any
    /// platform.  On non-Windows (where this test actually runs) it is a
    /// pure no-op, so the only assertion we need is that it doesn't panic.
    #[test]
    fn enable_windows_vt_no_panic_on_any_platform() {
        enable_windows_vt(); // must not panic
    }

    /// Compile-time check: the function has the expected signature.
    /// This also verifies it is callable from external code.
    #[test]
    fn enable_windows_vt_has_correct_signature() {
        let f: fn() = enable_windows_vt;
        f(); // callable, no arguments, no return value
    }

    /// On non-Windows the stub is `#[inline(always)]` and truly a no-op.
    /// This test verifies it can be called in a loop without side effects.
    #[test]
    fn enable_windows_vt_idempotent_noop_on_non_windows() {
        for _ in 0..3 {
            enable_windows_vt();
        }
        // No assertion needed — the test passes as long as it doesn't panic.
    }

    // --- Windows-only test (won't run on this host but must compile) ---

    /// Placeholder for the real Windows path.  This test is compiled only
    /// when `windows-vt` and `windows` are both active.
    #[cfg(all(feature = "windows-vt", target_os = "windows"))]
    #[test]
    fn enable_windows_vt_real_path_on_windows() {
        // On a real Windows host, calling this should either succeed or
        // silently ignore errors (e.g. when stdout is redirected).
        enable_windows_vt(); // must not panic even when piped
    }
}
