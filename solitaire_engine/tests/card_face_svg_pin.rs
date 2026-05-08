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
    ("face_AC", 0x615e0ae429f479d5),
    ("face_2C", 0xb882a01af5338788),
    ("face_3C", 0x2ca39e5044b20caa),
    ("face_4C", 0xf1439571d0877e48),
    ("face_5C", 0x30254c808947a0e8),
    ("face_6C", 0xb23c1bec933a3a2a),
    ("face_7C", 0x250d5dca1cd7f72f),
    ("face_8C", 0xe663eb85d871e71a),
    ("face_9C", 0xd60724ce062f1c3b),
    ("face_10C", 0x271554347971c038),
    ("face_JC", 0xe9e6d9105d4fdd73),
    ("face_QC", 0x3e9c1094ce524373),
    ("face_KC", 0x8bdd75d0c9b2f23a),
    ("face_AD", 0x452313abfc8ae82b),
    ("face_2D", 0x5e7bfe77ab0b28a5),
    ("face_3D", 0x327a1e905aea8beb),
    ("face_4D", 0x86a4d1f243c60687),
    ("face_5D", 0x0e806a2b7350efc5),
    ("face_6D", 0x18150445cdba5fcb),
    ("face_7D", 0x25891a7b57050f7f),
    ("face_8D", 0x17096711946662be),
    ("face_9D", 0x1015f68680fc63b6),
    ("face_10D", 0x828bb4a68d291b3d),
    ("face_JD", 0x07b88b412f1357de),
    ("face_QD", 0x79d83db7e08d6338),
    ("face_KD", 0x72e59e0b36af3ac0),
    ("face_AH", 0xe8591acb1b311f68),
    ("face_2H", 0x0ecbabd6851a6e06),
    ("face_3H", 0x26f618607d72fb28),
    ("face_4H", 0xd678c1b9fe409d54),
    ("face_5H", 0xc6d600ca7b935aa6),
    ("face_6H", 0xcccc4f21cdf2c708),
    ("face_7H", 0x5c73195762121eec),
    ("face_8H", 0xc8e5adc5a1878635),
    ("face_9H", 0xfc1a1962879e3fed),
    ("face_10H", 0x0e2bcd01a63bb11e),
    ("face_JH", 0x9b18ac201230d355),
    ("face_QH", 0xc98e562402c11083),
    ("face_KH", 0x36a1eca09821b25b),
    ("face_AS", 0x0a62cb01f3c6a27b),
    ("face_2S", 0xa42a55c0df68c582),
    ("face_3S", 0xf789fa97b5e9fff0),
    ("face_4S", 0x7ffe2bc702a019c2),
    ("face_5S", 0xe38731d462109022),
    ("face_6S", 0xf7e7570631786c70),
    ("face_7S", 0x4b70162e6a977a91),
    ("face_8S", 0xe45989c24d21fda0),
    ("face_9S", 0x5ae9856cd14f6e65),
    ("face_10S", 0x77ff64bff391d7f2),
    ("face_JS", 0xb27564785cb9d07d),
    ("face_QS", 0x072a9bfccdbc367d),
    ("face_KS", 0xbd1464c949ffa380),
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
