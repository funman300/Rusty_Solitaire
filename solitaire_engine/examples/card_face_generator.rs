//! Card-face generator — writes 52 Terminal-aesthetic face PNGs +
//! 5 back PNGs into `assets/cards/`.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example card_face_generator --release
//! ```
//!
//! This is **step 2** of the lockstep migration outlined in
//! `docs/ui-mockups/card-face-migration.md`. Running it overwrites
//! the legacy PNG artwork in-place; the resulting bytes are what
//! step 4 commits alongside the `card_plugin` constant migration.
//!
//! Output paths (matching the filenames `card_plugin::load_card_images`
//! already loads):
//!
//! - Faces: `assets/cards/faces/<RANK><SUIT>.png`
//!   where `RANK` ∈ `{A, 2..10, J, Q, K}` and `SUIT` ∈ `{C, D, H, S}`.
//! - Backs: `assets/cards/backs/back_{0..4}.png`.
//!
//! All output is 256 × 384 RGBA8 sRGB — half the default
//! `SvgLoaderSettings` resolution, sufficient for ~250 px-wide
//! desktop card sprites and ≈⅓ the disk weight of the legacy 512 ×
//! 768 art (per the migration plan's "Output format" rationale).

use bevy::math::UVec2;
use solitaire_core::card::{Rank, Suit};
use solitaire_engine::assets::rasterize_svg;
use std::path::PathBuf;
use tiny_skia::{IntSize, Pixmap};

// 2× the design-system logical sizes — the spec describes a 128 ×
// 192 logical card, this generator targets 256 × 384.
const TARGET: UVec2 = UVec2::new(256, 384);

// Palette literals — base16-eighties, mirroring `design-system.md`.
const BG_FACE: &str = "#1a1a1a"; // BG_ELEVATED
const SUIT_RED: &str = "#fb9fb1"; // hearts + diamonds
const SUIT_DARK: &str = "#d0d0d0"; // spades + clubs (also TEXT_PRIMARY)

// Card-back palette.
const BACK_BG: &str = "#151515";
const BACK_SCANLINE: &str = "#1a1a1a";
const BACK_BORDER: &str = "#353535";
const BACK_MONOGRAM: &str = "#505050";

// Five back-theme accent colours. Slot 0 is the canonical "Terminal"
// back from the design system; the other four cycle through the
// remaining base16-eighties accents so all 5 slots stay visually
// distinct without leaving the palette.
const BACK_ACCENTS: [&str; 5] = [
    "#6fc2ef", // 0 — cyan (Terminal canonical)
    "#acc267", // 1 — lime
    "#e1a3ee", // 2 — lavender
    "#fb9fb1", // 3 — pink
    "#ddb26f", // 4 — gold
];

fn main() {
    let cards_dir = workspace_assets_dir().join("cards");
    let faces_dir = cards_dir.join("faces");
    let backs_dir = cards_dir.join("backs");
    std::fs::create_dir_all(&faces_dir).expect("create faces dir");
    std::fs::create_dir_all(&backs_dir).expect("create backs dir");

    let mut written = 0usize;

    for suit in [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
        for rank in ALL_RANKS {
            let svg = face_svg(rank, suit);
            let pixmap = rasterize_to_pixmap(&svg);
            let path = faces_dir.join(format!("{}{}.png", rank_str(rank), suit_char(suit)));
            pixmap
                .save_png(&path)
                .unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
            written += 1;
        }
    }

    for (idx, accent) in BACK_ACCENTS.iter().enumerate() {
        let svg = back_svg(accent);
        let pixmap = rasterize_to_pixmap(&svg);
        let path = backs_dir.join(format!("back_{idx}.png"));
        pixmap
            .save_png(&path)
            .unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
        written += 1;
    }

    println!(
        "Wrote {written} PNGs ({}×{} RGBA8) to {}",
        TARGET.x,
        TARGET.y,
        cards_dir.display(),
    );
}

const ALL_RANKS: [Rank; 13] = [
    Rank::Ace,
    Rank::Two,
    Rank::Three,
    Rank::Four,
    Rank::Five,
    Rank::Six,
    Rank::Seven,
    Rank::Eight,
    Rank::Nine,
    Rank::Ten,
    Rank::Jack,
    Rank::Queen,
    Rank::King,
];

fn rank_str(rank: Rank) -> &'static str {
    match rank {
        Rank::Ace => "A",
        Rank::Two => "2",
        Rank::Three => "3",
        Rank::Four => "4",
        Rank::Five => "5",
        Rank::Six => "6",
        Rank::Seven => "7",
        Rank::Eight => "8",
        Rank::Nine => "9",
        Rank::Ten => "10",
        Rank::Jack => "J",
        Rank::Queen => "Q",
        Rank::King => "K",
    }
}

fn suit_char(suit: Suit) -> &'static str {
    match suit {
        Suit::Clubs => "C",
        Suit::Diamonds => "D",
        Suit::Hearts => "H",
        Suit::Spades => "S",
    }
}

/// Returns the suit colour and the `<text>` paint attributes for the
/// glyph (filled vs outlined). Hearts + spades are filled; diamonds +
/// clubs are outlined — the "always-on" color-blind glyph
/// differentiation from the design system.
fn suit_paint(suit: Suit) -> (&'static str, GlyphPaint) {
    match suit {
        Suit::Hearts => (SUIT_RED, GlyphPaint::Filled),
        Suit::Diamonds => (SUIT_RED, GlyphPaint::Outlined),
        Suit::Spades => (SUIT_DARK, GlyphPaint::Filled),
        Suit::Clubs => (SUIT_DARK, GlyphPaint::Outlined),
    }
}

