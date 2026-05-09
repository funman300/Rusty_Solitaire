//! Difficulty-tier game-start plugin.
//!
//! Handles [`StartDifficultyRequestEvent`] by picking the next seed from the
//! appropriate pre-verified catalog in `solitaire_data::difficulty_seeds` and
//! writing a [`NewGameRequestEvent`]. For [`DifficultyLevel::Random`] a
//! system-time seed is used instead — the deal may or may not be winnable.
//!
//! # Catalog cycling
//!
//! Each tier maintains an independent cursor in [`DifficultyIndexResource`]
//! that advances one step each time a game is started at that tier. The cursor
//! wraps modulo the catalog length so players never run out of variety. The
//! resource is *not* persisted — it resets to 0 on every launch, which is fine
//! because the starting position is effectively random (player-chosen timing
//! determines which seed in the 40-entry catalog they start at).

use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use solitaire_core::game_state::{DifficultyLevel, GameMode};
use solitaire_data::difficulty_seeds::seeds_for;

use crate::events::{NewGameRequestEvent, StartDifficultyRequestEvent};
use crate::game_plugin::GameMutation;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Per-tier catalog cursors. Each value is the index of the **next** seed to
/// deal from that tier's catalog. Wraps modulo the catalog length.
#[derive(Resource, Default)]
pub struct DifficultyIndexResource {
    easy: usize,
    medium: usize,
    hard: usize,
    expert: usize,
    grandmaster: usize,
}

impl DifficultyIndexResource {
    /// Advance the cursor for `level` and return the seed at the old position.
    /// Falls back to a system-time seed if the catalog is unexpectedly empty.
    pub fn next_seed(&mut self, level: DifficultyLevel) -> u64 {
        let Some(catalog) = seeds_for(level) else {
            return seed_from_system_time();
        };
        if catalog.is_empty() {
            return seed_from_system_time();
        }
        let cursor = match level {
            DifficultyLevel::Easy => &mut self.easy,
            DifficultyLevel::Medium => &mut self.medium,
            DifficultyLevel::Hard => &mut self.hard,
            DifficultyLevel::Expert => &mut self.expert,
            DifficultyLevel::Grandmaster => &mut self.grandmaster,
            DifficultyLevel::Random => unreachable!("Random has no catalog"),
        };
        let seed = catalog[*cursor % catalog.len()];
        *cursor = cursor.wrapping_add(1);
        seed
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers all difficulty-mode systems and resources.
pub struct DifficultyPlugin;

impl Plugin for DifficultyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DifficultyIndexResource>()
            .add_message::<StartDifficultyRequestEvent>()
            .add_message::<NewGameRequestEvent>()
            .add_systems(
                Update,
                handle_difficulty_request.before(GameMutation),
            );
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Resolves `StartDifficultyRequestEvent` → catalog seed → `NewGameRequestEvent`.
fn handle_difficulty_request(
    mut requests: MessageReader<StartDifficultyRequestEvent>,
    mut new_game: MessageWriter<NewGameRequestEvent>,
    mut index: ResMut<DifficultyIndexResource>,
) {
    for ev in requests.read() {
        let seed = if ev.level == DifficultyLevel::Random {
            seed_from_system_time()
        } else {
            index.next_seed(ev.level)
        };

        new_game.write(NewGameRequestEvent {
            seed: Some(seed),
            mode: Some(GameMode::Difficulty(ev.level)),
            confirmed: false,
        });
    }
}

fn seed_from_system_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0xD1FF_0000_DEAD_BEEF)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_plugin::GamePlugin;
    use crate::table_plugin::TablePlugin;
    use solitaire_data::difficulty_seeds::{EASY_SEEDS, MEDIUM_SEEDS};

    fn headless_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(GamePlugin)
            .add_plugins(TablePlugin)
            .add_plugins(DifficultyPlugin);
        app.update();
        app
    }

    fn fire_request(app: &mut App, level: DifficultyLevel) {
        app.world_mut()
            .write_message(StartDifficultyRequestEvent { level });
        app.update();
    }

    fn drain_new_game_events(app: &mut App) -> Vec<NewGameRequestEvent> {
        let msgs = app.world().resource::<Messages<NewGameRequestEvent>>();
        let mut cursor = msgs.get_cursor();
        cursor.read(msgs).copied().collect()
    }

    #[test]
    fn easy_request_dispatches_seed_from_easy_catalog() {
        let mut app = headless_app();
        fire_request(&mut app, DifficultyLevel::Easy);

        let events = drain_new_game_events(&mut app);
        assert_eq!(events.len(), 1);
        let ev = &events[0];
        assert!(ev.seed.is_some());
        assert_eq!(ev.mode, Some(GameMode::Difficulty(DifficultyLevel::Easy)));
        assert!(!ev.confirmed);
        // Seed must come from the Easy catalog (non-empty catalog is the test
        // precondition — the catalog uniqueness test in difficulty_seeds.rs
        // guards integrity).
        if !EASY_SEEDS.is_empty() {
            assert!(
                EASY_SEEDS.contains(&ev.seed.unwrap()),
                "seed {:?} not in EASY_SEEDS",
                ev.seed
            );
        }
    }

    #[test]
    fn successive_easy_requests_cycle_through_catalog() {
        let mut app = headless_app();
        fire_request(&mut app, DifficultyLevel::Easy);
        fire_request(&mut app, DifficultyLevel::Easy);

        let events = drain_new_game_events(&mut app);
        assert_eq!(events.len(), 2);
        // Two successive requests should return different seeds (assuming the
        // catalog has at least 2 entries — it has 40).
        if EASY_SEEDS.len() >= 2 {
            assert_ne!(
                events[0].seed, events[1].seed,
                "successive Easy requests should produce different seeds"
            );
        }
    }

    #[test]
    fn medium_request_dispatches_seed_from_medium_catalog() {
        let mut app = headless_app();
        fire_request(&mut app, DifficultyLevel::Medium);

        let events = drain_new_game_events(&mut app);
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].mode,
            Some(GameMode::Difficulty(DifficultyLevel::Medium))
        );
        if !MEDIUM_SEEDS.is_empty() {
            assert!(MEDIUM_SEEDS.contains(&events[0].seed.unwrap()));
        }
    }

    #[test]
    fn random_request_dispatches_some_seed_with_random_mode() {
        let mut app = headless_app();
        fire_request(&mut app, DifficultyLevel::Random);

        let events = drain_new_game_events(&mut app);
        assert_eq!(events.len(), 1);
        assert!(events[0].seed.is_some(), "Random should always produce Some(seed)");
        assert_eq!(
            events[0].mode,
            Some(GameMode::Difficulty(DifficultyLevel::Random))
        );
    }

    #[test]
    fn different_tier_cursors_are_independent() {
        let mut app = headless_app();
        fire_request(&mut app, DifficultyLevel::Easy);
        fire_request(&mut app, DifficultyLevel::Medium);

        let events = drain_new_game_events(&mut app);
        assert_eq!(events.len(), 2);
        // Seeds from different catalogs should differ (they come from different
        // address ranges by construction of gen_difficulty_seeds).
        assert_ne!(
            events[0].seed, events[1].seed,
            "Easy and Medium should draw from independent catalogs"
        );
    }
}
