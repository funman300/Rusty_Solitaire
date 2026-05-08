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
    ("face_AC", 0xecfed9881dab58cc),
    ("face_2C", 0x9d226854e375a071),
    ("face_3C", 0x57635bd3396b1c6f),
    ("face_4C", 0x3db4ebca46202411),
    ("face_5C", 0x6aaaa97f8d64d141),
    ("face_6C", 0x45ab0e3692f5086f),
    ("face_7C", 0xb6c6d47a9c41c042),
    ("face_8C", 0x95e467bebfe1f43f),
    ("face_9C", 0x78c67114e728f726),
    ("face_10C", 0x59ea22af2a519731),
    ("face_JC", 0x0757d9cae053863e),
    ("face_QC", 0xc3de9e10c2e8819e),
    ("face_KC", 0xefd2e9dd4c6f734f),
    ("face_AD", 0x95e2954416f7051d),
    ("face_2D", 0xfa494e129a7d130b),
    ("face_3D", 0x493f32ac1b4f1365),
    ("face_4D", 0x1303407818e3896d),
    ("face_5D", 0x3c68bc01d5661c9b),
    ("face_6D", 0x4ae0872812942c95),
    ("face_7D", 0xf4a040f288b53a3d),
    ("face_8D", 0xb5964ffbcc1834c0),
    ("face_9D", 0xfc2b244f9e6c987c),
    ("face_10D", 0xc9648dfd2f74e387),
    ("face_JD", 0x055c9e4b1f56b2b4),
    ("face_QD", 0x05d0d7e3be132b36),
    ("face_KD", 0x540753328025961e),
    ("face_AH", 0x8ac76ac84674dae6),
    ("face_2H", 0xf20c188bc5cf1008),
    ("face_3H", 0xc604901c0da15c0e),
    ("face_4H", 0x371c115d9292fa56),
    ("face_5H", 0x5cabef7840c6e378),
    ("face_6H", 0x48948872acab515e),
    ("face_7H", 0x49e96e37591f8c86),
    ("face_8H", 0xe30b740fd0f3575b),
    ("face_9H", 0x4067a838eeff2ea7),
    ("face_10H", 0xd9e9913fa5d9b974),
    ("face_JH", 0xe4344bff58d04e7f),
    ("face_QH", 0xf33df3f193827f25),
    ("face_KH", 0x8ada887b665fa3fd),
    ("face_AS", 0x586d5587ad518f46),
    ("face_2S", 0xbc0deb204e690d57),
    ("face_3S", 0xac04b5df8741d889),
    ("face_4S", 0x6a2ebcdb517b7ab7),
    ("face_5S", 0x9868f72763bbdae7),
    ("face_6S", 0x9a4c6842e0cbc489),
    ("face_7S", 0x15d17732dadf2ec0),
    ("face_8S", 0xb581df40dace0e59),
    ("face_9S", 0xce92a55ddcc6b4fc),
    ("face_10S", 0x1d92560a36938e97),
    ("face_JS", 0xd339b7a54139f9d4),
    ("face_QS", 0x59eae032af251c74),
    ("face_KS", 0x901e0d1ace6ff6a9),
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
