//! Toggleable full-window profile overlay (press **P**).
//!
//! Shows the player's sync account, progression, achievements, and a statistics
//! summary in a single scrollable panel. Spawned on the first `P` keypress and
//! despawned on the second.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use solitaire_core::achievement::achievement_by_id;
use solitaire_data::SyncBackend;

use crate::achievement_plugin::AchievementsResource;
use crate::events::ToggleProfileRequestEvent;
use crate::font_plugin::FontResource;
use crate::progress_plugin::ProgressResource;
use crate::resources::{SyncStatus, SyncStatusResource};
use crate::settings_plugin::SettingsResource;
use crate::stats_plugin::{format_fastest_win, format_win_rate, StatsResource};
use crate::ui_modal::{
    spawn_modal, spawn_modal_actions, spawn_modal_button, spawn_modal_header, ButtonVariant,
};
use crate::ui_theme::{
    ACCENT_PRIMARY, STATE_INFO, STATE_SUCCESS, TEXT_PRIMARY, TEXT_SECONDARY, TYPE_BODY,
    TYPE_BODY_LG, VAL_SPACE_2, Z_MODAL_PANEL,
};

/// Marker component on the profile overlay root node.
#[derive(Component, Debug)]
pub struct ProfileScreen;

/// Registers the `P` key toggle for the profile overlay.
pub struct ProfilePlugin;

/// Marker on the "Done" button inside the Profile modal.
#[derive(Component, Debug)]
pub struct ProfileCloseButton;

impl Plugin for ProfilePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ToggleProfileRequestEvent>()
            .add_systems(Update, (toggle_profile_screen, handle_profile_close_button));
    }
}

fn handle_profile_close_button(
    mut commands: Commands,
    close_buttons: Query<&Interaction, (With<ProfileCloseButton>, Changed<Interaction>)>,
    screens: Query<Entity, With<ProfileScreen>>,
) {
    if !close_buttons.iter().any(|i| *i == Interaction::Pressed) {
        return;
    }
    for entity in &screens {
        commands.entity(entity).despawn();
    }
}

#[allow(clippy::too_many_arguments)]
fn toggle_profile_screen(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut requests: MessageReader<ToggleProfileRequestEvent>,
    settings: Option<Res<SettingsResource>>,
    sync_status: Option<Res<SyncStatusResource>>,
    progress: Option<Res<ProgressResource>>,
    achievements: Option<Res<AchievementsResource>>,
    stats: Option<Res<StatsResource>>,
    font_res: Option<Res<FontResource>>,
    screens: Query<Entity, With<ProfileScreen>>,
) {
    let button_clicked = requests.read().count() > 0;
    if !keys.just_pressed(KeyCode::KeyP) && !button_clicked {
        return;
    }
    if let Ok(entity) = screens.single() {
        commands.entity(entity).despawn();
    } else {
        spawn_profile_screen(
            &mut commands,
            settings.as_deref(),
            sync_status.as_deref(),
            progress.as_deref(),
            achievements.as_deref(),
            stats.as_deref(),
            font_res.as_deref(),
        );
    }
}

