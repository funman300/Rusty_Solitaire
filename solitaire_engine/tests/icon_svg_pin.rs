//! Pinning test for the application-icon SVG builder.
//!
//! Hashes the raw RGBA8 pixel bytes produced by rasterising
//! `icon_svg()` at every size in `ICON_SIZES`, compares each hash
//! to an embedded constant, and fails on any drift. Catches
//! `usvg`/`resvg`/`tiny_skia` rendering changes and any
//! intentional builder edit that wasn't paired with a hash
//! refresh.
//!
//! When the icon SVG changes intentionally (or a dependency
//! upgrade legitimately changes rendering), update `EXPECTED` by
//! emptying it (`&[]`) and re-running this test once — the test
//! will panic with the new hashes formatted as Rust source ready
//! to paste back in. Same bootstrap pattern as
//! `card_face_svg_pin.rs`.

use bevy::math::UVec2;
use solitaire_engine::assets::icon_svg::{icon_svg, ICON_SIZES};
use solitaire_engine::assets::rasterize_svg;

const EXPECTED: &[(u32, u64)] = &[
    (16, 0x07e641beea430d66),
    (24, 0x24e66767f4756a60),
    (32, 0xf22a3104623a3873),
    (48, 0x2d7f978cf7b12763),
    (64, 0x1b377e3e30202eba),
    (128, 0xafdc80f901b45518),
    (256, 0x82b5b46f73c5921d),
    (512, 0xe14c018e1e285209),
    (1024, 0xfcd0a6a3beb68bdb),
];

#[test]
fn rasterised_icon_bytes_match_pinned_hashes() {
    let actual = compute_actual_hashes();

    if EXPECTED.is_empty() {
        panic_with_hashes_to_paste(&actual);
    }

    assert_eq!(
        actual.len(),
        EXPECTED.len(),
        "icon-size count drifted (actual {} vs expected {})",
        actual.len(),
        EXPECTED.len(),
    );

    let mut mismatches: Vec<String> = Vec::new();
    for ((actual_size, actual_hash), (expected_size, expected_hash)) in
        actual.iter().zip(EXPECTED.iter())
    {
        assert_eq!(
            actual_size, expected_size,
            "icon-size order drifted",
        );
        if actual_hash != expected_hash {
            mismatches.push(format!(
                "  icon_{actual_size}: actual 0x{actual_hash:016x}  expected 0x{expected_hash:016x}",
            ));
        }
    }

    if !mismatches.is_empty() {
        let mut msg = String::from(
            "rasterised icon bytes drifted from EXPECTED — usvg/resvg/tiny_skia/font upgrade?\n",
        );
        for m in &mismatches {
            msg.push_str(m);
            msg.push('\n');
        }
        msg.push_str(
            "\nIf this drift is intentional, replace EXPECTED with `&[]` and re-run\nthis test to print fresh hashes.\n",
        );
        panic!("{msg}");
    }
}

fn compute_actual_hashes() -> Vec<(u32, u64)> {
    let svg = icon_svg();
    ICON_SIZES
        .iter()
        .map(|&size| (size, hash_rasterised(&svg, size)))
        .collect()
}

fn hash_rasterised(svg: &str, size: u32) -> u64 {
    let target = UVec2::new(size, size);
    let image = rasterize_svg(svg.as_bytes(), target).expect("rasterise icon SVG");
    let bytes = image.data.expect("rasterised image carries RGBA pixel data");
    fnv1a(&bytes)
}

/// FNV-1a 64-bit, inline. Same shape as `card_face_svg_pin.rs` —
/// no cryptographic strength needed, just stable byte fingerprints.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn panic_with_hashes_to_paste(actual: &[(u32, u64)]) -> ! {
    let mut out = String::from(
        "\nEXPECTED is empty — paste the following into the const literal:\n\nconst EXPECTED: &[(u32, u64)] = &[\n",
    );
    for (size, hash) in actual {
        out.push_str(&format!("    ({size}, 0x{hash:016x}),\n"));
    }
    out.push_str("];\n");
    panic!("{out}");
}
