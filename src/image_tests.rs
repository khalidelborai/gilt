//! Tests for the `Image` renderable (TDD: written before implementation).
//!
//! Run with: `cargo nextest run --all-features -E 'test(image)'`

#[cfg(test)]
mod tests {
    use crate::console::Console;
    use crate::console_caps::ConsoleCapabilities;
    use crate::image::Image;

    // -----------------------------------------------------------------------
    // Helper: build a 2×2 RGBA image (red, green, blue, white pixels).
    //   Layout: [red, green] on top row, [blue, white] on bottom row
    //   RGBA bytes: R=255 G=0 B=0 A=255, R=0 G=255 B=0 A=255,
    //               R=0 G=0 B=255 A=255, R=255 G=255 B=255 A=255
    // -----------------------------------------------------------------------
    fn rgba_2x2() -> Vec<u8> {
        vec![
            255, 0, 0, 255, // top-left:  red
            0, 255, 0, 255, // top-right: green
            0, 0, 255, 255, // bot-left:  blue
            255, 255, 255, 255, // bot-right: white
        ]
    }

    // -----------------------------------------------------------------------
    // RED TEST 1 — halfblock core (no `image` crate dep required)
    //
    // A 2-wide × 2-tall pixel grid with width(2) target cells.
    // Each cell represents 1 column × 2 rows of pixels → 2 cells total.
    // halfblock: cell 0 → fg=red  bg=blue  (▀ with ESC[38;2;255;0;0m fg, ESC[48;2;0;0;255m bg)
    //            cell 1 → fg=green bg=white (▀ …)
    // Then a newline.
    // -----------------------------------------------------------------------
    #[test]
    fn halfblock_output_contains_upper_half_block_and_truecolor_escapes() {
        let img = Image::from_rgba(2, 2, rgba_2x2()).width(2);

        let mut console = Console::builder()
            .no_color(false)
            .force_terminal(true)
            .color_system("truecolor")
            .width(80)
            .build();

        // Force kitty=false AND iterm=false so halfblock is used regardless of the
        // test runner's environment (the developer may be inside ghostty/kitty/iTerm).
        console.set_capabilities(ConsoleCapabilities {
            kitty: false,
            iterm: false,
            ..console.capabilities().clone()
        });

        console.begin_capture();
        console.print(&img);
        let output = console.end_capture();

        // halfblock character must appear
        assert!(
            output.contains('\u{2580}'),
            "halfblock path must emit \u{2580}; got: {:?}",
            output
        );
        // truecolor fg SGR params for red pixel (top-left) — 38;2;255;0;0
        // The console may combine fg+bg into one SGR sequence, so we search
        // for the SGR parameter substring rather than the full standalone sequence.
        assert!(
            output.contains("38;2;255;0;0"),
            "expected fg=red truecolor params; got: {:?}",
            output
        );
        // truecolor bg SGR params for blue pixel (bottom-left) — 48;2;0;0;255
        assert!(
            output.contains("48;2;0;0;255"),
            "expected bg=blue truecolor params; got: {:?}",
            output
        );
        // truecolor fg SGR params for green pixel (top-right) — 38;2;0;255;0
        assert!(
            output.contains("38;2;0;255;0"),
            "expected fg=green truecolor params; got: {:?}",
            output
        );
        // truecolor bg SGR params for white pixel (bottom-right) — 48;2;255;255;255
        assert!(
            output.contains("48;2;255;255;255"),
            "expected bg=white truecolor params; got: {:?}",
            output
        );
        // NO Kitty APC — kitty was forced off
        assert!(
            !output.contains("\x1b_G"),
            "halfblock path must NOT emit Kitty APC"
        );
    }

