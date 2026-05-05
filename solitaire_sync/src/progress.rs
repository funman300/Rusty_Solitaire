//! Shared `PlayerProgress` definition — used by both the game client and the
//! sync server.

use std::collections::HashMap;

use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// XP-to-level calculation per ARCHITECTURE.md §13.
///
/// - Levels 1–10:  `level = floor(total_xp / 500)`
/// - Levels 11+:   `level = 10 + floor((total_xp - 5_000) / 1_000)`
pub fn level_for_xp(xp: u64) -> u32 {
    if xp < 5_000 {
        (xp / 500) as u32
    } else {
        10 + ((xp - 5_000) / 1_000) as u32
    }
}

/// Maximum number of dates retained in [`PlayerProgress::daily_challenge_history`].
///
/// Bounds the per-player file size across years of play. ~365 entries is
/// roughly a year of daily completions, far more than the 14-day window the
/// in-game calendar surfaces.
pub const DAILY_CHALLENGE_HISTORY_CAP: usize = 365;

/// Persisted player progression state.
///
/// Mutation helpers such as `add_xp`, `record_daily_completion`, etc. are
/// defined as inherent methods directly on this type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerProgress {
    /// Total XP accumulated across all games.
    pub total_xp: u64,
    /// Current player level, recomputed from `total_xp`.
    pub level: u32,
    /// Date of the last completed daily challenge, if any.
    pub daily_challenge_last_completed: Option<NaiveDate>,
    /// Current daily-challenge streak length.
    pub daily_challenge_streak: u32,
    /// Per-goal progress counters for the current ISO week.
    pub weekly_goal_progress: HashMap<String, u32>,
    /// ISO week key (e.g. `"2026-W17"`) the `weekly_goal_progress` counters
    /// belong to. Cleared when a new week begins.
    #[serde(default)]
    pub weekly_goal_week_iso: Option<String>,
    /// Indices of card-back designs the player has unlocked (index 0 is always unlocked).
    pub unlocked_card_backs: Vec<usize>,
    /// Indices of background designs the player has unlocked (index 0 is always unlocked).
    pub unlocked_backgrounds: Vec<usize>,
    /// Index of the next Challenge-mode seed to serve to this player.
    #[serde(default)]
    pub challenge_index: u32,
    /// All dates the player has completed the daily challenge, in
    /// chronological ascending order. Bounded to the most recent 365
    /// entries so file size stays bounded across years of play.
    #[serde(default)]
    pub daily_challenge_history: Vec<NaiveDate>,
    /// Longest daily-challenge streak ever achieved on this profile.
    #[serde(default)]
    pub daily_challenge_longest_streak: u32,
    /// Wall-clock time of the last modification (used for conflict detection).
    pub last_modified: DateTime<Utc>,
}

impl Default for PlayerProgress {
    fn default() -> Self {
        Self {
            total_xp: 0,
            level: 0,
            daily_challenge_last_completed: None,
            daily_challenge_streak: 0,
            weekly_goal_progress: HashMap::new(),
            weekly_goal_week_iso: None,
            unlocked_card_backs: vec![0],
            unlocked_backgrounds: vec![0],
            challenge_index: 0,
            daily_challenge_history: Vec::new(),
            daily_challenge_longest_streak: 0,
            last_modified: DateTime::UNIX_EPOCH,
        }
    }
}

impl PlayerProgress {
    /// Add XP and recompute level. Returns the previous level so callers can
    /// detect level-up events.
    pub fn add_xp(&mut self, amount: u64) -> u32 {
        let prev_level = self.level;
        self.total_xp = self.total_xp.saturating_add(amount);
        self.level = level_for_xp(self.total_xp);
        self.last_modified = Utc::now();
        prev_level
    }

    /// `true` if a level-up just occurred (current level > `prev_level`).
    pub fn leveled_up_from(&self, prev_level: u32) -> bool {
        self.level > prev_level
    }

