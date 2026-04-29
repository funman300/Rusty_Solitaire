//! Generates placeholder PNG assets for card faces, card backs, and table
//! backgrounds. All images are 16×16 pixels — Bevy's Sprite scales them via
//! `custom_size`, so small files keep the repository lightweight.
//!
//! Run with:
//! ```
//! cargo run -p solitaire_assetgen --bin gen_art
//! ```

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

// ---------------------------------------------------------------------------
// PNG helper
// ---------------------------------------------------------------------------

/// Write a 16×16 RGBA image to `path`.  `pixels` is a flat `[R,G,B,A, ...]`
/// byte array with exactly 16 * 16 * 4 = 1024 bytes.
fn save_png(path: &Path, pixels: &[u8; 1024]) {
    let file = File::create(path)
        .unwrap_or_else(|e| panic!("cannot create {}: {e}", path.display()));
    let mut w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(&mut w, 16, 16);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .unwrap_or_else(|e| panic!("png header error for {}: {e}", path.display()));
    writer
        .write_image_data(pixels)
        .unwrap_or_else(|e| panic!("png data error for {}: {e}", path.display()));
}

/// Build a flat 16×16 RGBA pixel array using a per-pixel closure.
fn make_image<F: Fn(u32, u32) -> [u8; 4]>(f: F) -> [u8; 1024] {
    let mut pixels = [0u8; 1024];
    for y in 0u32..16 {
        for x in 0u32..16 {
            let rgba = f(x, y);
            let i = ((y * 16 + x) * 4) as usize;
            pixels[i..i + 4].copy_from_slice(&rgba);
        }
    }
    pixels
}

// ---------------------------------------------------------------------------
// Card face
// ---------------------------------------------------------------------------

/// Cream/ivory solid fill — represents a blank card face.
fn make_face() -> [u8; 1024] {
    make_image(|_, _| [0xF8, 0xF8, 0xF0, 0xFF])
}

// ---------------------------------------------------------------------------
// Card backs  (match the colours used in card_plugin.rs `card_back_colour()`)
// ---------------------------------------------------------------------------

/// back_0 — blue base with semi-transparent white horizontal stripes every 4 px.
fn make_back_0() -> [u8; 1024] {
    make_image(|_, y| {
        if y % 4 < 2 {
            [0xFF, 0xFF, 0xFF, 40]
        } else {
            [0x26, 0x4D, 0x8C, 0xFF]
        }
    })
}

/// back_1 — red base with semi-transparent white diagonal stripes.
fn make_back_1() -> [u8; 1024] {
    make_image(|x, y| {
        if (x + y) % 4 < 2 {
            [0xFF, 0xFF, 0xFF, 40]
        } else {
            [0x8C, 0x1A, 0x1A, 0xFF]
        }
    })
}

/// back_2 — green base with white dots at every 4-px grid intersection.
fn make_back_2() -> [u8; 1024] {
    make_image(|x, y| {
        if x % 4 == 0 && y % 4 == 0 {
            [0xFF, 0xFF, 0xFF, 0xFF]
        } else {
            [0x0D, 0x66, 0x1A, 0xFF]
        }
    })
}

/// back_3 — purple base with a white diamond centred at (8, 8).
fn make_back_3() -> [u8; 1024] {
    make_image(|x, y| {
        let dx = (x as i32 - 8).unsigned_abs();
        let dy = (y as i32 - 8).unsigned_abs();
        if dx + dy <= 4 {
            [0xFF, 0xFF, 0xFF, 0xFF]
        } else {
            [0x59, 0x14, 0x85, 0xFF]
        }
    })
}

/// back_4 — teal base with a 1-px white border.
fn make_back_4() -> [u8; 1024] {
    make_image(|x, y| {
        if x == 0 || x == 15 || y == 0 || y == 15 {
            [0xFF, 0xFF, 0xFF, 0xFF]
        } else {
            [0x0D, 0x66, 0x6B, 0xFF]
        }
    })
}

// ---------------------------------------------------------------------------
// Backgrounds
// ---------------------------------------------------------------------------

/// bg_0 — dark green felt with very faint lighter grid lines every 8 px.
fn make_bg_0() -> [u8; 1024] {
    make_image(|x, y| {
        if x % 8 == 0 || y % 8 == 0 {
            [0xFF, 0xFF, 0xFF, 30]
        } else {
            [0x1A, 0x4D, 0x1A, 0xFF]
        }
    })
}

/// bg_1 — dark wood brown with faint horizontal grain lines every 2 px.
fn make_bg_1() -> [u8; 1024] {
    make_image(|_, y| {
        if y % 2 == 0 {
            [0xFF, 0xFF, 0xFF, 20]
        } else {
            [0x40, 0x2D, 0x1A, 0xFF]
        }
    })
}

/// bg_2 — navy with faint star/dot pattern (offset rows) every 8 px.
fn make_bg_2() -> [u8; 1024] {
    make_image(|x, y| {
        let row_offset: u32 = if (y / 4) % 2 == 0 { 0 } else { 4 };
        if (x + row_offset) % 8 == 0 && y % 8 == 0 {
            [0xFF, 0xFF, 0xFF, 0xFF]
        } else {
            [0x0D, 0x14, 0x38, 0xFF]
        }
    })
}

/// bg_3 — burgundy with a faint diamond-grid pattern.
fn make_bg_3() -> [u8; 1024] {
    make_image(|x, y| {
        if (x + y) % 8 == 0 {
            [0xFF, 0xFF, 0xFF, 30]
        } else {
            [0x4D, 0x0D, 0x14, 0xFF]
        }
    })
}

/// bg_4 — charcoal with faint pixel noise (alternating pixels every 3 columns).
fn make_bg_4() -> [u8; 1024] {
    make_image(|x, y| {
        if (x + y) % 2 == 0 && x % 3 == 0 {
            [0xFF, 0xFF, 0xFF, 20]
        } else {
            [0x1F, 0x1F, 0x24, 0xFF]
        }
    })
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn workspace_root() -> std::path::PathBuf {
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir.parent().unwrap().to_path_buf()
}

fn main() {
    let root = workspace_root();

    // Ensure output directories exist.
    std::fs::create_dir_all(root.join("assets/cards/faces")).unwrap();
    std::fs::create_dir_all(root.join("assets/cards/backs")).unwrap();
    std::fs::create_dir_all(root.join("assets/backgrounds")).unwrap();

    // Card face.
    let path = root.join("assets/cards/faces/face.png");
    save_png(&path, &make_face());
    println!("wrote {}", path.display());

    // Card backs.
    let backs = [
        make_back_0(),
        make_back_1(),
        make_back_2(),
        make_back_3(),
        make_back_4(),
    ];
    for (i, pixels) in backs.iter().enumerate() {
        let path = root.join(format!("assets/cards/backs/back_{i}.png"));
        save_png(&path, pixels);
        println!("wrote {}", path.display());
    }

    // Backgrounds.
    let bgs = [
        make_bg_0(),
        make_bg_1(),
        make_bg_2(),
        make_bg_3(),
        make_bg_4(),
    ];
    for (i, pixels) in bgs.iter().enumerate() {
        let path = root.join(format!("assets/backgrounds/bg_{i}.png"));
        save_png(&path, pixels);
        println!("wrote {}", path.display());
    }

    println!("gen_art: all placeholder PNG assets generated successfully.");
}
