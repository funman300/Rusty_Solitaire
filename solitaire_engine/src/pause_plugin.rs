//! Pause overlay (Esc).
//!
//! While paused:
//! - The `PausedResource` flag is true.
//! - Elapsed-time and Time Attack tickers stop counting (they read this
//!   resource and bail out early).
//!
//! Pressing Esc again dismisses the overlay and resumes ticking. Other
//! input (drag, keyboard hotkeys) is **not** blocked — pause is purely a
//! "stop the clock" screen for now. A future polish slice can layer
//! input-blocking on top if desired.

use bevy::prelude::*;
use solitaire_data::save_game_state_to;

use crate::game_plugin::GameStatePath;
use crate::progress_plugin::ProgressResource;
use crate::resources::GameStateResource;
use crate::stats_plugin::StatsResource;

/// Toggleable flag read by `tick_elapsed_time` and `advance_time_attack`.
#[derive(Resource, Debug, Default)]
pub struct PausedResource(pub bool);

/// Marker on the pause overlay root node.
#[derive(Component, Debug)]
pub struct PauseScreen;

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PausedResource>()
            .add_systems(Update, toggle_pause);
    }
}

#[allow(clippy::too_many_arguments)]
fn toggle_pause(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut paused: ResMut<PausedResource>,
    screens: Query<Entity, With<PauseScreen>>,
    game: Option<Res<GameStateResource>>,
    path: Option<Res<GameStatePath>>,
    progress: Option<Res<ProgressResource>>,
    stats: Option<Res<StatsResource>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    if let Ok(entity) = screens.get_single() {
        commands.entity(entity).despawn_recursive();
        paused.0 = false;
    } else {
        // Snapshot current level and streak at pause time.
        let level = progress.as_deref().map(|p| p.0.level);
        let streak = stats.as_deref().map(|s| s.0.win_streak_current);
        spawn_pause_screen(&mut commands, level, streak);
        paused.0 = true;
        // Persist the current game state whenever the player opens the pause
        // overlay so an OS-level kill still leaves a resumable save.
        if let (Some(g), Some(p)) = (game, path) {
            if let Some(disk_path) = p.0.as_deref() {
                if let Err(e) = save_game_state_to(disk_path, &g.0) {
                    warn!("game_state: failed to save on pause: {e}");
                }
            }
        }
    }
}

/// Spawns the full-screen pause overlay.
///
/// `level` and `streak` are optional snapshots taken at pause time. When
/// `ProgressResource` or `StatsResource` is not installed (e.g. in headless
/// tests), those lines are omitted from the overlay.
fn spawn_pause_screen(commands: &mut Commands, level: Option<u32>, streak: Option<u32>) {
    commands
        .spawn((
            PauseScreen,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(0.0),
                top: Val::Percent(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.82)),
            ZIndex(220),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new("Paused"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.87, 0.0)),
            ));
            // Level and streak line — only shown when the resources are present.
            if level.is_some() || streak.is_some() {
                let info = build_level_streak_line(level, streak);
                b.spawn((
                    Text::new(info),
                    TextFont {
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.75, 0.95, 0.75)),
                ));
            }
            b.spawn((
                Text::new("Press Esc to resume"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.85, 0.80)),
            ));
        });
}

/// Formats the level / win-streak summary line for the pause overlay.
///
/// Both values are optional because either resource may be absent in
/// headless or partially-configured app contexts.
fn build_level_streak_line(level: Option<u32>, streak: Option<u32>) -> String {
    match (level, streak) {
        (Some(l), Some(s)) => format!("Level {l}   Win streak: {s}"),
        (Some(l), None) => format!("Level {l}"),
        (None, Some(s)) => format!("Win streak: {s}"),
        (None, None) => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headless_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(PausePlugin);
        app.init_resource::<ButtonInput<KeyCode>>();
        app.update();
        app
    }

    fn press_esc(app: &mut App) {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.release(KeyCode::Escape);
        input.clear();
        input.press(KeyCode::Escape);
    }

    #[test]
    fn pressing_esc_pauses() {
        let mut app = headless_app();
        press_esc(&mut app);
        app.update();
        assert!(app.world().resource::<PausedResource>().0);
        assert_eq!(
            app.world_mut()
                .query::<&PauseScreen>()
                .iter(app.world())
                .count(),
            1
        );
    }

    #[test]
    fn pressing_esc_twice_resumes() {
        let mut app = headless_app();
        press_esc(&mut app);
        app.update();
        press_esc(&mut app);
        app.update();
        assert!(!app.world().resource::<PausedResource>().0);
        assert_eq!(
            app.world_mut()
                .query::<&PauseScreen>()
                .iter(app.world())
                .count(),
            0
        );
    }

    #[test]
    fn toggle_is_symmetric_for_multiple_cycles() {
        let mut app = headless_app();
        // Third press re-pauses after resume.
        press_esc(&mut app);
        app.update();
        press_esc(&mut app);
        app.update();
        press_esc(&mut app);
        app.update();
        assert!(
            app.world().resource::<PausedResource>().0,
            "third Esc must re-pause"
        );
        assert_eq!(
            app.world_mut()
                .query::<&PauseScreen>()
                .iter(app.world())
                .count(),
            1,
            "third Esc must re-spawn PauseScreen"
        );
    }

    // -----------------------------------------------------------------------
    // build_level_streak_line (pure function)
    // -----------------------------------------------------------------------

    #[test]
    fn level_streak_both_present() {
        assert_eq!(
            build_level_streak_line(Some(7), Some(3)),
            "Level 7   Win streak: 3"
        );
    }

    #[test]
    fn level_streak_only_level() {
        assert_eq!(build_level_streak_line(Some(5), None), "Level 5");
    }

    #[test]
    fn level_streak_only_streak() {
        assert_eq!(build_level_streak_line(None, Some(4)), "Win streak: 4");
    }

    #[test]
    fn level_streak_neither() {
        assert_eq!(build_level_streak_line(None, None), "");
    }

    // -----------------------------------------------------------------------
    // Pause screen with progress / stats resources present
    // -----------------------------------------------------------------------

    #[test]
    fn pause_screen_spawns_with_level_and_streak_when_resources_present() {
        use crate::progress_plugin::{ProgressPlugin, ProgressResource};
        use crate::stats_plugin::{StatsPlugin, StatsResource};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(crate::game_plugin::GamePlugin)
            .add_plugins(crate::table_plugin::TablePlugin)
            .add_plugins(ProgressPlugin::headless())
            .add_plugins(StatsPlugin::headless())
            .add_plugins(PausePlugin);
        app.init_resource::<ButtonInput<KeyCode>>();
        app.update();

        // Set known values.
        app.world_mut().resource_mut::<ProgressResource>().0.level = 7;
        app.world_mut().resource_mut::<StatsResource>().0.win_streak_current = 3;

        press_esc(&mut app);
        app.update();

        // Verify the screen was spawned.
        assert!(app.world().resource::<PausedResource>().0);

        // Find the text nodes on the PauseScreen children and check one contains
        // the expected level/streak string.
        let texts: Vec<String> = app
            .world_mut()
            .query::<&Text>()
            .iter(app.world())
            .map(|t| t.0.clone())
            .collect();
        assert!(
            texts.iter().any(|t| t == "Level 7   Win streak: 3"),
            "expected level/streak line in pause screen texts, got: {texts:?}"
        );
    }
}
