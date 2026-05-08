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
    ("face_AC", 0x79b449cb455e496d),
    ("face_2C", 0x10a1056c4800c45e),
    ("face_3C", 0xbd128e390e06673a),
    ("face_4C", 0x949c323c78a804c0),
    ("face_5C", 0xd396d5ed99fb57e9),
    ("face_6C", 0x15519c6d72d1720f),
    ("face_7C", 0xc24bdc1a2d380d78),
    ("face_8C", 0x36464f4ab4cf672e),
    ("face_9C", 0x32add2eb53b1aec4),
    ("face_10C", 0x68619202f29481fc),
    ("face_JC", 0x116b3eeac58e0f58),
    ("face_QC", 0xb149ab5b2cac85e3),
    ("face_KC", 0x2a9fd2c63b99bd3b),
    ("face_AD", 0xe49c3fec2c01817c),
    ("face_2D", 0x8f42b4014e0d6809),
    ("face_3D", 0x63ff77fa873c557b),
    ("face_4D", 0x33356bd9628daaf2),
    ("face_5D", 0x8897839054dbd808),
    ("face_6D", 0x03ff93fb0c05a195),
    ("face_7D", 0xc2b7f97f5b1cc545),
    ("face_8D", 0xd8515a8278d74a7b),
    ("face_9D", 0xfbfe52ec3bbd2962),
    ("face_10D", 0x8f2dfc06a1d55a2f),
    ("face_JD", 0x3941d34384607530),
    ("face_QD", 0x0dcf5a9e2fc99f02),
    ("face_KD", 0xb834cb89d80bd39c),
    ("face_AH", 0x1a2e6d2ac818093f),
    ("face_2H", 0x8ab9ad7d2111233e),
    ("face_3H", 0x5e1057fa87c90968),
    ("face_4H", 0x1e1550b0af8a35a5),
    ("face_5H", 0x77404642251596d3),
    ("face_6H", 0xf7bec77bcbb9f942),
    ("face_7H", 0x9b7c52a5c03fb4f2),
    ("face_8H", 0xd2623a827963fe68),
    ("face_9H", 0xec19380e53986015),
    ("face_10H", 0x1205d0ec042a7484),
    ("face_JH", 0xd28bf03e6e871ccb),
    ("face_QH", 0x78548704b4530c65),
    ("face_KH", 0x9708e6c2d9c3bedf),
    ("face_AS", 0xebabc54128f38105),
    ("face_2S", 0xaac2970387b18ffe),
    ("face_3S", 0xb0864e78a6802bea),
    ("face_4S", 0xd118bc992bd41330),
    ("face_5S", 0x7fb7d6040d9b0641),
    ("face_6S", 0xbc048e82f1079637),
    ("face_7S", 0x147ee7c002e43648),
    ("face_8S", 0xfed30db056fbaa8e),
    ("face_9S", 0x332bc2060d8fcca4),
    ("face_10S", 0x0b810ffaf105421c),
    ("face_JS", 0x2ea7b956f2f23c28),
    ("face_QS", 0xedca2e002087ae6b),
    ("face_KS", 0x92e486d4e96ac4a3),
    ("back_0", 0xf698d0e161eae13a),
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
