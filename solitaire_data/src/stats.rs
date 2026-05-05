//! Player statistics — persisted to `stats.json` between sessions.
//!
//! [`StatsSnapshot`] is defined in `solitaire_sync` and re-exported here.
//! This module adds the [`StatsExt`] extension trait, which supplies the
//! `update_on_win` method that depends on [`DrawMode`] from `solitaire_core`.

use chrono::Utc;
use solitaire_core::game_state::{DrawMode, GameMode};

pub use solitaire_sync::StatsSnapshot;

/// Extension trait providing game-logic mutation helpers for [`StatsSnapshot`].
///
/// Import this trait alongside `StatsSnapshot` to use `update_on_win`
/// and [`StatsExt::update_per_mode_bests`].
pub trait StatsExt {
    /// Updates rolling statistics from a completed game win. Call once per `GameWonEvent`.
    ///
    /// Tracks lifetime totals only — per-mode best scores and times are
    /// updated separately via [`StatsExt::update_per_mode_bests`] so the
    /// long-standing call sites that only know about [`DrawMode`] keep
    /// compiling.
    fn update_on_win(&mut self, score: i32, time_seconds: u64, draw_mode: &DrawMode);

    /// Updates the per-mode best score and fastest-win-time fields for the
    /// given [`GameMode`]. Call alongside [`StatsExt::update_on_win`] from
    /// the win handler.
    ///
    /// Behaviour:
    /// - `Classic`, `Zen`, `Challenge`: updates the matching `*_best_score`
    ///   (max) and `*_fastest_win_seconds` (zero-aware min — 0 means
    ///   "no win recorded yet").
    /// - `TimeAttack`: no-op. Time Attack uses session-level scoring (count
    ///   of wins in 10 minutes); a per-game best wouldn't compose with
    ///   the other modes' single-game scoring.
    fn update_per_mode_bests(&mut self, score: i32, time_seconds: u64, mode: GameMode);
}

impl StatsExt for StatsSnapshot {
    fn update_on_win(&mut self, score: i32, time_seconds: u64, draw_mode: &DrawMode) {
        let prev_wins = self.games_won;
        self.games_played += 1;
        self.games_won += 1;
        self.win_streak_current += 1;
        if self.win_streak_current > self.win_streak_best {
            self.win_streak_best = self.win_streak_current;
        }

        let score_u32 = score.max(0) as u32;
        self.lifetime_score = self.lifetime_score.saturating_add(score_u32 as u64);
        if score_u32 > self.best_single_score {
            self.best_single_score = score_u32;
        }

        if time_seconds < self.fastest_win_seconds {
            self.fastest_win_seconds = time_seconds;
        }

        self.avg_time_seconds = if prev_wins == 0 {
            time_seconds
        } else {
            ((self.avg_time_seconds as u128 * prev_wins as u128 + time_seconds as u128)
                / self.games_won as u128) as u64
        };

        match draw_mode {
            DrawMode::DrawOne => self.draw_one_wins += 1,
            DrawMode::DrawThree => self.draw_three_wins += 1,
        }

        self.last_modified = Utc::now();
    }

