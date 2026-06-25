//! Inline terminal image rendering.
//!
//! Provides [`Image`], a [`Renderable`] that renders images inline in the
//! terminal using the best protocol the terminal supports:
//!
//! 1. **Kitty graphics protocol** (`\x1b_G…\x1b\\`) — when `capabilities().kitty`
//!    is true AND the console is not recording/exporting.
//! 2. **iTerm2 inline images** (OSC 1337 `\x1b]1337;File=…\x07`) — when
//!    `capabilities().iterm` is true, the console is not recording, and the
//!    `inline-images` feature is enabled (the image is PNG-encoded).
//! 3. **Sixel** (DCS `\x1bP…q…\x1b\\`) — when `capabilities().sixel` is true and
//!    not recording. Dep-free (no `image` crate); 6×6×6 palette, banded RLE.
//! 4. **Halfblock** (`▀`) — always available, always used during recording so
//!    that `export_html` / `export_svg` produce correct styled output.
//!
//! # Feature gate
//!
//! The `Image::from_rgba` constructor and the halfblock renderer are always
//! available (no extra features required).  The `Image::from_path` and
//! `Image::from_bytes` constructors require the `inline-images` feature, which
//! pulls in the [`image`](https://crates.io/crates/image) crate for decoding.
//!
//! # Example — default build (no `inline-images`)
//!
//! ```rust
//! use gilt::image::Image;
//! use gilt::console::Console;
//! use gilt::console_caps::ConsoleCapabilities;
//!
//! // 1×2 image: red pixel on top, blue pixel on bottom.
//! let img = Image::from_rgba(1, 2, vec![255, 0, 0, 255,  0, 0, 255, 255])
//!     .width(1);
//!
//! // Use record(true) to guarantee halfblock output regardless of terminal type.
//! let mut c = Console::builder()
//!     .no_color(false)
//!     .force_terminal(true)
//!     .color_system("truecolor")
//!     .width(80)
//!     .record(true)
//!     .build();
//! c.begin_capture();
//! c.print(&img);
//! let out = c.end_capture();
//! assert!(out.contains('\u{2580}')); // ▀ upper-half-block
//! ```

