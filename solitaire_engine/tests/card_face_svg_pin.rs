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
    ("face_AC", 0xca11dff5bb9f0eb0),
    ("face_2C", 0xc929a25f0f217577),
    ("face_3C", 0xdaede8383266b5c3),
    ("face_4C", 0xeaa3ea51866f69e5),
    ("face_5C", 0xe5a74589cb09cc5c),
    ("face_6C", 0xdbbc1036895ee08e),
    ("face_7C", 0xb8a28119a85ccf5d),
    ("face_8C", 0xab4d19ce4b8d15e7),
    ("face_9C", 0x17c95eb07f382059),
    ("face_10C", 0x1f1b2c84e42211b1),
    ("face_JC", 0xd87c45124df8b03d),
    ("face_QC", 0xe23701b6685994b2),
    ("face_KC", 0xc628e55b8a15472a),
    ("face_AD", 0x49a140d84b0a731b),
    ("face_2D", 0x713f755b5ecfb67a),
    ("face_3D", 0xe59a72abc47af7d4),
    ("face_4D", 0xf75ac828822079d1),
    ("face_5D", 0x6db0cc9a5849395f),
    ("face_6D", 0x9b034cf6851512de),
    ("face_7D", 0x85f96e0326780a6e),
    ("face_8D", 0x59ec5533b615ecd4),
    ("face_9D", 0x3689911671b30921),
    ("face_10D", 0x682684217e3e8b60),
    ("face_JD", 0xd999f85e6862c5a7),
    ("face_QD", 0x6db493a3b370b211),
    ("face_KD", 0x4c2ec19166fdee7b),
    ("face_AH", 0x0d41c498281b9a74),
    ("face_2H", 0xec6493b71d4576b1),
    ("face_3H", 0xd2fb4b5956caf15b),
    ("face_4H", 0xfbe8e1eaa2b28c5a),
    ("face_5H", 0x649a0964e549f008),
    ("face_6H", 0xa10fa42b5549fc85),
    ("face_7H", 0x6823107295c149b5),
    ("face_8H", 0x474d2de14865e65b),
    ("face_9H", 0x1b0de1af8dae108a),
    ("face_10H", 0x451fd5855859c9d7),
    ("face_JH", 0xd821a7d4c79a37e0),
    ("face_QH", 0xde0c6ef7e963861a),
    ("face_KH", 0xe29039cb6a115214),
    ("face_AS", 0x1697fbcc61b64e0f),
    ("face_2S", 0x5ada7ea3e39547d0),
    ("face_3S", 0x6d8eed531f2d659c),
    ("face_4S", 0x1b1a2d25e080d71e),
    ("face_5S", 0x5eb82baa4f9a74bb),
    ("face_6S", 0xa00b217892d32ead),
    ("face_7S", 0xaf60935ec8d93346),
    ("face_8S", 0xffbde852d8699a80),
    ("face_9S", 0x8f68afa04b88e1a2),
    ("face_10S", 0x96fa4a08f168210a),
    ("face_JS", 0x73030a8109b5b5e6),
    ("face_QS", 0x303eb6c33e363cc1),
    ("face_KS", 0x3ed5b5a9432c91e9),
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