#[derive(Copy, Clone)]
enum GlyphPaint {
    Filled,
    /// 1.5 px stroke at logical scale → 3 px at 2× output.
    Outlined,
}

fn glyph_paint_attrs(colour: &str, paint: GlyphPaint) -> String {
    match paint {
        GlyphPaint::Filled => format!(r#"fill="{colour}""#),
        GlyphPaint::Outlined => {
            format!(r#"fill="none" stroke="{colour}" stroke-width="3""#)
        }
    }
}

fn suit_glyph(suit: Suit) -> &'static str {
    match suit {
        Suit::Clubs => "&#x2663;",
        Suit::Diamonds => "&#x2666;",
        Suit::Hearts => "&#x2665;",
        Suit::Spades => "&#x2660;",
    }
}

/// Builds the face-card SVG. Sizes are doubled from the design-system
/// logical pixels (the spec describes a 128 × 192 card; we emit at
/// 256 × 384).
fn face_svg(rank: Rank, suit: Suit) -> String {
    let (colour, paint) = suit_paint(suit);
    let glyph = suit_glyph(suit);
    let rank_text = rank_str(rank);
    let small_glyph_attrs = glyph_paint_attrs(colour, paint);
    let large_glyph_attrs = glyph_paint_attrs(colour, paint);

    // Numbers come from `design-system.md` § Game Cards, scaled 2×:
    //   border:        1 px  → 2 px  stroke-width
    //   corner radius: 8 px  → 16 px rx/ry
    //   rank font:    18 px  → 36 px
    //   small glyph:  10 px  → 20 px
    //   large glyph:  32 px  → 64 px
    //
    // Inset the border by 1 px (`x="1" y="1" width="254" height="382"`)
    // so the 2 px stroke renders fully inside the 256 × 384 pixmap
    // rather than getting clipped at the edge.
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="256" height="384" viewBox="0 0 256 384">
  <rect x="1" y="1" width="254" height="382" rx="16" ry="16"
        fill="{BG_FACE}" stroke="{colour}" stroke-width="2"/>

  <!-- Top-left rank + small suit glyph. -->
  <text x="14" y="44" font-family="Fira Mono" font-size="36" font-weight="700"
        fill="{colour}">{rank_text}</text>
  <text x="14" y="68" font-family="Fira Mono" font-size="20"
        {small_glyph_attrs}>{glyph}</text>

  <!-- Bottom-right large suit glyph, rotated 180° so it reads
       upside-down (the convention for inverted-corner indicators). -->
  <text x="242" y="350" font-family="Fira Mono" font-size="64"
        text-anchor="end" {large_glyph_attrs}
        transform="rotate(180 242 332)">{glyph}</text>
</svg>"##
    )
}

/// Builds a card-back SVG with the canonical Terminal scanline
/// pattern. `accent` swaps only the top-left badge — every other
/// element stays palette-locked so all 5 backs read as members of
/// the same family.
fn back_svg(accent: &str) -> String {
    // Pattern tile: 1 px line + 1 px gap at logical scale → 2 px +
    // 2 px at 2× output. `patternUnits="userSpaceOnUse"` so the tile
    // size is in viewBox pixels rather than fractions of the box.
    //
    // Badge: 12 × 16 px logical → 24 × 32 px output, 12 px from corner.
    // Monogram: "▌RS" in 12 px logical → 24 px output, 12 px inset.
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="256" height="384" viewBox="0 0 256 384">
  <defs>
    <pattern id="scanlines" x="0" y="0" width="2" height="4" patternUnits="userSpaceOnUse">
      <rect x="0" y="0" width="2" height="2" fill="{BACK_SCANLINE}"/>
    </pattern>
  </defs>

  <!-- Background fill, then scanlines on top (the scanlines stay
       darker than BACK_BG so the "off" rows show through). -->
  <rect x="1" y="1" width="254" height="382" rx="16" ry="16"
        fill="{BACK_BG}" stroke="{BACK_BORDER}" stroke-width="2"/>
  <rect x="1" y="1" width="254" height="382" rx="16" ry="16"
        fill="url(#scanlines)"/>

  <!-- Top-left accent badge (the only theme-varying element). -->
  <rect x="12" y="12" width="24" height="32" fill="{accent}"/>

  <!-- Bottom-right "▌RS" monogram in JetBrains-Mono-styled FiraMono. -->
  <text x="244" y="368" font-family="Fira Mono" font-size="24"
        fill="{BACK_MONOGRAM}" text-anchor="end">&#x258C;RS</text>
</svg>"##
    )
}

fn rasterize_to_pixmap(svg: &str) -> Pixmap {
    let image = rasterize_svg(svg.as_bytes(), TARGET).expect("rasterise card SVG");
    let bytes = image.data.expect("rasterised image carries pixel data");
    debug_assert_eq!(
        bytes.len(),
        (TARGET.x * TARGET.y * 4) as usize,
        "rasterised buffer must match width × height × 4 RGBA bytes",
    );
    let size = IntSize::from_wh(TARGET.x, TARGET.y).expect("non-zero target size");
    Pixmap::from_vec(bytes, size).expect("RGBA buffer forms a valid Pixmap")
}

/// Resolves the workspace-root `assets/` directory relative to the
/// running example crate (`solitaire_engine/`). `CARGO_MANIFEST_DIR`
/// is the engine crate; its parent is the workspace root.
fn workspace_assets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("solitaire_engine crate has a workspace-root parent")
        .join("assets")
}