fn spawn_profile_screen(
    commands: &mut Commands,
    settings: Option<&SettingsResource>,
    sync_status: Option<&SyncStatusResource>,
    progress: Option<&ProgressResource>,
    achievements: Option<&AchievementsResource>,
    stats: Option<&StatsResource>,
    font_res: Option<&FontResource>,
) {
    let font_handle = font_res.map(|f| f.0.clone()).unwrap_or_default();
    let font_section = TextFont {
        font: font_handle.clone(),
        font_size: TYPE_BODY_LG,
        ..default()
    };
    let font_row = TextFont {
        font: font_handle,
        font_size: TYPE_BODY,
        ..default()
    };

    spawn_modal(commands, ProfileScreen, Z_MODAL_PANEL, |card| {
        spawn_modal_header(card, "Profile", font_res);

        // ── Sync section ────────────────────────────────────────────
        card.spawn((
            Text::new("Sync"),
            font_section.clone(),
            TextColor(STATE_INFO),
        ));
        if let Some(s) = settings {
            let (backend_name, username) = sync_info(&s.0.sync_backend);
            card.spawn((
                Text::new(format!("Account: {username}  |  Backend: {backend_name}")),
                font_row.clone(),
                TextColor(TEXT_PRIMARY),
            ));
        }
        if let Some(ss) = sync_status {
            let status_text = match &ss.0 {
                SyncStatus::Idle => "Sync: idle".to_string(),
                SyncStatus::Syncing => "Sync: syncing\u{2026}".to_string(),
                SyncStatus::LastSynced(dt) => {
                    format!("Last synced: {}", dt.format("%Y-%m-%d %H:%M"))
                }
                SyncStatus::Error(e) => format!("Sync error: {e}"),
            };
            card.spawn((
                Text::new(status_text),
                font_row.clone(),
                TextColor(TEXT_SECONDARY),
            ));
        }

        // ── Progression section ─────────────────────────────────────
        spawn_spacer(card, VAL_SPACE_2);
        card.spawn((
            Text::new("Progression"),
            font_section.clone(),
            TextColor(STATE_INFO),
        ));
        if let Some(p) = progress {
            let prog = &p.0;
            let (xp_span, xp_done) = xp_progress(prog.total_xp, prog.level);
            let pct = if xp_span == 0 {
                100u64
            } else {
                xp_done.saturating_mul(100).checked_div(xp_span).unwrap_or(100)
            };
            card.spawn((
                Text::new(format!(
                    "Level {}  \u{2014}  {} XP  ({}/{} to next, {}%)",
                    prog.level, prog.total_xp, xp_done, xp_span, pct
                )),
                font_row.clone(),
                TextColor(TEXT_PRIMARY),
            ));
            card.spawn((
                Text::new(format!(
                    "Daily streak: {}  |  Card backs: {}  |  Backgrounds: {}",
                    prog.daily_challenge_streak,
                    prog.unlocked_card_backs.len(),
                    prog.unlocked_backgrounds.len(),
                )),
                font_row.clone(),
                TextColor(TEXT_PRIMARY),
            ));
        }

        // ── Achievements section ────────────────────────────────────
        spawn_spacer(card, VAL_SPACE_2);
        card.spawn((
            Text::new("Achievements"),
            font_section.clone(),
            TextColor(STATE_INFO),
        ));
        if let Some(ar) = achievements {
            let records = &ar.0;
            let unlocked_count = records.iter().filter(|r| r.unlocked).count();
            card.spawn((
                Text::new(format!("{} / 18 unlocked", unlocked_count)),
                font_row.clone(),
                TextColor(ACCENT_PRIMARY),
            ));

            let mut any_unlocked = false;
            for record in records {
                let def = achievement_by_id(record.id.as_str());
                let is_secret = def.map(|d| d.secret).unwrap_or(false);
                if is_secret && !record.unlocked {
                    continue;
                }
                if !record.unlocked {
                    continue;
                }
                any_unlocked = true;
                let name = def.map(|d| d.name).unwrap_or(record.id.as_str());
                let date_str = match record.unlock_date {
                    Some(dt) => format!("  ({})", dt.format("%Y-%m-%d")),
                    None => String::new(),
                };
                card.spawn((
                    Text::new(format!("  [x] {name}{date_str}")),
                    font_row.clone(),
                    TextColor(STATE_SUCCESS),
                ));
            }
            if !any_unlocked {
                card.spawn((
                    Text::new("  No achievements unlocked yet."),
                    font_row.clone(),
                    TextColor(TEXT_SECONDARY),
                ));
            }
        }

        // ── Statistics summary section ──────────────────────────────
        spawn_spacer(card, VAL_SPACE_2);
        card.spawn((
            Text::new("Statistics Summary"),
            font_section.clone(),
            TextColor(STATE_INFO),
        ));
        if let Some(sr) = stats {
            let s = &sr.0;
            let best_score_str = if s.best_single_score == 0 {
                "\u{2014}".to_string()
            } else {
                s.best_single_score.to_string()
            };
            card.spawn((
                Text::new(format!(
                    "Played: {}  |  Won: {}  |  Win rate: {}  |  Best time: {}",
                    s.games_played,
                    s.games_won,
                    format_win_rate(s),
                    format_fastest_win(s.fastest_win_seconds),
                )),
                font_row.clone(),
                TextColor(TEXT_PRIMARY),
            ));
            card.spawn((
                Text::new(format!(
                    "Win streak: {} current, {} best  |  Best score: {}",
                    s.win_streak_current, s.win_streak_best, best_score_str,
                )),
                font_row.clone(),
                TextColor(TEXT_PRIMARY),
            ));
        }

        spawn_modal_actions(card, |actions| {
            spawn_modal_button(
                actions,
                ProfileCloseButton,
                "Done",
                Some("P"),
                ButtonVariant::Primary,
                font_res,
            );
        });
    });
}

