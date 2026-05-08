//! Pinning test for the Terminal card-face SVG builders.
//!
//! Hashes the raw RGBA8 pixel bytes produced by rasterising every
//! `face_svg` × `back_svg` output through `assets::rasterize_svg`,
//! and compares each hash to an embedded constant. Catches silent
//! rendering drift if `usvg`, `resvg`, `tiny_skia`, or the bundled
//! `FiraMono` font ever change in a way that perturbs the rendered
//! pixels.
//!
//! When the SVG builders intentionally change (or a dependency
//! upgrade legitimately changes rendering), update `EXPECTED` by
//! emptying it (`&[]`) and re-running this test once — the test
//! will panic with the new hashes formatted as Rust source ready
//! to paste back in.
//!
//! Hashing is FNV-1a 64-bit on the raw RGBA byte buffer. PNG
//! compression is intentionally not in the loop — we only want
//! the test to fire on actual pixel changes, not zlib-level
//! shifts that don't affect what a player sees.

use solitaire_engine::assets::card_face_svg::{
    back_svg, face_svg, rank_filename, suit_filename, ALL_RANKS, ALL_SUITS, BACK_ACCENTS, TARGET,
};
use solitaire_engine::assets::rasterize_svg;

const EXPECTED: &[(&str, u64)] = &[
    ("face_AC", 0x287e3293f95990a5),
    ("face_2C", 0x01c66d8e461fb0c4),
    ("face_3C", 0xfdae6be53af8b7c8),
    ("face_4C", 0x4b2a7aef966c6cc2),
    ("face_5C", 0xa4ca0ce3759b5cc9),
    ("face_6C", 0xe1a730d1ce810314),
    ("face_7C", 0x9c8de5c7d014eca3),
    ("face_8C", 0x39e09f90c957b192),
    ("face_9C", 0xd6627707fb2d5079),
    ("face_10C", 0xbe8411c60411195c),
    ("face_JC", 0x7c33abf5619477ac),
    ("face_QC", 0xe75657d63c99a892),
    ("face_KC", 0xf4a445b771026496),
    ("face_AD", 0xad8820c694c464d7),
    ("face_2D", 0xef771dbb39ae4f5a),
    ("face_3D", 0xe955ec9a96e1256a),
    ("face_4D", 0x6bb5979ef6004957),
    ("face_5D", 0x55715fd2353b2126),
    ("face_6D", 0x87fbd6efce1b1f9f),
    ("face_7D", 0xabb2d52d363e93ab),
    ("face_8D", 0xde78161ee9093b05),
    ("face_9D", 0x1475987ba1e66036),
    ("face_10D", 0x3a52d7fda7158aeb),
    ("face_JD", 0xc9078d8a7b2e6372),
    ("face_QD", 0x84c9011b916fdbe8),
    ("face_KD", 0xbcd20dbb6b1c8cdf),
    ("face_AH", 0x2c8e05964b5e3a5f),
    ("face_2H", 0xb44e68b79bb3842e),
    ("face_3H", 0x15226ed29769e1c4),
    ("face_4H", 0xe28c86ba92a3aee9),
    ("face_5H", 0x18276e48b28d0f6b),
    ("face_6H", 0xcca5e60e65724eaa),
    ("face_7H", 0x7f3eee634137f13a),
    ("face_8H", 0x8974515a8904d6c4),
    ("face_9H", 0x2f8155cd7690d4b9),
    ("face_10H", 0x78142f898fd66578),
    ("face_JH", 0x5e6df78654a1de73),
    ("face_QH", 0xc231ae8c25d877a9),
    ("face_KH", 0x55a0a772baf3e97f),
    ("face_AS", 0xc90e798aebdc1c5f),
    ("face_2S", 0x4178c699a726ea70),
    ("face_3S", 0xdfcd34480bb06f4c),
    ("face_4S", 0xdbd4938042afb02e),
    ("face_5S", 0x8741456ab1ec58ab),
    ("face_6S", 0x6d2632f648f1c34d),
    ("face_7S", 0x3c05c70ff3d93ea6),
    ("face_8S", 0x12d7f456efbaffe0),
    ("face_9S", 0x11b6ade208b8fa12),
    ("face_10S", 0x475d4110834b6b2a),
    ("face_JS", 0x52525a2200c07246),
    ("face_QS", 0xb4f0251a2757cbb1),
    ("face_KS", 0x1e1975919bb9a029),
    ("back_0", 0xfd1742ebe330481a),
    ("back_1", 0x446fdc0a3c83a03a),
    ("back_2", 0xcf188fdec9f5819a),
    ("back_3", 0xcaffd02af141743a),
    ("back_4", 0xcee8a700bbaaf71a),
];

#[test]
fn rasterised_card_bytes_match_pinned_hashes() {
    let actual = compute_actual_hashes();

    if EXPECTED.is_empty() {
        panic_with_hashes_to_paste(&actual);
    }

    assert_eq!(
        actual.len(),
        EXPECTED.len(),
        "card-output count drifted (actual {} vs expected {})",
        actual.len(),
        EXPECTED.len(),
    );

    let mut mismatches: Vec<String> = Vec::new();
    for ((actual_name, actual_hash), (expected_name, expected_hash)) in
        actual.iter().zip(EXPECTED.iter())
    {
        assert_eq!(
            actual_name, expected_name,
            "card-output naming/order drifted",
        );
        if actual_hash != expected_hash {
            mismatches.push(format!(
                "  {actual_name}: actual 0x{actual_hash:016x}  expected 0x{expected_hash:016x}",
            ));
        }
    }

    if !mismatches.is_empty() {
        let mut msg = String::from(
            "rasterised card bytes drifted from EXPECTED — usvg/resvg/tiny_skia/font upgrade?\n",
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

fn compute_actual_hashes() -> Vec<(String, u64)> {
    let mut out = Vec::with_capacity(ALL_RANKS.len() * ALL_SUITS.len() + BACK_ACCENTS.len());
    for suit in ALL_SUITS {
        for rank in ALL_RANKS {
            let name = format!("face_{}{}", rank_filename(rank), suit_filename(suit));
            out.push((name, hash_rasterised(&face_svg(rank, suit))));
        }
    }
    for (idx, accent) in BACK_ACCENTS.iter().enumerate() {
        out.push((format!("back_{idx}"), hash_rasterised(&back_svg(accent))));
    }
    out
}

fn hash_rasterised(svg: &str) -> u64 {
    let image = rasterize_svg(svg.as_bytes(), TARGET).expect("rasterise card SVG");
    let bytes = image.data.expect("rasterised image carries RGBA pixel data");
    fnv1a(&bytes)
}

/// FNV-1a 64-bit. Inline rather than a dependency — adding `sha2`
/// or `blake3` for ~5 lines of code would burn a CLAUDE.md §8
/// "ask before adding deps" round-trip for no real benefit.
/// Cryptographic strength isn't load-bearing here — we just need
/// stable byte fingerprints.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn panic_with_hashes_to_paste(actual: &[(String, u64)]) -> ! {
    let mut out = String::from(
        "\nEXPECTED is empty — paste the following into the const literal:\n\nconst EXPECTED: &[(&str, u64)] = &[\n",
    );
    for (name, hash) in actual {
        out.push_str(&format!("    (\"{name}\", 0x{hash:016x}),\n"));
    }
    out.push_str("];\n");
    panic!("{out}");
}