    fn update_per_mode_bests(&mut self, score: i32, time_seconds: u64, mode: GameMode) {
        let score_u32 = score.max(0) as u32;
        // Zero-aware min — 0 means "no win recorded yet" for the per-mode
        // fastest fields, so we must not let a real time get clobbered to 0.
        // (Mirrors the merge logic in `solitaire_sync::merge`.)
        let min_ignore_zero = |existing: u64, candidate: u64| -> u64 {
            if existing == 0 {
                candidate
            } else if candidate == 0 {
                existing
            } else {
                existing.min(candidate)
            }
        };
        match mode {
            GameMode::Classic => {
                self.classic_best_score = self.classic_best_score.max(score_u32);
                self.classic_fastest_win_seconds =
                    min_ignore_zero(self.classic_fastest_win_seconds, time_seconds);
            }
            GameMode::Zen => {
                self.zen_best_score = self.zen_best_score.max(score_u32);
                self.zen_fastest_win_seconds =
                    min_ignore_zero(self.zen_fastest_win_seconds, time_seconds);
            }
            GameMode::Challenge => {
                self.challenge_best_score = self.challenge_best_score.max(score_u32);
                self.challenge_fastest_win_seconds =
                    min_ignore_zero(self.challenge_fastest_win_seconds, time_seconds);
            }
            // Time Attack uses its own session-level scoring; a per-game best
            // wouldn't compose with the other modes' single-game numbers.
            GameMode::TimeAttack => {}
        }
        self.last_modified = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_stats_are_all_zero() {
        let s = StatsSnapshot::default();
        assert_eq!(s.games_played, 0);
        assert_eq!(s.games_won, 0);
        assert_eq!(s.win_streak_current, 0);
        assert_eq!(s.win_streak_best, 0);
        assert_eq!(s.lifetime_score, 0);
        assert_eq!(s.best_single_score, 0);
        assert_eq!(s.fastest_win_seconds, u64::MAX);
    }

    #[test]
    fn first_win_sets_all_fields() {
        let mut s = StatsSnapshot::default();
        s.update_on_win(1500, 120, &DrawMode::DrawOne);
        assert_eq!(s.games_played, 1);
        assert_eq!(s.games_won, 1);
        assert_eq!(s.win_streak_current, 1);
        assert_eq!(s.win_streak_best, 1);
        assert_eq!(s.lifetime_score, 1500);
        assert_eq!(s.best_single_score, 1500);
        assert_eq!(s.fastest_win_seconds, 120);
        assert_eq!(s.avg_time_seconds, 120);
        assert_eq!(s.draw_one_wins, 1);
        assert_eq!(s.draw_three_wins, 0);
    }

    #[test]
    fn streak_tracks_across_wins() {
        let mut s = StatsSnapshot::default();
        for _ in 0..3 {
            s.update_on_win(100, 60, &DrawMode::DrawOne);
        }
        assert_eq!(s.win_streak_current, 3);
        assert_eq!(s.win_streak_best, 3);
    }

    #[test]
    fn record_abandoned_resets_streak_and_increments_played() {
        let mut s = StatsSnapshot::default();
        s.update_on_win(100, 60, &DrawMode::DrawOne);
        s.update_on_win(100, 60, &DrawMode::DrawOne);
        assert_eq!(s.win_streak_current, 2);
        s.record_abandoned();
        assert_eq!(s.games_played, 3);
        assert_eq!(s.games_lost, 1);
        assert_eq!(s.win_streak_current, 0);
        assert_eq!(s.win_streak_best, 2);
    }

    #[test]
    fn fastest_win_takes_minimum() {
        let mut s = StatsSnapshot::default();
        s.update_on_win(100, 300, &DrawMode::DrawOne);
        s.update_on_win(100, 120, &DrawMode::DrawOne);
        s.update_on_win(100, 500, &DrawMode::DrawOne);
        assert_eq!(s.fastest_win_seconds, 120);
    }

    #[test]
    fn avg_time_is_correct_rolling_average() {
        let mut s = StatsSnapshot::default();
        s.update_on_win(100, 100, &DrawMode::DrawOne);
        s.update_on_win(100, 200, &DrawMode::DrawOne);
        s.update_on_win(100, 300, &DrawMode::DrawOne);
        assert_eq!(s.avg_time_seconds, 200);
    }

    #[test]
    fn best_score_updates_only_on_higher_score() {
        let mut s = StatsSnapshot::default();
        s.update_on_win(500, 60, &DrawMode::DrawOne);
        s.update_on_win(300, 60, &DrawMode::DrawOne);
        assert_eq!(s.best_single_score, 500);
        s.update_on_win(800, 60, &DrawMode::DrawOne);
        assert_eq!(s.best_single_score, 800);
    }

    #[test]
    fn negative_score_treated_as_zero() {
        let mut s = StatsSnapshot::default();
        s.update_on_win(-50, 60, &DrawMode::DrawOne);
        assert_eq!(s.best_single_score, 0);
        assert_eq!(s.lifetime_score, 0);
    }

    #[test]
    fn draw_three_wins_tracked_separately() {
        let mut s = StatsSnapshot::default();
        s.update_on_win(100, 60, &DrawMode::DrawOne);
        s.update_on_win(100, 60, &DrawMode::DrawThree);
        assert_eq!(s.draw_one_wins, 1);
        assert_eq!(s.draw_three_wins, 1);
    }

    #[test]
    fn win_streak_best_never_decreases_after_shorter_subsequent_streak() {
        let mut s = StatsSnapshot::default();
        // Build a streak of 5.
        for _ in 0..5 {
            s.update_on_win(100, 60, &DrawMode::DrawOne);
        }
        assert_eq!(s.win_streak_best, 5);
        // Lose (abandon), resetting current.
        s.record_abandoned();
        assert_eq!(s.win_streak_current, 0);
        assert_eq!(s.win_streak_best, 5, "best must survive the loss");
        // Win once — current becomes 1, best must remain 5.
        s.update_on_win(100, 60, &DrawMode::DrawOne);
        assert_eq!(s.win_streak_current, 1);
        assert_eq!(s.win_streak_best, 5, "best must not drop to match shorter streak");
    }

    #[test]
    fn lifetime_score_saturates_at_u64_max() {
        let mut s = StatsSnapshot { lifetime_score: u64::MAX - 100, ..Default::default() };
        s.update_on_win(200, 60, &DrawMode::DrawOne);
        assert_eq!(s.lifetime_score, u64::MAX, "lifetime_score must saturate, not overflow");
    }

    // -----------------------------------------------------------------------
    // Per-mode bests
    // -----------------------------------------------------------------------

    #[test]
    fn classic_win_updates_classic_best_score_only() {
        let mut s = StatsSnapshot::default();
        s.update_per_mode_bests(1500, 200, GameMode::Classic);
        assert_eq!(s.classic_best_score, 1500);
        assert_eq!(s.classic_fastest_win_seconds, 200);
        // Other modes untouched.
        assert_eq!(s.zen_best_score, 0);
        assert_eq!(s.zen_fastest_win_seconds, 0);
        assert_eq!(s.challenge_best_score, 0);
        assert_eq!(s.challenge_fastest_win_seconds, 0);
    }

    #[test]
    fn zen_win_updates_zen_best_score_only() {
        let mut s = StatsSnapshot::default();
        s.update_per_mode_bests(1800, 600, GameMode::Zen);
        assert_eq!(s.zen_best_score, 1800);
        assert_eq!(s.zen_fastest_win_seconds, 600);
        assert_eq!(s.classic_best_score, 0);
        assert_eq!(s.challenge_best_score, 0);
    }

    #[test]
    fn challenge_win_updates_challenge_best_score_only() {
        let mut s = StatsSnapshot::default();
        s.update_per_mode_bests(2400, 480, GameMode::Challenge);
        assert_eq!(s.challenge_best_score, 2400);
        assert_eq!(s.challenge_fastest_win_seconds, 480);
        assert_eq!(s.classic_best_score, 0);
        assert_eq!(s.zen_best_score, 0);
    }

    #[test]
    fn time_attack_win_does_not_touch_per_mode_bests() {
        let mut s = StatsSnapshot::default();
        s.update_per_mode_bests(9999, 1, GameMode::TimeAttack);
        assert_eq!(s.classic_best_score, 0);
        assert_eq!(s.zen_best_score, 0);
        assert_eq!(s.challenge_best_score, 0);
        assert_eq!(s.classic_fastest_win_seconds, 0);
        assert_eq!(s.zen_fastest_win_seconds, 0);
        assert_eq!(s.challenge_fastest_win_seconds, 0);
    }

    #[test]
    fn per_mode_best_score_takes_max_across_calls() {
        let mut s = StatsSnapshot::default();
        s.update_per_mode_bests(500, 200, GameMode::Classic);
        s.update_per_mode_bests(200, 200, GameMode::Classic);
        s.update_per_mode_bests(900, 200, GameMode::Classic);
        assert_eq!(s.classic_best_score, 900);
    }

    #[test]
    fn per_mode_fastest_uses_zero_aware_min() {
        // First Classic win: 240s. Field starts at 0 (no win yet) — we
        // must adopt 240, not stay at 0 like a naive `min` would.
        let mut s = StatsSnapshot::default();
        s.update_per_mode_bests(100, 240, GameMode::Classic);
        assert_eq!(s.classic_fastest_win_seconds, 240);
        // Faster Classic win replaces it.
        s.update_per_mode_bests(100, 120, GameMode::Classic);
        assert_eq!(s.classic_fastest_win_seconds, 120);
        // Slower Classic win does not.
        s.update_per_mode_bests(100, 300, GameMode::Classic);
        assert_eq!(s.classic_fastest_win_seconds, 120);
    }

    #[test]
    fn negative_score_treated_as_zero_in_per_mode() {
        let mut s = StatsSnapshot::default();
        s.update_per_mode_bests(-50, 240, GameMode::Classic);
        assert_eq!(s.classic_best_score, 0);
        // Time still recorded — a win with a low score is still a win.
        assert_eq!(s.classic_fastest_win_seconds, 240);
    }

    #[test]
    fn legacy_stats_without_per_mode_fields_deserializes_to_zero() {
        // A pre-per-mode `stats.json` must still deserialise cleanly:
        // every new field falls back to 0 via `#[serde(default)]` so
        // updating the binary never wipes the player's old stats file.
        let legacy_json = r#"{
            "games_played": 12,
            "games_won": 5,
            "games_lost": 7,
            "win_streak_current": 1,
            "win_streak_best": 3,
            "avg_time_seconds": 240,
            "fastest_win_seconds": 180,
            "lifetime_score": 8500,
            "best_single_score": 2200,
            "draw_one_wins": 4,
            "draw_three_wins": 1,
            "last_modified": "2026-04-29T12:00:00Z"
        }"#;

        let s: StatsSnapshot = serde_json::from_str(legacy_json)
            .expect("legacy payload must deserialise without per-mode fields");

        // Pre-existing fields kept their values.
        assert_eq!(s.games_played, 12);
        assert_eq!(s.best_single_score, 2200);
        assert_eq!(s.fastest_win_seconds, 180);

        // Every new per-mode field defaulted to 0 ("no win yet").
        assert_eq!(s.classic_best_score, 0);
        assert_eq!(s.classic_fastest_win_seconds, 0);
        assert_eq!(s.zen_best_score, 0);
        assert_eq!(s.zen_fastest_win_seconds, 0);
        assert_eq!(s.challenge_best_score, 0);
        assert_eq!(s.challenge_fastest_win_seconds, 0);
    }
}
