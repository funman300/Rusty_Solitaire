//! Persistent in-game HUD: score, move count, elapsed time, mode badge,
//! daily-challenge constraint, and undo count.
//!
//! The HUD spawns once at startup and lives for the app's lifetime. Text is
//! refreshed whenever `GameStateResource` changes (which happens on every move
//! and every elapsed-time tick), so score, moves, and timer all stay current
//! without a separate tick system.

use bevy::prelude::*;
use solitaire_core::game_state::{DrawMode, GameMode};

use crate::auto_complete_plugin::AutoCompleteState;
use crate::daily_challenge_plugin::DailyChallengeResource;
use crate::events::InfoToastEvent;
use crate::game_plugin::GameMutation;
use crate::resources::GameStateResource;
use crate::time_attack_plugin::TimeAttackResource;

/// Marker on the score text node.
#[derive(Component, Debug)]
pub struct HudScore;

/// Marker on the move-count text node.
#[derive(Component, Debug)]
pub struct HudMoves;

/// Marker on the elapsed-time text node.
#[derive(Component, Debug)]
pub struct HudTime;

/// Marker on the mode badge text node.
#[derive(Component, Debug)]
pub struct HudMode;

/// Marker on the daily-challenge constraint text node.
///
/// Displays the active goal (time limit or score target) when a daily challenge
/// is in progress. Empty string when no challenge is active or the game is won.
#[derive(Component, Debug)]
pub struct HudChallenge;

/// Marker on the undo-count text node.
///
/// Shows how many undos have been used this game. Displayed in amber when
/// `undo_count > 0` because using undo blocks the no-undo achievement.
#[derive(Component, Debug)]
pub struct HudUndos;

/// Marker on the auto-complete badge text node.
///
/// Displays `"AUTO"` in green while `AutoCompleteState.active` is true;
/// empty string otherwise.
#[derive(Component, Debug)]
pub struct HudAutoComplete;

/// HUD Z-layer — above cards (which start at z=0) but below overlay screens.
const Z_HUD: i32 = 50;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud)
            .add_systems(Update, update_hud.after(GameMutation))
            .add_systems(Update, announce_auto_complete.after(GameMutation));
    }
}

fn spawn_hud(mut commands: Commands) {
    let white = TextColor(Color::srgba(1.0, 1.0, 1.0, 0.80));
    let font = TextFont { font_size: 18.0, ..default() };
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(12.0),
                top: Val::Px(8.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(20.0),
                align_items: AlignItems::Center,
                ..default()
            },
            ZIndex(Z_HUD),
        ))
        .with_children(|b| {
            b.spawn((HudScore, Text::new("Score: 0"), font.clone(), white));
            b.spawn((HudMoves, Text::new("Moves: 0"), font.clone(), white));
            b.spawn((HudTime, Text::new("0:00"), font.clone(), white));
            b.spawn((
                HudMode,
                Text::new(""),
                TextFont { font_size: 17.0, ..default() },
                TextColor(Color::srgb(1.0, 0.85, 0.25)),
            ));
            // Daily-challenge constraint (hidden until a challenge is active).
            b.spawn((
                HudChallenge,
                Text::new(""),
                TextFont { font_size: 17.0, ..default() },
                TextColor(Color::srgb(0.4, 0.9, 1.0)),
            ));
            // Undo counter (white by default; turns amber when undos are used).
            b.spawn((
                HudUndos,
                Text::new(""),
                font,
                white,
            ));
            // Auto-complete badge (green "AUTO" when sequence is running).
            b.spawn((
                HudAutoComplete,
                Text::new(""),
                TextFont { font_size: 17.0, ..default() },
                TextColor(Color::srgb(0.2, 0.9, 0.3)),
            ));
        });
}

