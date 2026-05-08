//! Application-icon generator — rasterises the project's icon SVG
//! into `assets/icon/icon_<size>.png` at every size in
//! `card_face_svg::ICON_SIZES` (16, 24, 32, 48, 64, 128, 256, 512,
//! 1024). Sufficient to cover Linux hicolor, Windows `.ico`, and
//! macOS `.icns` packaging targets — and the runtime `Window::icon`
//! wiring picks the 256 px slot.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example icon_generator --release
//! ```
//!
//! Same shape as `card_face_generator`: SVG builder lives in
//! `solitaire_engine::assets::icon_svg` so the `icon_svg_pin`
//! integration test can call it. Rasterisation runs through
//! `assets::rasterize_svg` (the `usvg` + `resvg` + `tiny_skia`
//! pipeline already used by every other generated asset).

use bevy::math::UVec2;
use solitaire_engine::assets::icon_svg::{icon_svg, ICON_SIZES};
use solitaire_engine::assets::rasterize_svg;
use std::path::PathBuf;
use tiny_skia::{IntSize, Pixmap};

fn main() {
    let icon_dir = workspace_assets_dir().join("icon");
    std::fs::create_dir_all(&icon_dir).expect("create icon dir");

    let svg = icon_svg();

    for &size in ICON_SIZES {
        let target = UVec2::new(size, size);
        let pixmap = rasterize_to_pixmap(&svg, target);
        let path = icon_dir.join(format!("icon_{size}.png"));
        pixmap
            .save_png(&path)
            .unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    }

    println!(
        "Wrote {} PNGs ({}–{} px) to {}",
        ICON_SIZES.len(),
        ICON_SIZES.iter().min().copied().unwrap_or(0),
        ICON_SIZES.iter().max().copied().unwrap_or(0),
        icon_dir.display(),
    );
}

fn rasterize_to_pixmap(svg: &str, target: UVec2) -> Pixmap {
    let image = rasterize_svg(svg.as_bytes(), target).expect("rasterise icon SVG");
    let bytes = image.data.expect("rasterised image carries pixel data");
    debug_assert_eq!(
        bytes.len(),
        (target.x * target.y * 4) as usize,
        "rasterised buffer must match width × height × 4 RGBA bytes",
    );
    let size = IntSize::from_wh(target.x, target.y).expect("non-zero target size");
    Pixmap::from_vec(bytes, size).expect("RGBA buffer forms a valid Pixmap")
}

fn workspace_assets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("solitaire_engine crate has a workspace-root parent")
        .join("assets")
}
