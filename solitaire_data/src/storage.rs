//! Atomic file I/O for `StatsSnapshot` persistence.
//!
//! All saves go through `filename.json.tmp` → `rename()` so a crash or power
//! loss during a write never corrupts the saved data.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::stats::StatsSnapshot;

const APP_DIR_NAME: &str = "solitaire_quest";
const STATS_FILE_NAME: &str = "stats.json";

/// Returns the platform-specific path to `stats.json`, or `None` if
/// `dirs::data_dir()` is unavailable (e.g. minimal Linux containers).
pub fn stats_file_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join(APP_DIR_NAME).join(STATS_FILE_NAME))
}

/// Load stats from an explicit path. Returns `StatsSnapshot::default()` if
/// the file is missing or cannot be deserialized (corrupt/truncated).
pub fn load_stats_from(path: &Path) -> StatsSnapshot {
    let Ok(data) = fs::read(path) else {
        return StatsSnapshot::default();
    };
    serde_json::from_slice(&data).unwrap_or_default()
}

/// Save stats to an explicit path using an atomic write (`.tmp` → rename).
pub fn save_stats_to(path: &Path, stats: &StatsSnapshot) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(stats).map_err(io::Error::other)?;

    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json.as_bytes())?;
    fs::rename(&tmp, path)?;
    Ok(())
}

/// Load stats from the platform default path. Returns default if the path
/// is unavailable or the file is missing/corrupt.
pub fn load_stats() -> StatsSnapshot {
    stats_file_path()
        .map(|p| load_stats_from(&p))
        .unwrap_or_default()
}

/// Save stats to the platform default path. Returns an error if the platform
/// data dir is unavailable or the write fails.
pub fn save_stats(stats: &StatsSnapshot) -> io::Result<()> {
    let path = stats_file_path().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "platform data dir unavailable")
    })?;
    save_stats_to(&path, stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::StatsSnapshot;
    use solitaire_core::game_state::DrawMode;
    use std::env;

    fn tmp_path(name: &str) -> PathBuf {
        env::temp_dir().join(format!("solitaire_test_{name}.json"))
    }

    #[test]
    fn round_trip_save_and_load() {
        let path = tmp_path("round_trip");
        let _ = fs::remove_file(&path);

        let mut stats = StatsSnapshot::default();
        stats.update_on_win(1000, 180, &DrawMode::DrawOne);
        save_stats_to(&path, &stats).expect("save");

        let loaded = load_stats_from(&path);
        assert_eq!(loaded.games_won, 1);
        assert_eq!(loaded.best_single_score, 1000);
        assert_eq!(loaded.fastest_win_seconds, 180);
    }

    #[test]
    fn load_from_missing_file_returns_default() {
        let path = tmp_path("missing_file_abc123");
        let _ = fs::remove_file(&path);
        let stats = load_stats_from(&path);
        assert_eq!(stats, StatsSnapshot::default());
    }

    #[test]
    fn save_is_atomic_no_half_written_file() {
        let path = tmp_path("atomic_write");
        let stats = StatsSnapshot::default();
        save_stats_to(&path, &stats).expect("save");

        let tmp = path.with_extension("json.tmp");
        assert!(!tmp.exists(), ".tmp file must be cleaned up after rename");
    }

    #[test]
    fn load_from_corrupt_file_returns_default() {
        let path = tmp_path("corrupt");
        fs::write(&path, b"not valid json!!!").expect("write corrupt");
        let stats = load_stats_from(&path);
        assert_eq!(stats, StatsSnapshot::default());
    }
}
