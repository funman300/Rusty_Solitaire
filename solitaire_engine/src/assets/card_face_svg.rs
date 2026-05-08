//! SVG builders for the Terminal-aesthetic card-face artwork.
//!
//! Used by the `card_face_generator` example to emit the 52 face PNGs +
//! 5 back PNGs into `assets/cards/`, and by the `card_face_svg_pin`
//! integration test to pin the rendered output against `usvg`/`resvg`
//! drift.
//!
//! The numbers below are 2× the `design-system.md` § Game Cards
//! logical sizes — the spec describes a 128 × 192 logical card and
//! this module emits at 256 × 384.
//!
//! See `docs/ui-mockups/card-face-migration.md` for the full
//! migration plan and the rationale behind the output dimensions
//! and palette mapping.
//!
//! # Filled vs outlined glyphs
//!
//! Hearts (♥) and spades (♠) render as filled glyphs. Diamonds (♦)
//! and clubs (♣) render as outlined glyphs (1.5 px stroke at logical
//! scale → 3 px at output). This is the design-system's "always-on"
//! color-blind glyph differentiation and is independent of the
//! red/black colour split.

use bevy::math::UVec2;
use solitaire_core::card::{Rank, Suit};

/// Target rasterisation size in pixels (2:3 aspect, half the default
/// `SvgLoaderSettings` resolution).
pub const TARGET: UVec2 = UVec2::new(256, 384);

const BG_FACE: &str = "#1a1a1a"; // BG_ELEVATED — face background
const SUIT_RED: &str = "#fb9fb1"; // hearts + diamonds
const SUIT_DARK: &str = "#d0d0d0"; // spades + clubs (also TEXT_PRIMARY)

const BACK_BG: &str = "#151515";
const BACK_SCANLINE: &str = "#1a1a1a";
const BACK_BORDER: &str = "#353535";
const BACK_MONOGRAM: &str = "#505050";

/// Five back-theme accent colours. Slot 0 is the canonical "Terminal"
/// back from the design system; the other four cycle through the
/// remaining base16-eighties accents so all 5 slots stay visually
/// distinct without leaving the palette.
pub const BACK_ACCENTS: [&str; 5] = [
    "#6fc2ef", // 0 — cyan (Terminal canonical)
    "#acc267", // 1 — lime
    "#e1a3ee", // 2 — lavender
    "#fb9fb1", // 3 — pink
    "#ddb26f", // 4 — gold
];

/// Every rank in the canonical Ace → King order. Mirrors the order
/// `card_plugin::load_card_images` uses to index `CardImageSet.faces`.
pub const ALL_RANKS: [Rank; 13] = [
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

/// Every suit in `Clubs, Diamonds, Hearts, Spades` order — matches
/// `card_plugin::load_card_images` so the suit index used here lines
/// up with `CardImageSet.faces[suit]`.
pub const ALL_SUITS: [Suit; 4] = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];

/// The rank component of the on-disk filename — `A`, `2`..`10`, `J`,
/// `Q`, `K`. Matches `card_plugin::load_card_images`'s `RANK_STRS`.
pub fn rank_filename(rank: Rank) -> &'static str {
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

/// The suit component of the on-disk filename — `C`, `D`, `H`, `S`.
/// Matches `card_plugin::load_card_images`'s `SUIT_CHARS`.
pub fn suit_filename(suit: Suit) -> &'static str {
    match suit {
        Suit::Clubs => "C",
        Suit::Diamonds => "D",
        Suit::Hearts => "H",
        Suit::Spades => "S",
    }
}

/// Lowercase full-word suit token used by the bundled-default theme's
/// SVG filenames (`<suit>_<rank>.svg` — e.g. `spades_ace.svg`). Mirrors
/// `solitaire_engine::theme::CardKey::manifest_name`.
pub fn theme_suit_token(suit: Suit) -> &'static str {
    match suit {
        Suit::Clubs => "clubs",
        Suit::Diamonds => "diamonds",
        Suit::Hearts => "hearts",
        Suit::Spades => "spades",
    }
}

/// Lowercase full-word rank token used by the bundled-default theme's
/// SVG filenames (`ace`, `2`..`10`, `jack`, `queen`, `king`). Mirrors
/// `solitaire_engine::theme::CardKey::manifest_name`.
pub fn theme_rank_token(rank: Rank) -> &'static str {
    match rank {
        Rank::Ace => "ace",
        Rank::Two => "2",
        Rank::Three => "3",
        Rank::Four => "4",
        Rank::Five => "5",
        Rank::Six => "6",
        Rank::Seven => "7",
        Rank::Eight => "8",
        Rank::Nine => "9",
        Rank::Ten => "10",
        Rank::Jack => "jack",
        Rank::Queen => "queen",
        Rank::King => "king",
    }
}

#[derive(Copy, Clone)]
enum GlyphPaint {
    Filled,
    /// 1.5 px stroke at logical scale → 3 px at 2× output.
    Outlined,
}

fn suit_paint(suit: Suit) -> (&'static str, GlyphPaint) {
    match suit {
        Suit::Hearts => (SUIT_RED, GlyphPaint::Filled),
        Suit::Diamonds => (SUIT_RED, GlyphPaint::Outlined),
        Suit::Spades => (SUIT_DARK, GlyphPaint::Filled),
        Suit::Clubs => (SUIT_DARK, GlyphPaint::Outlined),
    }
}