use crate::color::Color;
use crate::console::{Console, ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;

/// Append a run of `len` identical sixel values (`val`, a 6-bit mask) to `out`,
/// run-length-encoding runs of 4+ as `!<count><char>`. Sixel chars are
/// `0x3F + val` (`?`=empty … `~`=all six dots).
fn sixel_emit_run(out: &mut String, val: u8, len: usize) {
    if len == 0 {
        return;
    }
    let ch = (0x3F + val) as char;
    if len >= 4 {
        out.push('!');
        out.push_str(&len.to_string());
        out.push(ch);
    } else {
        for _ in 0..len {
            out.push(ch);
        }
    }
}

// ---------------------------------------------------------------------------
// Image
// ---------------------------------------------------------------------------

/// An inline terminal image renderable.
///
/// Construct with [`Image::from_rgba`] (always available) or with
/// [`Image::from_path`] / [`Image::from_bytes`] when the `inline-images`
/// feature is enabled.
///
/// Set a target display width with [`.width(cells)`](Image::width) and/or a
/// target display height with [`.height(cells)`](Image::height).  Aspect ratio
/// is preserved when only one dimension is given; if neither is given a default
/// of 40 columns (capped to the console width) is used.
#[derive(Debug, Clone)]
pub struct Image {
    /// Raw RGBA pixel data, row-major (4 bytes per pixel).
    pub(crate) rgba: Vec<u8>,
    /// Width of the pixel data in pixels.
    pub(crate) width_px: u32,
    /// Height of the pixel data in pixels.
    pub(crate) height_px: u32,
    /// Requested display width in terminal columns, if set.
    pub(crate) target_cols: Option<usize>,
    /// Requested display height in terminal rows, if set.
    pub(crate) target_rows: Option<usize>,
}

impl Image {
    // -- Constructors --------------------------------------------------------

    /// Create an `Image` from raw RGBA pixel data (always available, no dep).
    ///
    /// `rgba` must be exactly `width_px * height_px * 4` bytes.
    /// The byte order is `[R, G, B, A, R, G, B, A, …]`, row-major.
    ///
    /// # Panics
    ///
    /// Panics if `rgba.len() != width_px as usize * height_px as usize * 4`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gilt::image::Image;
    ///
    /// let img = Image::from_rgba(2, 2, vec![
    ///     255, 0,   0,   255,   // red
    ///     0,   255, 0,   255,   // green
    ///     0,   0,   255, 255,   // blue
    ///     255, 255, 255, 255,   // white
    /// ]);
    /// assert_eq!(img.pixel_width(), 2);
    /// assert_eq!(img.pixel_height(), 2);
    /// ```
    pub fn from_rgba(width_px: u32, height_px: u32, rgba: Vec<u8>) -> Self {
        let expected = width_px as usize * height_px as usize * 4;
        assert_eq!(
            rgba.len(),
            expected,
            "rgba length {} does not match {}×{}×4={}",
            rgba.len(),
            width_px,
            height_px,
            expected
        );
        Image {
            rgba,
            width_px,
            height_px,
            target_cols: None,
            target_rows: None,
        }
    }

    /// Decode an image from a file path (requires the `inline-images` feature).
    #[cfg(feature = "inline-images")]
    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ImageError> {
        let dyn_img = ::image::open(path).map_err(ImageError::Decode)?;
        Ok(Self::from_dyn_image(dyn_img))
    }

    /// Decode an image from bytes (requires the `inline-images` feature).
    #[cfg(feature = "inline-images")]
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ImageError> {
        let dyn_img = ::image::load_from_memory(bytes).map_err(ImageError::Decode)?;
        Ok(Self::from_dyn_image(dyn_img))
    }

    #[cfg(feature = "inline-images")]
    fn from_dyn_image(img: ::image::DynamicImage) -> Self {
        let rgba_img = img.into_rgba8();
        let width_px = rgba_img.width();
        let height_px = rgba_img.height();
        let rgba = rgba_img.into_raw();
        Image {
            rgba,
            width_px,
            height_px,
            target_cols: None,
            target_rows: None,
        }
    }

    // -- Accessors -----------------------------------------------------------

    /// The width of the underlying pixel data in pixels.
    pub fn pixel_width(&self) -> u32 {
        self.width_px
    }

    /// The height of the underlying pixel data in pixels.
    pub fn pixel_height(&self) -> u32 {
        self.height_px
    }

    // -- Builders ------------------------------------------------------------

    /// Set the target display width in terminal columns.
    ///
    /// The image will be scaled (with aspect ratio preserved if only this
    /// dimension is set) to occupy `cols` columns in the output.
    pub fn width(mut self, cols: usize) -> Self {
        self.target_cols = Some(cols);
        self
    }

    /// Set the target display height in terminal rows.
    ///
    /// Each row is 2 pixels tall in the halfblock renderer (one `▀` per cell).
    pub fn height(mut self, rows: usize) -> Self {
        self.target_rows = Some(rows);
        self
    }

    // -- Rendering helpers ---------------------------------------------------

    /// Compute the target (cols, rows) cell grid for a given console.
    ///
    /// One terminal cell is 1 pixel wide and 2 pixels tall (halfblock uses the
    /// upper-half-block `▀` to pack two vertical pixels into one character cell).
    /// Given a target number of columns `c`, the matching row count is:
    ///
    /// ```text
    /// rows = round( c * height_px / (width_px * 2) )
    /// ```
    ///
    /// Equivalently, `cols = round( r * width_px * 2 / height_px )`.
    fn resolve_cell_size(&self, console_width: usize) -> (usize, usize) {
        let px_w = self.width_px as f64;
        let px_h = self.height_px as f64;

        match (self.target_cols, self.target_rows) {
            (Some(c), Some(r)) => (c.max(1), r.max(1)),
            (Some(c), None) => {
                let c = c.max(1).min(console_width);
                // rows = c * height_px / (width_px * 2)
                let r = if px_w > 0.0 {
                    ((c as f64 * px_h / (px_w * 2.0)).round() as usize).max(1)
                } else {
                    1
                };
                (c, r)
            }
            (None, Some(r)) => {
                let r = r.max(1);
                // cols = r * width_px * 2 / height_px
                let c = if px_h > 0.0 {
                    ((r as f64 * px_w * 2.0 / px_h).round() as usize)
                        .max(1)
                        .min(console_width)
                } else {
                    1
                };
                (c, r)
            }
            (None, None) => {
                let c = 40_usize.min(console_width).max(1);
                let r = if px_w > 0.0 {
                    ((c as f64 * px_h / (px_w * 2.0)).round() as usize).max(1)
                } else {
                    1
                };
                (c, r)
            }
        }
    }

    /// Nearest-neighbour resize to `(dst_w × dst_h)` pixels.
    ///
    /// Returns RGBA bytes row-major.  No external dep required.
    fn resize_nearest(&self, dst_w: u32, dst_h: u32) -> Vec<u8> {
        let src_w = self.width_px as f64;
        let src_h = self.height_px as f64;
        let mut out = vec![0u8; (dst_w * dst_h * 4) as usize];
        for dy in 0..dst_h {
            for dx in 0..dst_w {
                let sx = ((dx as f64 + 0.5) * src_w / dst_w as f64) as u32;
                let sy = ((dy as f64 + 0.5) * src_h / dst_h as f64) as u32;
                let sx = sx.min(self.width_px - 1);
                let sy = sy.min(self.height_px - 1);
                let src_off = ((sy * self.width_px + sx) * 4) as usize;
                let dst_off = ((dy * dst_w + dx) * 4) as usize;
                out[dst_off..dst_off + 4].copy_from_slice(&self.rgba[src_off..src_off + 4]);
            }
        }
        out
    }

    // -- Protocol renderers --------------------------------------------------

    /// Render using halfblock (`▀`).
    ///
    /// Each character cell shows two vertical pixels:
    /// - foreground color = upper pixel
    /// - background color = lower pixel
    ///
    /// This is pure styled text and renders correctly everywhere and in all
    /// export formats (HTML, SVG).
    fn render_halfblock(&self, console: &Console, opts: &ConsoleOptions) -> Vec<Segment> {
        let (cols, rows) = self.resolve_cell_size(opts.max_width);

        // We need `cols` pixels wide and `rows * 2` pixels tall.
        let dst_w = cols as u32;
        let dst_h = (rows * 2) as u32;
        let pixels = if self.width_px == dst_w && self.height_px == dst_h {
            std::borrow::Cow::Borrowed(self.rgba.as_slice())
        } else {
            std::borrow::Cow::Owned(self.resize_nearest(dst_w, dst_h))
        };

        let mut segments: Vec<Segment> = Vec::with_capacity(rows * (cols + 1));

        for row in 0..rows {
            for col in 0..cols {
                let top_off = ((row * 2) as u32 * dst_w + col as u32) as usize * 4;
                let bot_off = ((row * 2 + 1) as u32 * dst_w + col as u32) as usize * 4;
                let tr = pixels[top_off];
                let tg = pixels[top_off + 1];
                let tb = pixels[top_off + 2];
                let br = pixels[bot_off];
                let bg_g = pixels[bot_off + 1];
                let bb = pixels[bot_off + 2];

                let fg_color = Color::from_rgb(tr, tg, tb);
                let bg_color = Color::from_rgb(br, bg_g, bb);

                let style = Style::null().fg(fg_color).bg(bg_color);

                // Blend alpha against black background for simplicity.
                // Full alpha-compositing omitted to keep dep-free.
                let cell_text = "▀";
                segments.push(Segment::styled(cell_text, style));
            }
            segments.push(Segment::line());
        }

        // Apply console color rendering (truecolor SGR emission)
        // via the console's render_buffer mechanism — but since
        // Renderable returns Segment items and the console's write path
        // calls render_buffer, we just return the segments here.
        // The console will call render_buffer on them and emit correct SGR.
        let _ = console; // console is used for protocol selection, not here
        segments
    }

    /// Render using the Kitty graphics protocol (APC `\x1b_G…\x1b\\`).
    ///
    /// Emits a single `Segment` whose text is the Kitty APC sequence with
    /// base64-encoded raw RGBA pixel data.
    fn render_kitty(&self, opts: &ConsoleOptions) -> Vec<Segment> {
        let (cols, rows) = self.resolve_cell_size(opts.max_width);

        // Transmit at (capped) native resolution and let Kitty scale the image
        // into a `cols × rows` *cell* box via the `c`/`r` keys. Previously the
        // image was downscaled to `cols × rows*2` *pixels* with no c/r, so Kitty
        // displayed it a few pixels wide — a tiny thumbnail regardless of
        // `.width()`. The cap keeps huge photos from producing enormous payloads
        // (~20 px per cell is more than enough resolution for any terminal cell).
        let max_w = (cols as u32).saturating_mul(20).max(1);
        let max_h = (rows as u32).saturating_mul(40).max(1);
        let (tw, th) = if self.width_px <= max_w && self.height_px <= max_h {
            (self.width_px.max(1), self.height_px.max(1))
        } else {
            let scale = (max_w as f64 / self.width_px.max(1) as f64)
                .min(max_h as f64 / self.height_px.max(1) as f64);
            (
                ((self.width_px as f64 * scale).round() as u32).max(1),
                ((self.height_px as f64 * scale).round() as u32).max(1),
            )
        };
        let pixels: Vec<u8> = if self.width_px == tw && self.height_px == th {
            self.rgba.clone()
        } else {
            self.resize_nearest(tw, th)
        };

        // Kitty APC transmit-and-display: f=32 RGBA, s/v = transmitted pixel
        // size, c/r = display size in cells (Kitty scales into that box),
        // a=T (transmit+display).
        let b64 = crate::utils::control::base64_encode(&pixels);
        let apc = format!(
            "\x1b_Gf=32,s={},v={},c={},r={},a=T;{}\x1b\\",
            tw, th, cols, rows, b64
        );

        // Emit as a control segment (empty control vec) so the render pipeline
        // treats it as zero-width and never width-crops or line-splits the APC
        // payload — a plain text segment gets truncated to the console width.
        vec![Segment::new(&apc, None, Some(vec![])), Segment::line()]
    }

    /// Render using the Sixel graphics protocol (DCS `\x1bP…q … \x1b\\`).
    ///
    /// Quantizes to a uniform 6×6×6 (216-colour) palette and emits banded,
    /// run-length-encoded sixel data. This is **dep-free** — no `image` crate
    /// required — so it works in any build. Sixel images are drawn at literal
    /// pixels, so the cell box is approximated at ~10×20 px per cell.
    fn render_sixel(&self, opts: &ConsoleOptions) -> Vec<Segment> {
        let (cols, rows) = self.resolve_cell_size(opts.max_width);
        let w = (cols as u32 * 10).max(1) as usize;
        let h = (rows as u32 * 20).max(1) as usize;
        let pixels = if self.width_px as usize == w && self.height_px as usize == h {
            std::borrow::Cow::Borrowed(self.rgba.as_slice())
        } else {
            std::borrow::Cow::Owned(self.resize_nearest(w as u32, h as u32))
        };

        // Quantize each pixel to a 6×6×6 palette: index = r6*36 + g6*6 + b6.
        let q = |c: u8| ((c as u32 * 5 + 127) / 255) as usize; // 0..=5, rounded
        let mut idx = vec![0u16; w * h];
        let mut used = [false; 216];
        for (i, px) in pixels.chunks_exact(4).enumerate() {
            let p = q(px[0]) * 36 + q(px[1]) * 6 + q(px[2]);
            idx[i] = p as u16;
            used[p] = true;
        }

        let mut out = String::with_capacity(w * h / 4 + 256);
        // DCS introducer: P1=0 aspect, P2=1 (0-bits transparent — required for
        // the multi-pass banded encoding), P3=0; then raster attributes.
        out.push_str("\x1bP0;1;0q");
        out.push_str(&format!("\"1;1;{w};{h}"));
        // Colour registers (RGB, 0..=100 scale) for the palette entries in use.
        let lvl = |l: usize| l * 20; // 0,20,40,60,80,100
        for (p, &u) in used.iter().enumerate() {
            if u {
                out.push_str(&format!("#{};2;{};{};{}", p, lvl(p / 36), lvl((p / 6) % 6), lvl(p % 6)));
            }
        }

        // Emit pixel data band by band (6 rows per band).
        let mut band = 0usize;
        while band < h {
            let band_h = (h - band).min(6);
            // Which palette entries appear in this band?
            let mut present = [false; 216];
            for y in band..band + band_h {
                for x in 0..w {
                    present[idx[y * w + x] as usize] = true;
                }
            }
            let mut first = true;
            for (p, &is_present) in present.iter().enumerate() {
                if !is_present {
                    continue;
                }
                if !first {
                    out.push('$'); // graphics CR: overlay next colour on this band
                }
                first = false;
                out.push_str(&format!("#{p}"));
                // Build the 6-bit sixel value per column for this colour, RLE it.
                let mut run_val = 0u8;
                let mut run_len = 0usize;
                for x in 0..w {
                    let mut bits = 0u8;
                    for (dy, row) in (band..band + band_h).enumerate() {
                        if idx[row * w + x] as usize == p {
                            bits |= 1 << dy;
                        }
                    }
                    if run_len > 0 && bits == run_val {
                        run_len += 1;
                    } else {
                        sixel_emit_run(&mut out, run_val, run_len);
                        run_val = bits;
                        run_len = 1;
                    }
                }
                sixel_emit_run(&mut out, run_val, run_len);
            }
            out.push('-'); // graphics NL: advance to the next 6-row band
            band += 6;
        }

        out.push_str("\x1b\\"); // ST terminator
        vec![Segment::new(&out, None, Some(vec![])), Segment::line()]
    }

    /// Render using the iTerm2 inline-image protocol (OSC 1337).
    ///
    /// Encodes the image as PNG, base64-encodes it, and wraps it in an
    /// `ESC ] 1337 ; File = … : <base64> BEL` sequence sized to `cols × rows`
    /// terminal cells (iTerm2 scales the source image into that cell box,
    /// preserving aspect ratio). Requires the `inline-images` feature for PNG
    /// encoding; without it the iTerm2 path is never taken and `Image` falls back
    /// to halfblock.
    #[cfg(feature = "inline-images")]
    fn render_iterm(&self, opts: &ConsoleOptions) -> Vec<Segment> {
        let (cols, rows) = self.resolve_cell_size(opts.max_width);
        let png = self.to_png();
        let b64 = crate::utils::control::base64_encode(&png);
        let osc = format!(
            "\x1b]1337;File=inline=1;size={};width={};height={};preserveAspectRatio=1:{}\x07",
            png.len(),
            cols,
            rows,
            b64,
        );
        // Control segment (empty control vec): zero-width, never cropped/split,
        // text emitted verbatim — so the full OSC 1337 payload reaches the terminal.
        vec![Segment::new(&osc, None, Some(vec![])), Segment::line()]
    }

    /// Encode the raw RGBA buffer as PNG bytes (for the iTerm2 protocol).
    #[cfg(feature = "inline-images")]
    fn to_png(&self) -> Vec<u8> {
        use std::io::Cursor;
        let img = ::image::RgbaImage::from_raw(self.width_px, self.height_px, self.rgba.clone())
            .expect("rgba buffer length matches width_px × height_px × 4");
        let mut buf = Cursor::new(Vec::new());
        ::image::DynamicImage::ImageRgba8(img)
            .write_to(&mut buf, ::image::ImageFormat::Png)
            .expect("PNG encoding to an in-memory buffer cannot fail");
        buf.into_inner()
    }
}

impl Renderable for Image {
    fn gilt_console(&self, console: &Console, opts: &ConsoleOptions) -> Vec<Segment> {
        let caps = console.capabilities();

        // Protocol selection:
        //   1. If recording → halfblock (export uses styled text)
        //   2. If kitty supported and not recording → Kitty APC
        //   3. If iterm supported (and `inline-images`) → iTerm2 OSC 1337
        //   4. If sixel supported → Sixel DCS
        //   5. Halfblock (always available fallback)
        if console.is_recording() {
            return self.render_halfblock(console, opts);
        }
        if caps.kitty {
            return self.render_kitty(opts);
        }
        // iTerm2 inline images (OSC 1337) — needs PNG encoding (inline-images).
        #[cfg(feature = "inline-images")]
        if caps.iterm {
            return self.render_iterm(opts);
        }
        if caps.sixel {
            return self.render_sixel(opts);
        }
        self.render_halfblock(console, opts)
    }
}

// ---------------------------------------------------------------------------
// Error type (inline-images feature)
// ---------------------------------------------------------------------------

/// Error returned by [`Image::from_path`] and [`Image::from_bytes`].
///
/// Only available when the `inline-images` feature is enabled.
#[cfg(feature = "inline-images")]
#[derive(Debug)]
pub enum ImageError {
    /// The image could not be decoded.
    Decode(::image::ImageError),
}

#[cfg(feature = "inline-images")]
impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageError::Decode(e) => write!(f, "image decode error: {}", e),
        }
    }
}

#[cfg(feature = "inline-images")]
impl std::error::Error for ImageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ImageError::Decode(e) => Some(e),
        }
    }
}
