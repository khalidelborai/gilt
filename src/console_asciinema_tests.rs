//! TDD tests for the asciinema v2 `.cast` export feature.
//!
//! These tests are gated on `#[cfg(feature = "asciinema")]` and live in a
//! dedicated file wired into `console_asciinema.rs` via `#[cfg(test)] mod tests`.

#[cfg(feature = "asciinema")]
mod asciinema_tests {
    use crate::console::Console;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn cast_console() -> Console {
        Console::builder()
            .width(80)
            .height(24)
            .force_terminal(true)
            .no_color(true)
            .markup(true)
            .build()
    }

    // -----------------------------------------------------------------------
    // Test 1: injectable mock clock, two timed events
    //
    // RED: fails to compile because `begin_asciinema_record`,
    //      `with_asciinema_clock`, and `export_asciinema` don't exist yet.
    // -----------------------------------------------------------------------

    #[test]
    fn asciinema_timed_events_mock_clock() {
        use std::sync::{Arc, Mutex};

        // A mock clock that returns 0.0 on first call, 1.5 on second call,
        // and 1.5 on all subsequent calls.
        let counter = Arc::new(Mutex::new(0u32));
        let counter_clone = Arc::clone(&counter);
        let clock = move || -> f64 {
            let mut c = counter_clone.lock().unwrap();
            let v = *c;
            *c += 1;
            match v {
                0 => 0.0,
                _ => 1.5,
            }
        };

        let mut console = cast_console().with_asciinema_clock(clock);
        console.begin_asciinema_record();
        console.print_text("a"); // t = 0.0 (first clock call)
        console.print_text("b"); // t = 1.5 (second clock call)

        let cast = console.export_asciinema(Some("demo"));

        // --- Parse and verify -------------------------------------------------

        let lines: Vec<&str> = cast.lines().collect();
        assert!(
            lines.len() >= 3,
            "need header + 2 events; got {}",
            lines.len()
        );

        // Line 0: header JSON object
        let header: serde_json::Value =
            serde_json::from_str(lines[0]).expect("line 0 must be valid JSON");
        assert_eq!(header["version"], 2, "version must be 2");
        assert_eq!(
            header["width"].as_u64().unwrap_or(0),
            80,
            "width must be 80"
        );
        assert_eq!(header["title"], "demo", "title must match");

        // Events start at line 1.
        let ev0: serde_json::Value =
            serde_json::from_str(lines[1]).expect("event line 1 must be valid JSON");
        let ev1: serde_json::Value =
            serde_json::from_str(lines[2]).expect("event line 2 must be valid JSON");

        // Each event must be a 3-element array: [f64, "o", string]
        assert!(ev0.is_array() && ev0.as_array().unwrap().len() == 3);
        assert!(ev1.is_array() && ev1.as_array().unwrap().len() == 3);

        assert_eq!(ev0[1], "o", "event type must be 'o'");
        assert_eq!(ev1[1], "o", "event type must be 'o'");

        // Timestamps
        let t0 = ev0[0].as_f64().expect("timestamp must be f64");
        let t1 = ev1[0].as_f64().expect("timestamp must be f64");
        assert!(
            (t0 - 0.0).abs() < 1e-6,
            "first timestamp must be ~0.0; got {}",
            t0
        );
        assert!(
            (t1 - 1.5).abs() < 1e-6,
            "second timestamp must be ~1.5; got {}",
            t1
        );

        // Output strings must contain the rendered text
        let s0 = ev0[2].as_str().expect("output must be a string");
        let s1 = ev1[2].as_str().expect("output must be a string");
        assert!(
            s0.contains('a'),
            "first event output must contain 'a'; got {:?}",
            s0
        );
        assert!(
            s1.contains('b'),
            "second event output must contain 'b'; got {:?}",
            s1
        );
    }

    // -----------------------------------------------------------------------
    // Test 2: fallback — no timed events, use record_buffer at t=0
    // -----------------------------------------------------------------------