/// Spawn a fixed-height vertical spacer node.
fn spawn_spacer(parent: &mut ChildSpawnerCommands, height: Val) {
    parent.spawn(Node {
        height,
        ..default()
    });
}

/// Return `(backend_name, username_display)` for the given sync backend.
fn sync_info(backend: &SyncBackend) -> (&'static str, String) {
    match backend {
        SyncBackend::Local => ("Local", "—".to_string()),
        SyncBackend::SolitaireServer { username, .. } => {
            ("Solitaire Server", username.clone())
        }
    }
}

/// Return `(xp_span_for_level, xp_done_in_level)` for the given `total_xp` and `level`.
///
/// Levels 1–10 each require 500 XP; levels 11+ each require 1 000 XP.
fn xp_progress(total_xp: u64, level: u32) -> (u64, u64) {
    let level_start = if level < 10 {
        level as u64 * 500
    } else {
        5_000 + (level as u64 - 10) * 1_000
    };
    let xp_span: u64 = if level < 10 { 500 } else { 1_000 };
    let xp_done = total_xp.saturating_sub(level_start).min(xp_span);
    (xp_span, xp_done)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::achievement_plugin::AchievementPlugin;
    use crate::game_plugin::GamePlugin;
    use crate::progress_plugin::ProgressPlugin;
    use crate::settings_plugin::SettingsPlugin;
    use crate::stats_plugin::StatsPlugin;
    use crate::table_plugin::TablePlugin;

    fn headless_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(GamePlugin)
            .add_plugins(TablePlugin)
            .add_plugins(StatsPlugin::headless())
            .add_plugins(ProgressPlugin::headless())
            .add_plugins(AchievementPlugin::headless())
            .add_plugins(SettingsPlugin::headless())
            .add_plugins(ProfilePlugin);
        app.init_resource::<ButtonInput<KeyCode>>();
        app.update();
        app
    }

    #[test]
    fn pressing_p_spawns_profile_screen() {
        let mut app = headless_app();
        assert_eq!(
            app.world_mut()
                .query::<&ProfileScreen>()
                .iter(app.world())
                .count(),
            0
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyP);
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&ProfileScreen>()
                .iter(app.world())
                .count(),
            1
        );
    }

    #[test]
    fn pressing_p_twice_closes_profile_screen() {
        let mut app = headless_app();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyP);
        app.update();

        {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.release(KeyCode::KeyP);
            input.clear();
            input.press(KeyCode::KeyP);
        }
        app.update();

        assert_eq!(
            app.world_mut()
                .query::<&ProfileScreen>()
                .iter(app.world())
                .count(),
            0
        );
    }

    #[test]
    fn xp_progress_at_zero() {
        assert_eq!(xp_progress(0, 0), (500, 0));
    }

    #[test]
    fn xp_progress_halfway_through_level_1() {
        // Level 1 starts at 500 XP; span is 500.  At 750 XP: done = 250.
        assert_eq!(xp_progress(750, 1), (500, 250));
    }

    #[test]
    fn xp_progress_at_level_10() {
        // Level 10 is the first post-table level (span = 1000, starts at 5000).
        assert_eq!(xp_progress(5_000, 10), (1_000, 0));
    }
}