/// Formats a time-limit value in seconds as `"mm:ss"` for HUD display.
///
/// For example `format_time_limit(300)` returns `"5:00"`.
pub fn format_time_limit(secs: u64) -> String {
    let m = secs / 60;
    let s = secs % 60;
    format!("{m}:{s:02}")
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn update_hud(
    game: Res<GameStateResource>,
    time_attack: Option<Res<TimeAttackResource>>,
    daily: Option<Res<DailyChallengeResource>>,
    auto_complete: Option<Res<AutoCompleteState>>,
    mut score_q: Query<
        &mut Text,
        (
            With<HudScore>,
            Without<HudMoves>,
            Without<HudTime>,
            Without<HudMode>,
            Without<HudChallenge>,
            Without<HudUndos>,
            Without<HudAutoComplete>,
        ),
    >,
    mut moves_q: Query<
        &mut Text,
        (
            With<HudMoves>,
            Without<HudScore>,
            Without<HudTime>,
            Without<HudMode>,
            Without<HudChallenge>,
            Without<HudUndos>,
            Without<HudAutoComplete>,
        ),
    >,
    mut time_q: Query<
        &mut Text,
        (
            With<HudTime>,
            Without<HudScore>,
            Without<HudMoves>,
            Without<HudMode>,
            Without<HudChallenge>,
            Without<HudUndos>,
            Without<HudAutoComplete>,
        ),
    >,
    mut mode_q: Query<
        &mut Text,
        (
            With<HudMode>,
            Without<HudScore>,
            Without<HudMoves>,
            Without<HudTime>,
            Without<HudChallenge>,
            Without<HudUndos>,
            Without<HudAutoComplete>,
        ),
    >,
    mut challenge_q: Query<
        (&mut Text, &mut TextColor),
        (
            With<HudChallenge>,
            Without<HudScore>,
            Without<HudMoves>,
            Without<HudTime>,
            Without<HudMode>,
            Without<HudUndos>,
            Without<HudAutoComplete>,
        ),
    >,
    mut undos_q: Query<
        (&mut Text, &mut TextColor),
        (
            With<HudUndos>,
            Without<HudScore>,
            Without<HudMoves>,
            Without<HudTime>,
            Without<HudMode>,
            Without<HudChallenge>,
            Without<HudAutoComplete>,
        ),
    >,
    mut auto_q: Query<
        &mut Text,
        (
            With<HudAutoComplete>,
            Without<HudScore>,
            Without<HudMoves>,
            Without<HudTime>,
            Without<HudMode>,
            Without<HudChallenge>,
            Without<HudUndos>,
        ),
    >,
) {
    let ta_active = time_attack.as_ref().is_some_and(|ta| ta.active);

    // Score, moves, mode, challenge, and undos only need updating when game state changes.
    if game.is_changed() {
        let g = &game.0;
        let is_zen = g.mode == GameMode::Zen;
        if let Ok(mut t) = score_q.get_single_mut() {
            // Zen mode suppresses score display per spec ("No score display").
            **t = if is_zen {
                String::new()
            } else {
                format!("Score: {}", g.score)
            };
        }
        if let Ok(mut t) = moves_q.get_single_mut() {
            **t = format!("Moves: {}", g.move_count);
        }
        if let Ok(mut t) = mode_q.get_single_mut() {
            **t = match g.mode {
                GameMode::Classic => match g.draw_mode {
                    DrawMode::DrawOne => String::new(),
                    DrawMode::DrawThree => "Draw 3".to_string(),
                },
                GameMode::Zen => "ZEN".to_string(),
                GameMode::Challenge => "CHALLENGE".to_string(),
                GameMode::TimeAttack => "TIME ATTACK".to_string(),
            };
        }

        // --- Daily challenge constraint ---
        if let Ok((mut t, _)) = challenge_q.get_single_mut() {
            **t = if g.is_won {
                // Hide constraint once the game is over.
                String::new()
            } else if let Some(dc) = daily.as_deref() {
                challenge_hud_text(dc)
            } else {
                String::new()
            };
        }

        // --- Undo count ---
        if let Ok((mut t, mut color)) = undos_q.get_single_mut() {
            let count = g.undo_count;
            if count == 0 {
                **t = String::new();
                *color = TextColor(Color::srgba(1.0, 1.0, 1.0, 0.80));
            } else {
                **t = format!("Undos: {count}");
                // Amber warning: using undo blocks the no-undo achievement.
                *color = TextColor(Color::srgb(1.0, 0.7, 0.2));
            }
        }
    }

    // Time display: show Time Attack countdown every frame when active;
    // Zen mode suppresses the timer per spec ("No timer").
    // Otherwise show game elapsed time (updates once per second via game.is_changed()).
    let is_zen = game.0.mode == GameMode::Zen;
    let update_time = (ta_active || game.is_changed()) && !is_zen;
    if update_time {
        if let Ok(mut t) = time_q.get_single_mut() {
            if let Some(ta) = time_attack.as_ref().filter(|ta| ta.active) {
                let remaining = ta.remaining_secs.max(0.0) as u64;
                let m = remaining / 60;
                let s = remaining % 60;
                **t = format!("{m}:{s:02}");
            } else {
                let secs = game.0.elapsed_seconds;
                let m = secs / 60;
                let s = secs % 60;
                **t = format!("{m}:{s:02}");
            }
        }
    } else if is_zen && game.is_changed() {
        // Clear the time display when entering Zen mode.
        if let Ok(mut t) = time_q.get_single_mut() {
            **t = String::new();
        }
    }

    // --- Auto-complete badge ---
    // Reflects the AutoCompleteState resource; update whenever it changes or game changes.
    let ac_active = auto_complete.as_ref().is_some_and(|ac| ac.active);
    let ac_changed = auto_complete.as_ref().is_some_and(|ac| ac.is_changed());
    if ac_changed || game.is_changed() {
        if let Ok(mut t) = auto_q.get_single_mut() {
            **t = if ac_active {
                "AUTO".to_string()
            } else {
                String::new()
            };
        }
    }
}

/// Fires `InfoToastEvent("Auto-completing...")` exactly once each time
/// `AutoCompleteState` transitions from inactive to active. Uses a `Local<bool>`
/// to debounce so the toast only appears on the leading edge.
fn announce_auto_complete(
    auto_complete: Option<Res<AutoCompleteState>>,
    mut toast: EventWriter<InfoToastEvent>,
    mut was_active: Local<bool>,
) {
    let now_active = auto_complete.as_ref().is_some_and(|ac| ac.active);
    if now_active && !*was_active {
        toast.send(InfoToastEvent("Auto-completing...".to_string()));
    }
    *was_active = now_active;
}

/// Builds the HUD text for the active daily challenge constraints.
///
/// Returns `"Limit: mm:ss"` when a time limit is set, `"Goal: N pts"` when a
/// score target is set, or an empty string when the challenge has no extra
/// constraints.
fn challenge_hud_text(dc: &DailyChallengeResource) -> String {
    if let Some(secs) = dc.max_time_secs {
        format!("Limit: {}", format_time_limit(secs))
    } else if let Some(score) = dc.target_score {
        format!("Goal: {score} pts")
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_plugin::GamePlugin;
    use crate::table_plugin::TablePlugin;
    use chrono::Local;
    use solitaire_core::game_state::{DrawMode, GameState};

    fn headless_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(GamePlugin)
            .add_plugins(TablePlugin)
            .add_plugins(HudPlugin);
        app.update();
        app
    }

    #[test]
    fn hud_plugin_registers_without_panic() {
        let _app = headless_app();
    }

    #[test]
    fn update_hud_runs_after_game_mutation_without_panic() {
        let mut app = headless_app();
        app.world_mut().resource_mut::<GameStateResource>().0 =
            GameState::new(42, DrawMode::DrawOne);
        app.update();
    }

    fn read_hud_text<M: Component>(app: &mut App) -> String {
        app.world_mut()
            .query_filtered::<&Text, With<M>>()
            .iter(app.world())
            .next()
            .map(|t| t.0.clone())
            .unwrap_or_default()
    }

    #[test]
    fn score_reflects_game_state() {
        let mut app = headless_app();
        app.world_mut().resource_mut::<GameStateResource>().0.score = 750;
        app.update();
        assert_eq!(read_hud_text::<HudScore>(&mut app), "Score: 750");
    }

    #[test]
    fn moves_reflects_game_state() {
        let mut app = headless_app();
        app.world_mut().resource_mut::<GameStateResource>().0.move_count = 42;
        app.update();
        assert_eq!(read_hud_text::<HudMoves>(&mut app), "Moves: 42");
    }

    #[test]
    fn draw_three_mode_shows_draw_3_badge() {
        use solitaire_core::game_state::GameMode;
        let mut app = headless_app();
        app.world_mut().resource_mut::<GameStateResource>().0 =
            GameState::new_with_mode(42, DrawMode::DrawThree, GameMode::Classic);
        app.update();
        assert_eq!(read_hud_text::<HudMode>(&mut app), "Draw 3");
    }

    #[test]
    fn zen_mode_hides_score() {
        use solitaire_core::game_state::GameMode;
        let mut app = headless_app();
        app.world_mut().resource_mut::<GameStateResource>().0 =
            GameState::new_with_mode(42, DrawMode::DrawOne, GameMode::Zen);
        app.world_mut().resource_mut::<GameStateResource>().0.score = 999;
        app.update();
        // Zen mode spec: "No score display" → text must be empty.
        assert_eq!(read_hud_text::<HudScore>(&mut app), "");
    }

    #[test]
    fn time_display_uses_mm_ss_format() {
        let mut app = headless_app();
        app.world_mut().resource_mut::<GameStateResource>().0.elapsed_seconds = 125;
        app.update();
        // 125 seconds = 2 minutes 5 seconds → "2:05"
        assert_eq!(read_hud_text::<HudTime>(&mut app), "2:05");
    }

    // -----------------------------------------------------------------------
    // format_time_limit (pure function)
    // -----------------------------------------------------------------------

    #[test]
    fn format_time_limit_300_is_5_00() {
        assert_eq!(format_time_limit(300), "5:00");
    }

    #[test]
    fn format_time_limit_zero() {
        assert_eq!(format_time_limit(0), "0:00");
    }

    #[test]
    fn format_time_limit_pads_seconds() {
        assert_eq!(format_time_limit(65), "1:05");
    }

    // -----------------------------------------------------------------------
    // challenge_hud_text (pure function)
    // -----------------------------------------------------------------------

    #[test]
    fn challenge_hud_text_shows_time_limit() {
        let dc = DailyChallengeResource {
            date: Local::now().date_naive(),
            seed: 1,
            goal_description: None,
            target_score: None,
            max_time_secs: Some(300),
        };
        assert_eq!(challenge_hud_text(&dc), "Limit: 5:00");
    }

    #[test]
    fn challenge_hud_text_shows_score_goal() {
        let dc = DailyChallengeResource {
            date: Local::now().date_naive(),
            seed: 1,
            goal_description: None,
            target_score: Some(4000),
            max_time_secs: None,
        };
        assert_eq!(challenge_hud_text(&dc), "Goal: 4000 pts");
    }

    #[test]
    fn challenge_hud_text_empty_when_no_constraints() {
        let dc = DailyChallengeResource {
            date: Local::now().date_naive(),
            seed: 1,
            goal_description: None,
            target_score: None,
            max_time_secs: None,
        };
        assert_eq!(challenge_hud_text(&dc), "");
    }

    // -----------------------------------------------------------------------
    // HudChallenge in-app tests
    // -----------------------------------------------------------------------

    #[test]
    fn challenge_hud_empty_when_no_daily_resource() {
        // No DailyChallengeResource inserted → HudChallenge must be empty.
        let mut app = headless_app();
        app.world_mut().resource_mut::<GameStateResource>().0.score = 1; // force change
        app.update();
        assert_eq!(read_hud_text::<HudChallenge>(&mut app), "");
    }

    #[test]
    fn challenge_hud_shows_time_limit_when_resource_present() {
        let mut app = headless_app();
        app.world_mut().insert_resource(DailyChallengeResource {
            date: Local::now().date_naive(),
            seed: 42,
            goal_description: Some("Win fast".to_string()),
            target_score: None,
            max_time_secs: Some(300),
        });
        app.world_mut().resource_mut::<GameStateResource>().0.score = 1; // force change
        app.update();
        assert_eq!(read_hud_text::<HudChallenge>(&mut app), "Limit: 5:00");
    }

    #[test]
    fn challenge_hud_shows_score_goal_when_resource_present() {
        let mut app = headless_app();
        app.world_mut().insert_resource(DailyChallengeResource {
            date: Local::now().date_naive(),
            seed: 42,
            goal_description: None,
            target_score: Some(4000),
            max_time_secs: None,
        });
        app.world_mut().resource_mut::<GameStateResource>().0.score = 1;
        app.update();
        assert_eq!(read_hud_text::<HudChallenge>(&mut app), "Goal: 4000 pts");
    }

    #[test]
    fn challenge_hud_clears_on_win() {
        let mut app = headless_app();
        app.world_mut().insert_resource(DailyChallengeResource {
            date: Local::now().date_naive(),
            seed: 42,
            goal_description: None,
            target_score: None,
            max_time_secs: Some(300),
        });
        // Mark the game as won — HudChallenge should be empty.
        app.world_mut().resource_mut::<GameStateResource>().0.is_won = true;
        app.update();
        assert_eq!(read_hud_text::<HudChallenge>(&mut app), "");
    }

    // -----------------------------------------------------------------------
    // HudUndos in-app tests
    // -----------------------------------------------------------------------

    #[test]
    fn undos_hud_empty_at_game_start() {
        let mut app = headless_app();
        app.update();
        assert_eq!(read_hud_text::<HudUndos>(&mut app), "");
    }

    #[test]
    fn undos_hud_shows_count_after_undo() {
        let mut app = headless_app();
        app.world_mut().resource_mut::<GameStateResource>().0.undo_count = 3;
        app.update();
        assert_eq!(read_hud_text::<HudUndos>(&mut app), "Undos: 3");
    }

    // -----------------------------------------------------------------------
    // HudAutoComplete in-app tests (Task #56)
    // -----------------------------------------------------------------------

    fn headless_app_with_auto_complete() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(GamePlugin)
            .add_plugins(TablePlugin)
            .add_plugins(HudPlugin);
        app.init_resource::<AutoCompleteState>();
        app.update();
        app
    }

    #[test]
    fn auto_complete_badge_shows_auto_when_active() {
        let mut app = headless_app_with_auto_complete();
        app.world_mut().resource_mut::<AutoCompleteState>().active = true;
        // Also trigger game state change so the update fires.
        app.world_mut().resource_mut::<GameStateResource>().0.move_count += 1;
        app.update();
        assert_eq!(read_hud_text::<HudAutoComplete>(&mut app), "AUTO");
    }

    #[test]
    fn auto_complete_badge_empty_when_inactive() {
        let mut app = headless_app_with_auto_complete();
        // active is false by default.
        app.world_mut().resource_mut::<GameStateResource>().0.move_count += 1;
        app.update();
        assert_eq!(read_hud_text::<HudAutoComplete>(&mut app), "");
    }
}
