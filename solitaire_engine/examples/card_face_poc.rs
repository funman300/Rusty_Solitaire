//! Card-face migration PoC — generates a Terminal-aesthetic Ace of
//! Spades to `/tmp/ace_spades_terminal.png`.
//!
//! Tracks `docs/ui-mockups/card-face-migration.md`'s recommended
//! generation pipeline (programmatic SVG via the existing
//! `usvg` + `resvg` + `tiny_skia` stack already used by
//! `solitaire_engine::assets::svg_loader`). One card is enough to
//! prove the pipeline; the next step in the migration loops the
//! same code over all 52 faces + 5 backs.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example card_face_poc --release
//! ```
//!
//! The example writes the PNG to `/tmp` rather than `assets/` so a
//! human can eyeball it before committing to a deck-wide rollout.
//! When the migration lands, the generator graduates into a real
//! binary that writes into `assets/cards/`.
//!
//! What this PoC proves:
//!
//! 1. The SVG-to-PNG pipeline produces valid output for a card-shaped
//!    SVG (the existing rasteriser is used for full theme atlases
//!    today; this confirms it works at the per-card grain).
//! 2. The Terminal palette (`#1a1a1a` background, `#d0d0d0` foreground
//!    for spade glyphs) renders correctly.
//! 3. The bundled FiraMono in `svg_loader::shared_fontdb` resolves
//!    `font-family="Fira Mono"` and renders both the rank "A" and
//!    the Unicode suit glyph `♠` (U+2660).
//! 4. The 8 px corner radius (per `design-system.md`) renders as a
//!    rounded rect at the 256×384 output size.
//! 5. SVG `transform="rotate(180 …)"` produces the bottom-right
//!    flipped suit glyph called for in the spec.

use bevy::math::UVec2;
use solitaire_engine::assets::rasterize_svg;
use tiny_skia::{IntSize, Pixmap};

fn main() {
    let svg = ace_of_spades_svg();

    // 256×384 = 2:3 aspect at half the default svg_loader resolution.
    // See migration plan § "Output format" for the rationale.
    let target = UVec2::new(256, 384);
    let image = rasterize_svg(svg.as_bytes(), target)
        .expect("rasterising the PoC SVG should succeed");

    let bytes = image
        .data
        .expect("rasterized image must carry RGBA pixel data");
    assert_eq!(
        bytes.len(),
        (target.x * target.y * 4) as usize,
        "raw byte count must match width × height × 4 RGBA bytes",
    );

    // Re-wrap the raw bytes in a `tiny_skia::Pixmap` so we can write
    // a PNG via `save_png`. `rasterize_svg` already produced these
    // bytes from a Pixmap inside `svg_loader`; this round-trip is
    // the cost of going through Bevy's `Image` shape.
    let size = IntSize::from_wh(target.x, target.y).expect("target size is non-zero");
    let pixmap = Pixmap::from_vec(bytes, size)
        .expect("RGBA byte buffer should form a valid Pixmap");

    let out = "/tmp/ace_spades_terminal.png";
    pixmap.save_png(out).expect("writing the PNG should succeed");

    println!(
        "Wrote {} ({}×{} RGBA8, {} bytes on disk)",
        out,
        target.x,
        target.y,
        std::fs::metadata(out).map(|m| m.len()).unwrap_or(0),
    );
}

/// Builds the Ace-of-Spades SVG matching `design-system.md`
/// § Game Cards. The numbers below are the spec's logical sizes
/// scaled by 2× for the 256×384 output target (the spec describes
/// pixel sizes for a 128×192 logical card; doubling preserves the
/// visual proportions).
fn ace_of_spades_svg() -> String {
    // Palette literals come straight from the design system. Quoted
    // verbatim rather than constructed via the engine's `ui_theme`
    // because the SVG renderer expects CSS colour strings, and a
    // round-trip through `Color::srgb` → CSS would be both lossier
    // and noisier than the inline string.
    let bg = "#1a1a1a"; // BG_ELEVATED — card face
    let suit = "#d0d0d0"; // TEXT_PRIMARY — spades use the foreground gray

    // Corner radius: 8 px logical → 16 px at 2× scale.
    // Border: 1 px solid in suit colour → 2 px at 2× scale.
    // Rank: JetBrains Mono Bold 18 px → 36 px. The bundled fontdb
    //   ships FiraMono only; usvg substitutes when JetBrains Mono
    //   isn't available, so we explicitly request `Fira Mono` to
    //   skip the substitution lookup.
    // Small suit glyph: 10 px → 20 px.
    // Large suit glyph: 32 px → 64 px, rotated 180° about its own
    //   centre so the bottom-right corner reads as the top-left
    //   when you flip the card.
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="256" height="384" viewBox="0 0 256 384">
  <rect x="1" y="1" width="254" height="382" rx="16" ry="16"
        fill="{bg}" stroke="{suit}" stroke-width="2"/>

  <!-- Top-left rank + small suit glyph. -->
  <text x="14" y="44" font-family="Fira Mono" font-size="36" font-weight="700"
        fill="{suit}">A</text>
  <text x="14" y="68" font-family="Fira Mono" font-size="20"
        fill="{suit}">&#x2660;</text>

  <!-- Bottom-right large suit glyph, rotated 180° about its own
       baseline anchor so the glyph reads upside-down. -->
  <text x="242" y="350" font-family="Fira Mono" font-size="64"
        fill="{suit}" text-anchor="end"
        transform="rotate(180 242 332)">&#x2660;</text>
</svg>"##
    )
}
