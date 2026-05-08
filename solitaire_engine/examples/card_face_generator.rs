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
//! The SVG builders live in
//! `solitaire_engine::assets::card_face_svg` so the integration
//! test at `tests/card_face_svg_pin.rs` can pin their output
//! against `usvg`/`resvg` rendering drift.

use solitaire_engine::assets::card_face_svg::{
    back_svg, face_svg, rank_filename, suit_filename, ALL_RANKS, ALL_SUITS, BACK_ACCENTS, TARGET,
};
use solitaire_engine::assets::rasterize_svg;
use std::path::PathBuf;
use tiny_skia::{IntSize, Pixmap};

fn main() {
    let cards_dir = workspace_assets_dir().join("cards");
    let faces_dir = cards_dir.join("faces");
    let backs_dir = cards_dir.join("backs");
    std::fs::create_dir_all(&faces_dir).expect("create faces dir");
    std::fs::create_dir_all(&backs_dir).expect("create backs dir");

    let mut written = 0usize;

    for suit in ALL_SUITS {
        for rank in ALL_RANKS {
            let svg = face_svg(rank, suit);
            let pixmap = rasterize_to_pixmap(&svg);
            let path = faces_dir.join(format!(
                "{}{}.png",
                rank_filename(rank),
                suit_filename(suit)
            ));
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
