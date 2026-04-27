//! Shared `AchievementRecord` definition — used by both the game client and
//! the sync server.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// One player's unlock state for a single achievement.
///
/// The achievement *definition* (name, description, condition fn) lives in
/// `solitaire_core`. This record only tracks runtime unlock state and is
/// what gets persisted and synced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AchievementRecord {
    /// Matches the `id` field of the corresponding `AchievementDef` in
    /// `solitaire_core`.
    pub id: String,
    /// Whether the achievement has been unlocked.
    pub unlocked: bool,
    /// The UTC timestamp at which the achievement was first unlocked.
    /// `None` when not yet unlocked.
    pub unlock_date: Option<DateTime<Utc>>,
    /// Whether the unlock reward (XP, cosmetic, etc.) has been granted.
    pub reward_granted: bool,
}

impl AchievementRecord {
    /// Construct an initial record for an achievement that is not yet unlocked.
    pub fn locked(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            unlocked: false,
            unlock_date: None,
            reward_granted: false,
        }
    }

    /// Mark this record unlocked at the given timestamp.
    ///
    /// No-op if already unlocked — preserves the earliest `unlock_date` so
    /// that merging two unlock records always keeps the older timestamp.
    pub fn unlock(&mut self, at: DateTime<Utc>) {
        if self.unlocked {
            return;
        }
        self.unlocked = true;
        self.unlock_date = Some(at);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locked_creates_an_unlocked_record() {
        let r = AchievementRecord::locked("first_win");
        assert_eq!(r.id, "first_win");
        assert!(!r.unlocked);
        assert!(r.unlock_date.is_none());
        assert!(!r.reward_granted);
    }

    #[test]
    fn unlock_sets_unlocked_and_stores_timestamp() {
        let mut r = AchievementRecord::locked("first_win");
        let ts = Utc::now();
        r.unlock(ts);
        assert!(r.unlocked);
        assert_eq!(r.unlock_date, Some(ts));
    }

    #[test]
    fn unlock_is_idempotent_and_preserves_earliest_date() {
        let mut r = AchievementRecord::locked("first_win");
        let early = DateTime::UNIX_EPOCH;
        let later = Utc::now();
        r.unlock(early);
        r.unlock(later); // should be a no-op
        assert_eq!(r.unlock_date, Some(early), "earliest unlock date must be preserved");
    }
}