fn glyph_paint_attrs(colour: &str, paint: GlyphPaint) -> String {
    match paint {
        GlyphPaint::Filled => format!(r#"fill="{colour}""#),
        GlyphPaint::Outlined => {
            format!(r#"fill="none" stroke="{colour}" stroke-width="3""#)
        }
    }
}

/// SVG `path` `d` attribute tracing the suit's silhouette inside a
/// 32 × 32 logical box (origin top-left, +Y down). All four suits are
/// authored as a single closed perimeter so the same path renders
/// correctly whether filled (♥ ♠) or outlined (♦ ♣).
///
/// Path-based rendering replaces the earlier `<text>` approach because
/// the bundled `FiraMono` font doesn't carry the Unicode suit glyphs
/// (U+2660-2666) at the requested size — `usvg` was falling back to a
/// substitute rendering that produced near-invisible "tofu" marks.
/// Paths bypass the font system entirely.
fn suit_path_d(suit: Suit) -> &'static str {
    match suit {
        Suit::Hearts => {
            "M16,28 C 8,22 2,17 2,11 C 2,7 5,4 9,4 \
             C 12,4 14,6 16,9 C 18,6 20,4 23,4 \
             C 27,4 30,7 30,11 C 30,17 24,22 16,28 Z"
        }
        Suit::Diamonds => "M16,2 L 29,16 L 16,30 L 3,16 Z",
        Suit::Spades => {
            "M16,4 C 9,9 2,14 2,21 C 2,25 5,28 9,28 \
             C 13,28 14,26 14,24 L 13,30 L 19,30 L 18,24 \
             C 18,26 19,28 23,28 C 27,28 30,25 30,21 \
             C 30,14 23,9 16,4 Z"
        }
        Suit::Clubs => {
            "M16,4 C 13,4 10,7 10,10 C 10,12 11,13 12,14 \
             C 9,14 4,17 4,21 C 4,24 7,27 10,27 \
             C 12,27 14,26 14,24 L 13,30 L 19,30 L 18,24 \
             C 18,26 20,27 22,27 C 25,27 28,24 28,21 \
             C 28,17 23,14 20,14 C 21,13 22,12 22,10 \
             C 22,7 19,4 16,4 Z"
        }
    }
}

/// Build the SVG markup for a single face card. The output is a
/// self-contained, parsable SVG document.
pub fn face_svg(rank: Rank, suit: Suit) -> String {
    let (colour, paint) = suit_paint(suit);
    let path_d = suit_path_d(suit);
    let rank_text = rank_filename(rank);
    let small_glyph_attrs = glyph_paint_attrs(colour, paint);
    let large_glyph_attrs = glyph_paint_attrs(colour, paint);

    // Numbers come from `design-system.md` § Game Cards, scaled 2×:
    //   border:        1 px  → 2 px  stroke-width
    //   corner radius: 8 px  → 16 px rx/ry
    //   rank font:    18 px  → 36 px
    //   small glyph:  10 px  → 20 px (suit_path_d is authored at 32 →
    //                                 scale 0.625 to land at 20)
    //   large glyph:  32 px  → 64 px (scale 2.0)
    //
    // Inset the border by 1 px so the 2 px stroke renders fully
    // inside the 256 × 384 pixmap rather than getting clipped.
    //
    // Suit glyphs are rendered as inline SVG paths (not `<text>`)
    // because the bundled `FiraMono` font doesn't carry usable
    // U+2660-2666 glyphs at the requested size. See `suit_path_d`
    // for the rationale.
    //
    // Both glyphs render in the same upright orientation. The
    // traditional playing-card convention rotates the bottom-right
    // indicator 180° so the card reads correctly when flipped, but
    // most digital decks have abandoned that — single-orientation
    // play doesn't benefit from the inverted-corner readback. See
    // `design-system.md` § Game Cards for the spec deviation.
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="256" height="384" viewBox="0 0 256 384">
  <rect x="1" y="1" width="254" height="382" rx="16" ry="16"
        fill="{BG_FACE}" stroke="{colour}" stroke-width="2"/>

  <!-- Top-left rank in JetBrains-Mono-styled FiraMono (rank digits
       and letters render correctly in FiraMono; only the suit glyphs
       needed to escape to paths). -->
  <text x="14" y="44" font-family="Fira Mono" font-size="36" font-weight="700"
        fill="{colour}">{rank_text}</text>

  <!-- Top-left small suit glyph at (14, 50), 20 × 20.
       `suit_path_d` is authored in a 32-unit box, so scale 0.625
       lands the visible glyph at 20 px. -->
  <g transform="translate(14 50) scale(0.625)">
    <path d="{path_d}" {small_glyph_attrs}/>
  </g>

  <!-- Bottom-right large suit glyph at (178, 286), 64 × 64.
       Visible bottom-right at (242, 350), visible top-left at
       (178, 286). Same upright orientation as the top-left small
       glyph — no 180° rotation applied. -->
  <g transform="translate(178 286) scale(2)">
    <path d="{path_d}" {large_glyph_attrs}/>
  </g>
</svg>"##
    )
}

/// Build the SVG markup for a card back with the canonical Terminal
/// scanline pattern. `accent` swaps only the top-left badge.
pub fn back_svg(accent: &str) -> String {
    // Scanline tile: 1 px line + 1 px gap at logical scale → 2 px +
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
