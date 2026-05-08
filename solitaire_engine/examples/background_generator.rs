//! Table-background generator — writes 5 Terminal-aesthetic solid-colour
//! PNGs into `assets/backgrounds/`.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example background_generator --release
//! ```
//!
//! The play-surface fill is `Settings::selected_background`-indexed
//! into `assets/backgrounds/bg_{0..4}.png` by
//! `table_plugin::load_background_images`. Per `design-system.md` the
//! Terminal play surface is *flat* — no felt texture, no gradient —
//! so all five slots are simple solid colours from the base16-eighties
//! palette, each visually distinct enough for the picker but all on
//! brand.
//!
//! Output is small (a 120 × 168 PNG with one solid colour compresses
//! to ≈100 bytes) and stretched to `window_size * 2.0` at runtime by
//! `table_plugin::spawn_background`, so the source resolution is
//! immaterial — keeping it 120 × 168 preserves the legacy tile size.

use std::path::PathBuf;
use tiny_skia::{Color, IntSize, Pixmap};

const TILE_W: u32 = 120;
const TILE_H: u32 = 168;

/// Five Terminal-palette play-surface variants. Slot 0 is the canonical
/// design-system `#151515`; the others stay in the same near-black
/// family so the picker offers choice without ever leaving the brand.
const BACKGROUNDS: [(u8, u8, u8); 5] = [
    (0x15, 0x15, 0x15), // 0 — Terminal canonical (#151515 BG_PRIMARY)
    (0x0a, 0x0a, 0x0a), // 1 — deeper near-black (BG_DEEPEST)
    (0x1a, 0x1a, 0x1a), // 2 — BG_ELEVATED (matches card face)
    (0x12, 0x18, 0x20), // 3 — slight cool tint
    (0x20, 0x18, 0x12), // 4 — slight warm tint
];

fn main() {
    let dir = workspace_assets_dir().join("backgrounds");
    std::fs::create_dir_all(&dir).expect("create backgrounds dir");

    let size = IntSize::from_wh(TILE_W, TILE_H).expect("non-zero tile size");

    for (idx, (r, g, b)) in BACKGROUNDS.iter().enumerate() {
        let mut pixmap = Pixmap::new(size.width(), size.height()).expect("alloc pixmap");
        pixmap.fill(Color::from_rgba8(*r, *g, *b, 0xff));

        let path = dir.join(format!("bg_{idx}.png"));
        pixmap
            .save_png(&path)
            .unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    }

    println!(
        "Wrote 5 PNGs ({}×{} solid Terminal colours) to {}",
        TILE_W,
        TILE_H,
        dir.display(),
    );
}

/// Resolves the workspace-root `assets/` directory relative to the
/// running example crate. `CARGO_MANIFEST_DIR` is the engine crate;
/// its parent is the workspace root.
fn workspace_assets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("solitaire_engine crate has a workspace-root parent")
        .join("assets")
}
