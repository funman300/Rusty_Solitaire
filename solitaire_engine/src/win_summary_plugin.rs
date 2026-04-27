//! Win summary modal overlay and screen-shake effect.
//!
//! # Task #33 — Win summary screen
//! On `GameWonEvent`, after a 0.5 s delay (so the cascade animation has
//! started), a full-screen modal is spawned showing score, time, XP, and a
//! "Play Again" button that fires `NewGameRequestEvent` and closes the modal.
//!
//! # Task #47 — Win fanfare screen-shake
//! When `GameWonEvent` fires, `ScreenShakeResource` is set. A system offsets
//! the `Camera2d` `Transform` each frame with a decaying oscillation until the
//! shake duration elapses.

use bevy::prelude::*;

use crate::events::{GameWonEvent, NewGameRequestEvent, XpAwardedEvent};
use crate::game_plugin::GameMutation;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Delay after `GameWonEvent` before the win-summary modal is spawned.
/// Chosen so the cascade animation has a moment to start first.
const WIN_SUMMARY_DELAY_SECS: f32 = 0.5;

/// Duration of the screen-shake in seconds.
const SHAKE_DURATION_SECS: f32 = 0.6;
/// Maximum camera displacement in world-space pixels at the start of the shake.
const SHAKE_INTENSITY: f32 = 8.0;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Accumulates win data while waiting for `XpAwardedEvent` to arrive.
///
/// The XP event fires shortly after `GameWonEvent`. We store both pieces of
/// data here so the modal can show the complete picture.
#[derive(Resource, Debug, Clone, Default)]
pub struct WinSummaryPending {
    /// Score from the most recent `GameWonEvent`.
    pub score: i32,
    /// Elapsed game time (seconds) from the most recent `GameWonEvent`.
    pub time_seconds: u64,
    /// XP awarded from the most recent `XpAwardedEvent` (0 until that event fires).
    pub xp: u64,
}

