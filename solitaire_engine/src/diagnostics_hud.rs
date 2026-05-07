//! Optional on-screen FPS / frame-time overlay.
//!
//! Wraps Bevy's [`FrameTimeDiagnosticsPlugin`] and renders a tiny
//! corner readout that the developer (or a curious player) can toggle
//! with `F3`. Hidden by default — production builds ship the plugin
//! but the overlay starts invisible, so the production HUD is never
//! cluttered unless explicitly summoned.
//!
//! Why this exists: with an Android port on the roadmap, "feels
//! slow" became a real risk to plan around. A togglable FPS / frame-
//! time pair gives us a numeric baseline we can quote across desktop
//! and mobile, instead of optimising on vibes.
//!
//! ## Display contract
//!
//! When visible, the overlay reads `"FPS NN \u{2022} M.MM ms"` in a
//! small monospaced cell, anchored top-right. Both numbers are the
//! `smoothed()` value (Bevy's exponential moving average) — peak
//! and worst-case readings would jitter the text every frame, which
//! is harder to glance at than a smoothed reading.
//!
//! ## Hotkey scope
//!
//! `F3` is a global, gameplay-blockable toggle: the system reads
//! `ButtonInput<KeyCode>` directly and ignores the rest of the modal
//! / pause stack. The overlay is informational and shouldn't depend
//! on game state.

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::font_plugin::FontResource;
use crate::ui_theme::Z_SPLASH;

/// Z-index for the diagnostics HUD — above every modal / toast /
/// splash layer so a developer can always see the readout, no matter
/// what overlay is up.
const Z_DIAGNOSTICS_HUD: i32 = Z_SPLASH + 100;

/// Width-stable font size for the readout. Hand-tuned literal — the
/// HUD is a developer affordance and uses its own sizing rather than
/// borrowing a typography token whose meaning may drift.
const DIAGNOSTICS_FONT_SIZE: f32 = 12.0;

/// Background alpha for the readout cell. Translucent so the HUD
/// doesn't fully obscure whatever's behind it but stays legible.
const DIAGNOSTICS_BG_ALPHA: f32 = 0.7;

/// Wires the FPS / frame-time HUD overlay.
///
/// Adds [`FrameTimeDiagnosticsPlugin`] (no-op if already added — the
/// plugin's `Plugin::build` is idempotent on duplicate registration
/// in our codebase since no other site adds it). Spawns the HUD
/// hidden, registers the toggle handler, and wires the per-frame
/// text refresh.
pub struct DiagnosticsHudPlugin;

impl Plugin for DiagnosticsHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .init_resource::<DiagnosticsHudVisible>()
            .add_systems(Startup, spawn_diagnostics_hud)
            .add_systems(
                Update,
                (toggle_diagnostics_hud, update_diagnostics_hud).chain(),
            );
    }
}

/// Tracks whether the overlay is currently visible. Flipped by the
/// `F3` toggle; defaults to hidden so production launches start clean.
#[derive(Resource, Debug, Default)]
struct DiagnosticsHudVisible(bool);

/// Marker on the overlay's root Node — used to flip `Visibility`.
#[derive(Component, Debug)]
struct DiagnosticsHudRoot;

/// Marker on the readout `Text` node — used by the per-frame refresh
/// system to find the right text to overwrite.
#[derive(Component, Debug)]
struct DiagnosticsHudText;

/// Spawns the (initially-hidden) overlay at startup. Anchored
/// top-right with absolute positioning so it never participates in
/// the rest of the UI flex tree.
fn spawn_diagnostics_hud(mut commands: Commands, font_res: Option<Res<FontResource>>) {
    let font_handle = font_res.map(|f| f.0.clone()).unwrap_or_default();
    let bg = Color::srgba(0.0, 0.0, 0.0, DIAGNOSTICS_BG_ALPHA);

    commands
        .spawn((
            DiagnosticsHudRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                right: Val::Px(8.0),
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(bg),
            Visibility::Hidden,
            GlobalZIndex(Z_DIAGNOSTICS_HUD),
        ))
        .with_children(|parent| {
            parent.spawn((
                DiagnosticsHudText,
                Text::new("FPS \u{2014}"),
                TextFont {
                    font: font_handle,
                    font_size: DIAGNOSTICS_FONT_SIZE,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// `F3` flips the visible flag and the overlay's `Visibility`. Reads
/// the keyboard input directly so it isn't gated by pause / modal
/// state — diagnostics should always be reachable.
fn toggle_diagnostics_hud(
    keys: Res<ButtonInput<KeyCode>>,
    mut visible: ResMut<DiagnosticsHudVisible>,
    mut roots: Query<&mut Visibility, With<DiagnosticsHudRoot>>,
) {
    if !keys.just_pressed(KeyCode::F3) {
        return;
    }
    visible.0 = !visible.0;
    let target = if visible.0 {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut v in &mut roots {
        *v = target;
    }
}

/// Reads the smoothed FPS + frame-time diagnostics each frame and
/// rewrites the readout text. Skipped while the overlay is hidden so
/// we don't pay the diagnostic-store lookup or the text mutation
/// when nobody's looking.
fn update_diagnostics_hud(
    diagnostics: Res<DiagnosticsStore>,
    visible: Res<DiagnosticsHudVisible>,
    mut texts: Query<&mut Text, With<DiagnosticsHudText>>,
) {
    if !visible.0 {
        return;
    }
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);
    let frame_time_ms = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);
    for mut text in &mut texts {
        **text = format!("FPS {fps:.0} \u{2022} {frame_time_ms:.2} ms");
    }
}
