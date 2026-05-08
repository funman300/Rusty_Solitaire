//! Card-face generator — writes Terminal-aesthetic artwork into both
//! rendering paths the engine consults at runtime:
//!
//! 1. **Asset PNGs at `assets/cards/`** — 52 face + 5 back PNGs loaded
//!    by `card_plugin::load_card_images` as the *fallback* art.
//! 2. **Default-theme SVGs at `solitaire_engine/assets/themes/default/`**
//!    — 52 face + 1 back SVGs that get `include_bytes!()`-embedded into
//!    the binary by `solitaire_engine::assets::sources` and applied to
//!    `CardImageSet` at startup by `theme::plugin::apply_theme_to_card_image_set`.
//!    These *override* the asset PNGs in production; the PNGs only show
//!    if the active theme fails to provide a face.
//!
//! Both paths share the same SVG builders in
//! `solitaire_engine::assets::card_face_svg`, so the artwork stays
//! identical at the source level — running this generator keeps both
//! paths in lockstep.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example card_face_generator --release
//! ```
//!
//! Step 2 of the lockstep migration outlined in
//! `docs/ui-mockups/card-face-migration.md`. Running it overwrites the
//! legacy artwork in-place; the resulting bytes are what step 4 commits
//! alongside the `card_plugin` constant migration.

use solitaire_engine::assets::card_face_svg::{
    back_svg, face_svg, rank_filename, suit_filename, theme_rank_token, theme_suit_token,
    ALL_RANKS, ALL_SUITS, BACK_ACCENTS, TARGET,
};
use solitaire_engine::assets::rasterize_svg;
use std::path::PathBuf;
use tiny_skia::{IntSize, Pixmap};

fn main() {
    let workspace_assets = workspace_assets_dir();
    let cards_dir = workspace_assets.join("cards");
    let faces_dir = cards_dir.join("faces");
    let backs_dir = cards_dir.join("backs");
    std::fs::create_dir_all(&faces_dir).expect("create faces dir");
    std::fs::create_dir_all(&backs_dir).expect("create backs dir");

    // The default theme lives inside the engine crate (so its SVGs can
    // be `include_bytes!()`-embedded relative to the `assets/sources.rs`
    // file path). Workspace-level `assets/cards/` is the fallback path;
    // engine-level `assets/themes/default/` is what production renders.
    let theme_dir = engine_default_theme_dir();
    std::fs::create_dir_all(&theme_dir).expect("create default-theme dir");

    let mut png_written = 0usize;
    let mut svg_written = 0usize;

    for suit in ALL_SUITS {
        for rank in ALL_RANKS {
            let svg = face_svg(rank, suit);

            // Path 1 — fallback PNG.
            let png_path = faces_dir.join(format!(
                "{}{}.png",
                rank_filename(rank),
                suit_filename(suit)
            ));
            rasterize_to_pixmap(&svg)
                .save_png(&png_path)
                .unwrap_or_else(|e| panic!("write {}: {e}", png_path.display()));
            png_written += 1;

            // Path 2 — bundled-default-theme SVG. Same SVG bytes; the
            // theme system rasterises them at runtime.
            let svg_path = theme_dir.join(format!(
                "{}_{}.svg",
                theme_suit_token(suit),
                theme_rank_token(rank),
            ));
            std::fs::write(&svg_path, &svg)
                .unwrap_or_else(|e| panic!("write {}: {e}", svg_path.display()));
            svg_written += 1;
        }
    }

    // Fallback backs — 5 PNGs, one per `Settings::selected_card_back`.
    for (idx, accent) in BACK_ACCENTS.iter().enumerate() {
        let svg = back_svg(accent);
        let png_path = backs_dir.join(format!("back_{idx}.png"));
        rasterize_to_pixmap(&svg)
            .save_png(&png_path)
            .unwrap_or_else(|e| panic!("write {}: {e}", png_path.display()));
        png_written += 1;
    }

    // Theme back — single SVG. Use the canonical Terminal accent
    // (`BACK_ACCENTS[0]` cyan) — the theme system only carries one back
    // per theme, and the canonical Terminal back is the design-system
    // default. The other four accents only live as PNG fallbacks.
    let theme_back_path = theme_dir.join("back.svg");
    std::fs::write(&theme_back_path, back_svg(BACK_ACCENTS[0]))
        .unwrap_or_else(|e| panic!("write {}: {e}", theme_back_path.display()));
    svg_written += 1;

    println!(
        "Wrote {png_written} PNGs ({}×{} RGBA8) to {} and {svg_written} SVGs to {}",
        TARGET.x,
        TARGET.y,
        cards_dir.display(),
        theme_dir.display(),
    );
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

/// Resolves `solitaire_engine/assets/themes/default/` relative to the
/// example crate. Matches `DEFAULT_THEME_MANIFEST_PATH` in
/// `solitaire_engine::assets::sources`.
fn engine_default_theme_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("themes")
        .join("default")
}
