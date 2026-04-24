//! Persistence for per-player achievement unlock records.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const APP_DIR_NAME: &str = "solitaire_quest";
const FILE_NAME: &str = "achievements.json";

/// One player's unlock state for a single achievement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AchievementRecord {
    pub id: String,
    pub unlocked: bool,
    pub unlock_date: Option<DateTime<Utc>>,
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

    /// Mark this record unlocked at the given timestamp. No-op if already unlocked
    /// (preserves earliest `unlock_date`).
    pub fn unlock(&mut self, at: DateTime<Utc>) {
        if self.unlocked {
            return;
        }
        self.unlocked = true;
        self.unlock_date = Some(at);
    }
}

/// Platform-specific default path for `achievements.json`.
pub fn achievements_file_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join(APP_DIR_NAME).join(FILE_NAME))
}

/// Load achievements from an explicit path. Returns `Vec::new()` if the file
/// is missing or unreadable.
pub fn load_achievements_from(path: &Path) -> Vec<AchievementRecord> {
    let Ok(data) = fs::read(path) else {
        return Vec::new();
    };
    serde_json::from_slice(&data).unwrap_or_default()
}

/// Save achievements to an explicit path using an atomic write.
pub fn save_achievements_to(path: &Path, records: &[AchievementRecord]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(records).map_err(io::Error::other)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json.as_bytes())?;
    fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn tmp_path(name: &str) -> PathBuf {
        env::temp_dir().join(format!("solitaire_ach_test_{name}.json"))
    }

    #[test]
    fn unlock_sets_flag_and_date() {
        let mut r = AchievementRecord::locked("x");
        let at = Utc::now();
        r.unlock(at);
        assert!(r.unlocked);
        assert_eq!(r.unlock_date, Some(at));
    }

    #[test]
    fn unlock_is_idempotent_on_date() {
        let mut r = AchievementRecord::locked("x");
        let first = Utc::now();
        r.unlock(first);
        let later = first + chrono::Duration::hours(1);
        r.unlock(later);
        assert_eq!(r.unlock_date, Some(first), "earliest date preserved");
    }

    #[test]
    fn round_trip_save_and_load() {
        let path = tmp_path("round_trip");
        let _ = fs::remove_file(&path);

        let records = vec![
            AchievementRecord::locked("first_win"),
            {
                let mut r = AchievementRecord::locked("century");
                r.unlock(Utc::now());
                r
            },
        ];
        save_achievements_to(&path, &records).expect("save");
        let loaded = load_achievements_from(&path);
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[1].id, "century");
        assert!(loaded[1].unlocked);
    }

    #[test]
    fn load_from_missing_file_returns_empty() {
        let path = tmp_path("missing_abc");
        let _ = fs::remove_file(&path);
        assert!(load_achievements_from(&path).is_empty());
    }

    #[test]
    fn load_from_corrupt_file_returns_empty() {
        let path = tmp_path("corrupt");
        fs::write(&path, b"not json").expect("write");
        assert!(load_achievements_from(&path).is_empty());
    }

    #[test]
    fn save_cleans_up_tmp_file() {
        let path = tmp_path("atomic");
        save_achievements_to(&path, &[]).expect("save");
        let tmp = path.with_extension("json.tmp");
        assert!(!tmp.exists());
    }
}