    // -----------------------------------------------------------------------
    // RED TEST 2 — Kitty path
    //
    // Build a console whose capabilities have kitty=true and is NOT recording.
    // Rendering Image::from_rgba must produce a segment whose text contains
    // the Kitty APC introducer \x1b_G and the base64 of the raw RGBA bytes.
    // -----------------------------------------------------------------------
    #[test]
    fn kitty_path_emits_apc_and_base64_of_pixels() {
        let rgba = rgba_2x2();
        let img = Image::from_rgba(2, 2, rgba.clone()).width(2);

        // Build a console with kitty capability forced
        let mut console = Console::builder()
            .no_color(false)
            .force_terminal(true)
            .color_system("truecolor")
            .width(80)
            .build();

        // Override capabilities with kitty=true after build
        console.set_capabilities(ConsoleCapabilities {
            kitty: true,
            ..console.capabilities().clone()
        });

        console.begin_capture();
        console.print(&img);
        let output = console.end_capture();

        // Must contain Kitty APC introducer
        assert!(
            output.contains("\x1b_G"),
            "Kitty path must emit \\x1b_G APC; got: {:?}",
            output
        );
        // The full APC must survive — the introducer AND the ST terminator
        // `\x1b\\`. (Regression guard: a plain text segment gets width-cropped,
        // truncating the base64 payload and dropping the terminator.)
        assert!(
            output.contains("\x1b\\"),
            "Kitty APC must reach its ST terminator (not be width-cropped); got len {}",
            output.len()
        );
        // Must carry the display cell-box size (c=cols, r=rows) so Kitty scales
        // the image to the requested cells instead of a tiny native-pixel thumb.
        assert!(
            output.contains(",c=") && output.contains(",r="),
            "Kitty APC must specify c=/r= cell dimensions; got: {:?}",
            output
        );
        // Must NOT contain ▀ (halfblock chars)
        assert!(
            !output.contains('▀'),
            "Kitty path must NOT emit halfblock ▀"
        );
    }

    // -----------------------------------------------------------------------
    // RED TEST 3 — recording console always uses halfblock (for HTML/SVG export)
    // -----------------------------------------------------------------------
    #[test]
    fn recording_console_uses_halfblock_not_kitty() {
        let img = Image::from_rgba(2, 2, rgba_2x2()).width(2);

        let mut console = Console::builder()
            .no_color(false)
            .force_terminal(true)
            .color_system("truecolor")
            .width(80)
            .record(true) // recording mode → halfblock regardless
            .build();

        // Even with kitty=true, recording uses halfblock
        console.set_capabilities(ConsoleCapabilities {
            kitty: true,
            ..console.capabilities().clone()
        });

        console.begin_capture();
        console.print(&img);
        let output = console.end_capture();

        assert!(
            output.contains('▀'),
            "recording console must use halfblock for export; got: {:?}",
            output
        );
        assert!(
            !output.contains("\x1b_G"),
            "recording console must NOT emit Kitty APC (breaks HTML export)"
        );
    }

    // -----------------------------------------------------------------------
    // RED TEST 4 — ConsoleCapabilities detects kitty from env
    // -----------------------------------------------------------------------
    #[test]
    fn caps_kitty_flag_from_xterm_kitty() {
        let caps = ConsoleCapabilities::from_env_parts(
            Some("truecolor"),
            Some("xterm-kitty"),
            true,
            None,
            None, // kitty_window_id
            None, // term_program
        );
        assert!(caps.kitty, "TERM=xterm-kitty should set kitty=true");
    }

    #[test]
    fn caps_kitty_flag_from_kitty_window_id() {
        let caps = ConsoleCapabilities::from_env_parts(
            None,
            None,
            true,
            None,
            Some("42"), // KITTY_WINDOW_ID set
            None,
        );
        assert!(caps.kitty, "KITTY_WINDOW_ID set should set kitty=true");
    }

    #[test]
    fn caps_kitty_flag_from_wezterm() {
        let caps = ConsoleCapabilities::from_env_parts(
            None,
            None,
            true,
            None,
            None,
            Some("WezTerm"), // TERM_PROGRAM=WezTerm
        );
        assert!(caps.kitty, "TERM_PROGRAM=WezTerm should set kitty=true");
    }

    #[test]
    fn caps_kitty_flag_from_ghostty() {
        let caps =
            ConsoleCapabilities::from_env_parts(None, None, true, None, None, Some("ghostty"));
        assert!(caps.kitty, "TERM_PROGRAM=ghostty should set kitty=true");
    }

    #[test]
    fn caps_iterm_flag_from_term_program() {
        let caps =
            ConsoleCapabilities::from_env_parts(None, None, true, None, None, Some("iTerm.app"));
        assert!(caps.iterm, "TERM_PROGRAM=iTerm.app should set iterm=true");
    }

    #[test]
    fn caps_no_kitty_on_plain_term() {
        let caps = ConsoleCapabilities::from_env_parts(
            Some("truecolor"),
            Some("xterm-256color"),
            true,
            None,
            None,
            None,
        );
        assert!(
            !caps.kitty,
            "plain xterm-256color should NOT set kitty=true"
        );
        assert!(
            !caps.iterm,
            "plain xterm-256color should NOT set iterm=true"
        );
    }