    #[test]
    fn asciinema_fallback_no_timed_events() {
        // Build a console with record(true) but DON'T call begin_asciinema_record.
        // export_asciinema should fall back to a single t=0 event built from the
        // record_buffer.
        let mut console = Console::builder()
            .width(80)
            .height(24)
            .force_terminal(true)
            .no_color(true)
            .markup(false)
            .record(true)
            .build();

        console.print_text("static report line 1");
        console.print_text("static report line 2");

        let cast = console.export_asciinema(Some("static"));

        let lines: Vec<&str> = cast.lines().collect();
        // Must have at least a header + one event line.
        assert!(
            lines.len() >= 2,
            "need at least header + 1 event; got {}",
            lines.len()
        );

        let header: serde_json::Value =
            serde_json::from_str(lines[0]).expect("header must be valid JSON");
        assert_eq!(header["version"], 2);
        assert_eq!(header["title"], "static");

        // The single fallback event at t=0
        let ev: serde_json::Value =
            serde_json::from_str(lines[1]).expect("event line 1 must be valid JSON");
        assert!(ev.is_array() && ev.as_array().unwrap().len() == 3);
        assert_eq!(ev[1], "o");
        let t = ev[0].as_f64().expect("timestamp must be f64");
        assert!((t - 0.0).abs() < 1e-9, "fallback timestamp must be 0.0");

        let s = ev[2].as_str().expect("output must be string");
        assert!(
            s.contains("static report line 1"),
            "fallback output must contain record content; got {:?}",
            s
        );
    }

    // -----------------------------------------------------------------------
    // Test 3: JSON escaping — output containing `"` and ESC bytes
    // -----------------------------------------------------------------------

    #[test]
    fn asciinema_json_escaping() {
        use std::sync::{Arc, Mutex};

        let counter = Arc::new(Mutex::new(0u32));
        let counter_clone = Arc::clone(&counter);
        let clock = move || -> f64 {
            let mut c = counter_clone.lock().unwrap();
            let v = *c;
            *c += 1;
            v as f64 * 0.1
        };

        let mut console = Console::builder()
            .width(80)
            .height(24)
            .force_terminal(true)
            .no_color(false) // keep ANSI so ESC bytes appear
            .markup(false)
            .build()
            .with_asciinema_clock(clock);

        console.begin_asciinema_record();
        // Print something that will produce ANSI escape codes (ESC = \x1b)
        // and also contains a literal double-quote character in the text.
        console.print_text("say \"hello\"");

        let cast = console.export_asciinema(None);

        // The whole cast must be valid JSON lines (no unescaped control chars).
        for (i, line) in cast.lines().enumerate() {
            serde_json::from_str::<serde_json::Value>(line).unwrap_or_else(|e| {
                panic!("line {} is not valid JSON: {} | raw: {:?}", i, e, line)
            });
        }

        let lines: Vec<&str> = cast.lines().collect();
        if lines.len() >= 2 {
            let ev: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
            let s = ev[2].as_str().unwrap_or("");
            // Must contain the text content (quotes JSON-escaped correctly by serde_json)
            assert!(
                s.contains("hello"),
                "output must contain 'hello'; got {:?}",
                s
            );
        }
    }

    // -----------------------------------------------------------------------
    // Test 4: save_asciinema writes a file (native only)
    // -----------------------------------------------------------------------

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn asciinema_save_to_file() {
        use std::io::Read as _;

        let mut console = Console::builder()
            .width(40)
            .height(10)
            .force_terminal(true)
            .no_color(true)
            .markup(false)
            .record(true)
            .build();
        console.print_text("save test");

        let path = std::env::temp_dir().join("gilt_asciinema_test.cast");
        console
            .save_asciinema(&path, Some("save-test"))
            .expect("save_asciinema must not error");

        let mut contents = String::new();
        std::fs::File::open(&path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        let _ = std::fs::remove_file(&path); // cleanup

        assert!(contents.contains("\"version\":2") || contents.contains("\"version\": 2"));
        assert!(contents.contains("save test"));
    }
}
