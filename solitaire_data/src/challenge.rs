//! Static seed list for Challenge mode + helpers.
//!
//! Challenge mode walks a fixed sequence of hard-but-winnable seeds. The
//! player advances by winning a deal in `GameMode::Challenge`. The
//! `challenge_index` cursor is stored per-player in `PlayerProgress`.
//!
//! Seeds wrap modulo `CHALLENGE_SEEDS.len()` so a sufficiently dedicated
//! player never runs out of challenges.

/// Curated Challenge-mode seeds. Order is stable across versions; add new
/// seeds at the end.
pub const CHALLENGE_SEEDS: &[u64] = &[
    0xDEAD_BEEF_CAFE_F00D,
    0xC0DE_FACE_8BAD_F00D,
    0xFEE1_DEAD_DEAD_BEEF,
    0xBAAD_F00D_BAAD_F00D,
    0x1337_C0DE_4242_BABE,
];

/// Resolve a `challenge_index` to its corresponding seed, wrapping when
/// the index exceeds the seed-list length. Returns `None` if the seed list
/// is empty (defensive — `CHALLENGE_SEEDS` is non-empty by construction).
pub fn challenge_seed_for(index: u32) -> Option<u64> {
    if CHALLENGE_SEEDS.is_empty() {
        return None;
    }
    Some(CHALLENGE_SEEDS[(index as usize) % CHALLENGE_SEEDS.len()])
}

/// Total number of currently-defined challenges. Useful for displaying
/// "Challenge {n + 1} of {total}" in UI.
pub fn challenge_count() -> u32 {
    CHALLENGE_SEEDS.len() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn challenge_seed_for_0_is_first_seed() {
        assert_eq!(challenge_seed_for(0), Some(CHALLENGE_SEEDS[0]));
    }

    #[test]
    fn challenge_seed_wraps_past_end() {
        let len = CHALLENGE_SEEDS.len() as u32;
        assert_eq!(
            challenge_seed_for(len),
            Some(CHALLENGE_SEEDS[0]),
            "wraps to seed 0 when index == len"
        );
        assert_eq!(
            challenge_seed_for(len + 2),
            Some(CHALLENGE_SEEDS[2]),
            "wraps modulo len"
        );
    }

    #[test]
    fn all_challenge_seeds_are_unique() {
        let mut seeds: Vec<u64> = CHALLENGE_SEEDS.to_vec();
        seeds.sort();
        let len_before = seeds.len();
        seeds.dedup();
        assert_eq!(seeds.len(), len_before);
    }

    #[test]
    fn challenge_count_matches_seed_list_length() {
        assert_eq!(challenge_count() as usize, CHALLENGE_SEEDS.len());
    }
}