    // -----------------------------------------------------------------------
    // iTerm2 path (OSC 1337). Requires `inline-images` for PNG encoding.
    //
    // With iterm=true, kitty=false, and not recording, Image must emit an
    // `ESC ]1337;File=inline=1;…:<base64 PNG> BEL` sequence — not halfblock,
    // not Kitty APC.
    // -----------------------------------------------------------------------
    #[cfg(feature = "inline-images")]
    #[test]
    fn iterm_path_emits_osc1337_with_png_base64() {
        let img = Image::from_rgba(2, 2, rgba_2x2()).width(2);

        let mut console = Console::builder()
            .no_color(false)
            .force_terminal(true)
            .color_system("truecolor")
            .width(80)
            .build();

        console.set_capabilities(ConsoleCapabilities {
            kitty: false,
            iterm: true,
            ..console.capabilities().clone()
        });

        console.begin_capture();
        console.print(&img);
        let output = console.end_capture();

        // OSC 1337 inline-image introducer + args.
        assert!(
            output.contains("\x1b]1337;File=inline=1"),
            "iTerm2 path must emit OSC 1337 File=inline=1; got: {:?}",
            output
        );
        // The base64 of any PNG begins with "iVBORw0KGgo" (base64 of the PNG
        // magic \x89PNG\r\n\x1a\n) — proves the payload is a real PNG.
        assert!(
            output.contains("iVBORw0KGgo"),
            "iTerm2 payload must be a base64-encoded PNG; got: {:?}",
            output
        );
        // BEL terminator for the OSC sequence.
        assert!(
            output.contains('\u{0007}'),
            "iTerm2 OSC 1337 must be BEL-terminated; got: {:?}",
            output
        );
        // Must NOT fall back to halfblock or use Kitty.
        assert!(
            !output.contains('▀'),
            "iTerm2 path must NOT emit halfblock ▀"
        );
        assert!(
            !output.contains("\x1b_G"),
            "iTerm2 path must NOT emit Kitty APC"
        );
    }

    // Recording mode wins over iTerm2: export must stay halfblock styled text.
    #[cfg(feature = "inline-images")]
    #[test]
    fn recording_console_uses_halfblock_not_iterm() {
        let img = Image::from_rgba(2, 2, rgba_2x2()).width(2);

        let mut console = Console::builder()
            .no_color(false)
            .force_terminal(true)
            .color_system("truecolor")
            .width(80)
            .record(true)
            .build();

        console.set_capabilities(ConsoleCapabilities {
            iterm: true,
            kitty: false,
            ..console.capabilities().clone()
        });

        console.begin_capture();
        console.print(&img);
        let output = console.end_capture();

        assert!(
            output.contains('▀'),
            "recording console must use halfblock for export; got: {:?}",
            output
        );
        assert!(
            !output.contains("\x1b]1337"),
            "recording console must NOT emit iTerm2 OSC 1337 (breaks HTML export)"
        );
    }

    // -----------------------------------------------------------------------
    // Sixel path (DCS). Dep-free — no `inline-images` feature needed.
    //
    // With sixel=true, kitty/iterm=false, and not recording, Image must emit a
    // `ESC P … q … ESC \\` sixel stream with colour registers — not halfblock,
    // not Kitty, not iTerm2.
    // -----------------------------------------------------------------------
    #[test]
    fn sixel_path_emits_dcs_stream() {
        let img = Image::from_rgba(2, 2, rgba_2x2()).width(2);

        let mut console = Console::builder()
            .no_color(false)
            .force_terminal(true)
            .color_system("truecolor")
            .width(80)
            .build();

        console.set_capabilities(ConsoleCapabilities {
            kitty: false,
            iterm: false,
            sixel: true,
            ..console.capabilities().clone()
        });

        console.begin_capture();
        console.print(&img);
        let output = console.end_capture();

        // DCS sixel introducer (ESC P ... q) and raster attributes.
        assert!(
            output.contains("\x1bP") && output.contains('q'),
            "Sixel path must emit a DCS `\\x1bP…q` introducer; got: {:?}",
            output
        );
        assert!(
            output.contains("\"1;1;"),
            "Sixel stream must carry raster attributes; got: {:?}",
            output
        );
        // At least one colour register definition (#n;2;r;g;b).
        assert!(
            output.contains(";2;"),
            "Sixel stream must define RGB colour registers; got: {:?}",
            output
        );
        // ST terminator.
        assert!(
            output.contains("\x1b\\"),
            "Sixel stream must be ST-terminated; got: {:?}",
            output
        );
        // Not any other protocol / fallback.
        assert!(
            !output.contains('▀'),
            "Sixel path must NOT emit halfblock ▀"
        );
        assert!(
            !output.contains("\x1b_G"),
            "Sixel path must NOT emit Kitty APC"
        );
        assert!(
            !output.contains("\x1b]1337"),
            "Sixel path must NOT emit iTerm2 OSC 1337"
        );
    }
}