/// Drives the camera shake effect after a win.
///
/// While `remaining > 0` a system applies a decaying sinusoidal offset to the
/// main camera's `Transform`.  The system resets the camera to the origin when
/// `remaining` reaches zero.
#[derive(Resource, Debug, Clone, Default)]
pub struct ScreenShakeResource {
    /// Seconds of shake remaining.
    pub remaining: f32,
    /// Peak displacement in world-space pixels (decays to zero over `remaining`).
    pub intensity: f32,
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker on the win-summary modal root entity.
#[derive(Component, Debug)]
pub struct WinSummaryOverlay;

/// Marker on the "Play Again" button inside the win-summary modal.
#[derive(Component, Debug)]
enum WinSummaryButton {
    PlayAgain,
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers the win-summary modal and screen-shake systems.
pub struct WinSummaryPlugin;

impl Plugin for WinSummaryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WinSummaryPending>()
            .init_resource::<ScreenShakeResource>()
            .add_event::<GameWonEvent>()
            .add_event::<XpAwardedEvent>()
            .add_event::<NewGameRequestEvent>()
            .add_systems(
                Update,
                (
                    cache_win_data,
                    spawn_win_summary_after_delay,
                    handle_win_summary_buttons,
                    apply_screen_shake,
                )
                    .after(GameMutation),
            );
    }
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// Formats `seconds` as `m:ss`.
///
/// ```
/// # use solitaire_engine::win_summary_plugin::format_win_time;
/// assert_eq!(format_win_time(0),   "0:00");
/// assert_eq!(format_win_time(65),  "1:05");
/// assert_eq!(format_win_time(3661), "61:01");
/// ```
pub fn format_win_time(seconds: u64) -> String {
    let m = seconds / 60;
    let s = seconds % 60;
    format!("{m}:{s:02}")
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Caches score/time from `GameWonEvent` and XP from `XpAwardedEvent` into
/// `WinSummaryPending` so they are available when the modal spawns.
fn cache_win_data(
    mut won: EventReader<GameWonEvent>,
    mut xp: EventReader<XpAwardedEvent>,
    mut pending: ResMut<WinSummaryPending>,
) {
    for ev in won.read() {
        pending.score = ev.score;
        pending.time_seconds = ev.time_seconds;
        pending.xp = 0; // reset; XP event follows
    }
    for ev in xp.read() {
        pending.xp = ev.amount;
    }
}

/// After `GameWonEvent`, arms the screen-shake resource.
///
/// This system shares the `GameWonEvent` stream with `cache_win_data` through
/// the delay timer stored in `Local` — the shake fires immediately, while the
/// modal waits 0.5 s.
fn spawn_win_summary_after_delay(
    mut commands: Commands,
    mut won: EventReader<GameWonEvent>,
    mut shake: ResMut<ScreenShakeResource>,
    pending: Res<WinSummaryPending>,
    time: Res<Time>,
    overlays: Query<Entity, With<WinSummaryOverlay>>,
    mut delay: Local<Option<f32>>,
) {
    // Process new win events.
    for _ in won.read() {
        // Arm the screen shake immediately.
        shake.remaining = SHAKE_DURATION_SECS;
        shake.intensity = SHAKE_INTENSITY;
        // Start the delay timer (overwrite if a second win arrives).
        *delay = Some(WIN_SUMMARY_DELAY_SECS);
        // Clear any stale overlay from a previous win.
        for entity in &overlays {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Tick the delay timer.
    if let Some(remaining) = delay.as_mut() {
        *remaining -= time.delta_secs();
        if *remaining <= 0.0 {
            *delay = None;
            // Only spawn if there is no overlay already.
            if overlays.is_empty() {
                spawn_overlay(&mut commands, &pending);
            }
        }
    }
}

/// Despawns the win-summary modal and fires `NewGameRequestEvent` when
/// the player presses "Play Again".
fn handle_win_summary_buttons(
    interaction_query: Query<(&Interaction, &WinSummaryButton), Changed<Interaction>>,
    overlays: Query<Entity, With<WinSummaryOverlay>>,
    mut commands: Commands,
    mut new_game: EventWriter<NewGameRequestEvent>,
) {
    for (interaction, button) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match button {
            WinSummaryButton::PlayAgain => {
                // Despawn the modal.
                for entity in &overlays {
                    commands.entity(entity).despawn_recursive();
                }
                new_game.send(NewGameRequestEvent::default());
            }
        }
    }
}

/// Applies a decaying sinusoidal offset to the main `Camera2d` each frame
/// while `ScreenShakeResource::remaining > 0`.
///
/// Uses a deterministic oscillation (`sin`/`cos` of total elapsed time) to
/// avoid a dependency on a random-number crate in this crate.
fn apply_screen_shake(
    mut shake: ResMut<ScreenShakeResource>,
    time: Res<Time>,
    mut cameras: Query<&mut Transform, With<Camera2d>>,
) {
    let dt = time.delta_secs();
    if shake.remaining <= 0.0 {
        // Ensure the camera is back at origin whenever shake is idle.
        for mut t in &mut cameras {
            t.translation.x = 0.0;
            t.translation.y = 0.0;
        }
        return;
    }

    shake.remaining = (shake.remaining - dt).max(0.0);
    // Decay factor: 1.0 at start, 0.0 at end.
    let decay = shake.remaining / SHAKE_DURATION_SECS;
    let elapsed = time.elapsed_secs();
    let offset_x = (elapsed * 47.0).sin() * shake.intensity * decay;
    let offset_y = (elapsed * 31.0).cos() * shake.intensity * decay;

    for mut t in &mut cameras {
        t.translation.x = offset_x;
        t.translation.y = offset_y;
    }
}

// ---------------------------------------------------------------------------
// UI construction
// ---------------------------------------------------------------------------

fn spawn_overlay(commands: &mut Commands, pending: &WinSummaryPending) {
    commands
        .spawn((
            WinSummaryOverlay,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(0.0),
                top: Val::Percent(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.70)),
            ZIndex(300),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(36.0)),
                    row_gap: Val::Px(18.0),
                    min_width: Val::Px(320.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.10, 0.12, 0.10)),
                BorderRadius::all(Val::Px(12.0)),
            ))
            .with_children(|card| {
                // Heading
                card.spawn((
                    Text::new("You Won!"),
                    TextFont { font_size: 42.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.87, 0.0)),
                ));

                // Score
                card.spawn((
                    Text::new(format!("Score: {}", pending.score)),
                    TextFont { font_size: 26.0, ..default() },
                    TextColor(Color::WHITE),
                ));

                // Time
                card.spawn((
                    Text::new(format!("Time: {}", format_win_time(pending.time_seconds))),
                    TextFont { font_size: 26.0, ..default() },
                    TextColor(Color::WHITE),
                ));

                // XP
                card.spawn((
                    Text::new(format!("XP earned: +{}", pending.xp)),
                    TextFont { font_size: 22.0, ..default() },
                    TextColor(Color::srgb(0.4, 1.0, 0.4)),
                ));

                // Play Again button
                card.spawn((
                    WinSummaryButton::PlayAgain,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(28.0), Val::Px(12.0)),
                        justify_content: JustifyContent::Center,
                        margin: UiRect::top(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.22, 0.45, 0.22)),
                    BorderRadius::all(Val::Px(6.0)),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new("Play Again"),
                        TextFont { font_size: 22.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
            });
        });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_win_time_zero() {
        assert_eq!(format_win_time(0), "0:00");
    }

    #[test]
    fn format_win_time_one_minute_five_seconds() {
        assert_eq!(format_win_time(65), "1:05");
    }

    #[test]
    fn format_win_time_exact_minute() {
        assert_eq!(format_win_time(120), "2:00");
    }

    #[test]
    fn format_win_time_large() {
        // 3661 s = 61 min 1 s
        assert_eq!(format_win_time(3661), "61:01");
    }

    #[test]
    fn format_win_time_59_seconds() {
        assert_eq!(format_win_time(59), "0:59");
    }

    #[test]
    fn screen_shake_resource_default_is_idle() {
        let shake = ScreenShakeResource::default();
        assert!(shake.remaining <= 0.0);
    }

    #[test]
    fn win_summary_pending_default_is_zeroed() {
        let p = WinSummaryPending::default();
        assert_eq!(p.score, 0);
        assert_eq!(p.time_seconds, 0);
        assert_eq!(p.xp, 0);
    }

    #[test]
    fn win_summary_plugin_inserts_resources() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(WinSummaryPlugin);
        app.update();
        assert!(app.world().get_resource::<WinSummaryPending>().is_some());
        assert!(app.world().get_resource::<ScreenShakeResource>().is_some());
    }

    #[test]
    fn cache_win_data_sets_score_and_time() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(WinSummaryPlugin);
        app.update();

        app.world_mut()
            .send_event(GameWonEvent { score: 1234, time_seconds: 90 });
        app.update();

        let pending = app.world().resource::<WinSummaryPending>();
        assert_eq!(pending.score, 1234);
        assert_eq!(pending.time_seconds, 90);
    }

    #[test]
    fn cache_win_data_sets_xp_from_xp_awarded_event() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(WinSummaryPlugin);
        app.update();

        app.world_mut().send_event(GameWonEvent { score: 0, time_seconds: 0 });
        app.world_mut().send_event(XpAwardedEvent { amount: 75 });
        app.update();

        let pending = app.world().resource::<WinSummaryPending>();
        assert_eq!(pending.xp, 75);
    }

    #[test]
    fn game_won_event_arms_screen_shake() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(WinSummaryPlugin);
        app.update();

        app.world_mut()
            .send_event(GameWonEvent { score: 0, time_seconds: 0 });
        app.update();

        let shake = app.world().resource::<ScreenShakeResource>();
        assert!(shake.remaining > 0.0, "shake must be armed after GameWonEvent");
    }
}