    /// Reset weekly-goal progress when the ISO week has rolled over.
    /// No-op if the stored week key already matches `current`.
    pub fn roll_weekly_goals_if_new_week(&mut self, current: &str) -> bool {
        if self.weekly_goal_week_iso.as_deref() == Some(current) {
            return false;
        }
        self.weekly_goal_progress.clear();
        self.weekly_goal_week_iso = Some(current.to_string());
        self.last_modified = Utc::now();
        true
    }

    /// Increment progress for `goal_id` by 1, capped at `target`.
    ///
    /// Returns `true` if this call brought the counter from below `target`
    /// to at-or-above `target` (i.e. just completed the goal).
    pub fn record_weekly_progress(&mut self, goal_id: &str, target: u32) -> bool {
        let entry = self.weekly_goal_progress.entry(goal_id.to_string()).or_insert(0);
        if *entry >= target {
            return false;
        }
        *entry = entry.saturating_add(1);
        self.last_modified = Utc::now();
        *entry >= target
    }

    /// Record a daily-challenge completion for `date`.
    ///
    /// - First completion ever, or a gap of more than one day: streak resets to 1.
    /// - Completion the day after the previous: streak increments.
    /// - Same day as the previous: no-op (idempotent).
    ///
    /// On every fresh completion, `date` is appended to
    /// `daily_challenge_history` (kept sorted ascending and capped at
    /// [`DAILY_CHALLENGE_HISTORY_CAP`] entries) and
    /// `daily_challenge_longest_streak` is bumped if the current streak
    /// exceeds it.
    ///
    /// Returns `true` if this call recorded a fresh completion.
    pub fn record_daily_completion(&mut self, date: NaiveDate) -> bool {
        match self.daily_challenge_last_completed {
            Some(last) if last == date => return false,
            Some(last) if last + Duration::days(1) == date => {
                self.daily_challenge_streak = self.daily_challenge_streak.saturating_add(1);
            }
            _ => {
                self.daily_challenge_streak = 1;
            }
        }
        self.daily_challenge_last_completed = Some(date);
        // Append to history (defensive against duplicates and out-of-order
        // dates so a hand-edited or merged file can't corrupt the order).
        if !self.daily_challenge_history.contains(&date) {
            self.daily_challenge_history.push(date);
            self.daily_challenge_history.sort();
            if self.daily_challenge_history.len() > DAILY_CHALLENGE_HISTORY_CAP {
                let excess = self.daily_challenge_history.len() - DAILY_CHALLENGE_HISTORY_CAP;
                self.daily_challenge_history.drain(0..excess);
            }
        }
        if self.daily_challenge_streak > self.daily_challenge_longest_streak {
            self.daily_challenge_longest_streak = self.daily_challenge_streak;
        }
        self.last_modified = Utc::now();
        true
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    // -----------------------------------------------------------------------
    // level_for_xp
    // -----------------------------------------------------------------------

    #[test]
    fn level_zero_at_zero_xp() {
        assert_eq!(level_for_xp(0), 0);
    }

    #[test]
    fn level_one_at_500_xp() {
        assert_eq!(level_for_xp(500), 1);
    }

    #[test]
    fn level_nine_at_4500_xp() {
        assert_eq!(level_for_xp(4_500), 9);
    }

    #[test]
    fn level_ten_at_5000_xp() {
        assert_eq!(level_for_xp(5_000), 10);
    }

    #[test]
    fn level_eleven_at_6000_xp() {
        assert_eq!(level_for_xp(6_000), 11);
    }

    #[test]
    fn level_scales_correctly_above_ten() {
        // Level 10 + floor((7000 - 5000) / 1000) = 10 + 2 = 12
        assert_eq!(level_for_xp(7_000), 12);
    }

    // -----------------------------------------------------------------------
    // add_xp
    // -----------------------------------------------------------------------

    #[test]
    fn add_xp_increases_total_xp() {
        let mut p = PlayerProgress::default();
        p.add_xp(300);
        assert_eq!(p.total_xp, 300);
    }

    #[test]
    fn add_xp_returns_previous_level() {
        let mut p = PlayerProgress::default();
        p.add_xp(400); // still level 0
        let prev = p.add_xp(200); // crosses into level 1
        assert_eq!(prev, 0, "returned level should be the pre-call level");
        assert_eq!(p.level, 1);
    }

    #[test]
    fn add_xp_saturates_on_overflow() {
        let mut p = PlayerProgress { total_xp: u64::MAX, ..Default::default() };
        p.add_xp(1);
        assert_eq!(p.total_xp, u64::MAX);
    }

    // -----------------------------------------------------------------------
    // leveled_up_from
    // -----------------------------------------------------------------------

    #[test]
    fn leveled_up_from_returns_true_when_level_increased() {
        let mut p = PlayerProgress::default();
        p.add_xp(600); // reaches level 1
        assert!(p.leveled_up_from(0));
    }

    #[test]
    fn leveled_up_from_returns_false_when_same_level() {
        let p = PlayerProgress::default();
        assert!(!p.leveled_up_from(0));
    }

    // -----------------------------------------------------------------------
    // roll_weekly_goals_if_new_week
    // -----------------------------------------------------------------------

    #[test]
    fn roll_weekly_goals_clears_progress_for_new_week() {
        let mut p = PlayerProgress { weekly_goal_week_iso: Some("2026-W16".to_string()), ..Default::default() };
        p.weekly_goal_progress.insert("weekly_5_wins".to_string(), 3);

        let rolled = p.roll_weekly_goals_if_new_week("2026-W17");
        assert!(rolled);
        assert!(p.weekly_goal_progress.is_empty());
        assert_eq!(p.weekly_goal_week_iso, Some("2026-W17".to_string()));
    }

    #[test]
    fn roll_weekly_goals_is_noop_for_same_week() {
        let mut p = PlayerProgress { weekly_goal_week_iso: Some("2026-W17".to_string()), ..Default::default() };
        p.weekly_goal_progress.insert("weekly_5_wins".to_string(), 2);

        let rolled = p.roll_weekly_goals_if_new_week("2026-W17");
        assert!(!rolled);
        assert_eq!(p.weekly_goal_progress.get("weekly_5_wins"), Some(&2));
    }

    // -----------------------------------------------------------------------
    // record_weekly_progress
    // -----------------------------------------------------------------------

    #[test]
    fn record_weekly_progress_increments_counter() {
        let mut p = PlayerProgress::default();
        p.roll_weekly_goals_if_new_week("2026-W17");
        let done = p.record_weekly_progress("weekly_5_wins", 5);
        assert!(!done, "1/5 should not be done");
        assert_eq!(p.weekly_goal_progress.get("weekly_5_wins"), Some(&1));
    }

    #[test]
    fn record_weekly_progress_returns_true_on_completion() {
        let mut p = PlayerProgress::default();
        p.roll_weekly_goals_if_new_week("2026-W17");
        for _ in 0..4 {
            p.record_weekly_progress("weekly_5_wins", 5);
        }
        let done = p.record_weekly_progress("weekly_5_wins", 5);
        assert!(done, "5th increment should complete the goal");
    }

    #[test]
    fn record_weekly_progress_does_not_exceed_target() {
        let mut p = PlayerProgress::default();
        p.roll_weekly_goals_if_new_week("2026-W17");
        for _ in 0..10 {
            p.record_weekly_progress("weekly_5_wins", 5);
        }
        // Counter must be capped at target — never go above.
        assert_eq!(p.weekly_goal_progress.get("weekly_5_wins"), Some(&5));
    }

    // -----------------------------------------------------------------------
    // record_daily_completion
    // -----------------------------------------------------------------------

    #[test]
    fn record_daily_completion_starts_streak_at_one() {
        let mut p = PlayerProgress::default();
        let recorded = p.record_daily_completion(date(2026, 4, 20));
        assert!(recorded);
        assert_eq!(p.daily_challenge_streak, 1);
        assert_eq!(p.daily_challenge_last_completed, Some(date(2026, 4, 20)));
    }

    #[test]
    fn record_daily_completion_same_day_is_noop() {
        let mut p = PlayerProgress::default();
        p.record_daily_completion(date(2026, 4, 20));
        let recorded = p.record_daily_completion(date(2026, 4, 20));
        assert!(!recorded);
        assert_eq!(p.daily_challenge_streak, 1, "streak must not double-count same day");
    }

    #[test]
    fn record_daily_completion_consecutive_days_extend_streak() {
        let mut p = PlayerProgress::default();
        p.record_daily_completion(date(2026, 4, 20));
        p.record_daily_completion(date(2026, 4, 21));
        assert_eq!(p.daily_challenge_streak, 2);
    }

    #[test]
    fn record_daily_completion_gap_resets_streak_to_one() {
        let mut p = PlayerProgress::default();
        p.record_daily_completion(date(2026, 4, 20));
        p.record_daily_completion(date(2026, 4, 22)); // skip the 21st
        assert_eq!(p.daily_challenge_streak, 1, "gap must reset streak");
    }

    // -----------------------------------------------------------------------
    // record_daily_completion — history + longest-streak side effects
    // -----------------------------------------------------------------------

    #[test]
    fn record_daily_completion_appends_to_history_in_chronological_order() {
        let mut p = PlayerProgress::default();
        assert!(p.daily_challenge_history.is_empty());
        p.record_daily_completion(date(2026, 4, 20));
        p.record_daily_completion(date(2026, 4, 21));
        p.record_daily_completion(date(2026, 4, 22));
        assert_eq!(
            p.daily_challenge_history,
            vec![
                date(2026, 4, 20),
                date(2026, 4, 21),
                date(2026, 4, 22),
            ],
            "history should hold all three completions in ascending order"
        );
    }

    #[test]
    fn record_daily_completion_same_day_does_not_duplicate_history() {
        let mut p = PlayerProgress::default();
        p.record_daily_completion(date(2026, 4, 20));
        p.record_daily_completion(date(2026, 4, 20));
        assert_eq!(
            p.daily_challenge_history,
            vec![date(2026, 4, 20)],
            "same-day completion is a no-op and must not duplicate history"
        );
    }

    #[test]
    fn record_daily_completion_updates_longest_streak() {
        let mut p = PlayerProgress::default();
        // Three-day streak: longest jumps from 0 → 3.
        p.record_daily_completion(date(2026, 4, 20));
        p.record_daily_completion(date(2026, 4, 21));
        p.record_daily_completion(date(2026, 4, 22));
        assert_eq!(p.daily_challenge_streak, 3);
        assert_eq!(p.daily_challenge_longest_streak, 3);

        // Gap resets the current streak — longest must NOT regress.
        p.record_daily_completion(date(2026, 4, 25));
        assert_eq!(p.daily_challenge_streak, 1);
        assert_eq!(
            p.daily_challenge_longest_streak, 3,
            "longest_streak must never regress after a gap"
        );

        // Two-day streak — still below longest, so longest stays at 3.
        p.record_daily_completion(date(2026, 4, 26));
        assert_eq!(p.daily_challenge_streak, 2);
        assert_eq!(p.daily_challenge_longest_streak, 3);
    }

    #[test]
    fn daily_challenge_history_is_capped_at_max() {
        // Push DAILY_CHALLENGE_HISTORY_CAP + 5 consecutive days; the
        // earliest five must be evicted and the most recent CAP retained.
        let mut p = PlayerProgress::default();
        let start = date(2024, 1, 1);
        let total = DAILY_CHALLENGE_HISTORY_CAP + 5;
        for offset in 0..total {
            p.record_daily_completion(start + Duration::days(offset as i64));
        }
        assert_eq!(p.daily_challenge_history.len(), DAILY_CHALLENGE_HISTORY_CAP);
        // Oldest retained is `start + 5` (we dropped the first 5).
        assert_eq!(
            p.daily_challenge_history.first().copied(),
            Some(start + Duration::days(5))
        );
        // Newest retained is the last date pushed.
        assert_eq!(
            p.daily_challenge_history.last().copied(),
            Some(start + Duration::days(total as i64 - 1))
        );
    }
}
