//! First-run onboarding banner.
//!
//! On startup, if `Settings.first_run_complete` is `false`, spawn a centered
//! welcome banner pointing at the **H**/`?` cheat sheet. The first key or
//! mouse-button press dismisses it, sets the flag, and persists settings —
//! so returning players never see it again.

use std::path::PathBuf;

use bevy::prelude::*;
use solitaire_data::{save_settings_to, Settings};

use crate::settings_plugin::{SettingsResource, SettingsStoragePath};

/// Marker on the onboarding overlay root node.
#[derive(Component, Debug)]
pub struct OnboardingScreen;

pub struct OnboardingPlugin;

impl Plugin for OnboardingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, spawn_if_first_run)
            .add_systems(Update, dismiss_on_any_input);
    }
}

fn spawn_if_first_run(mut commands: Commands, settings: Option<Res<SettingsResource>>) {
    let Some(s) = settings else {
        return;
    };
    if s.0.first_run_complete {
        return;
    }
    spawn_onboarding_screen(&mut commands);
}

fn dismiss_on_any_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut settings: ResMut<SettingsResource>,
    path: Option<Res<SettingsStoragePath>>,
    screens: Query<Entity, With<OnboardingScreen>>,
) {
    let Ok(entity) = screens.get_single() else {
        return;
    };
    let pressed = keys.get_just_pressed().next().is_some()
        || mouse.get_just_pressed().next().is_some();
    if !pressed {
        return;
    }
    commands.entity(entity).despawn_recursive();
    settings.0.first_run_complete = true;
    persist(path.as_deref().map(|p| &p.0), &settings.0);
}

fn persist(path: Option<&Option<PathBuf>>, settings: &Settings) {
    let Some(Some(target)) = path else {
        return;
    };
    if let Err(e) = save_settings_to(target, settings) {
        warn!("failed to save settings (onboarding): {e}");
    }
}

fn spawn_onboarding_screen(commands: &mut Commands) {
    let lines: Vec<(String, f32)> = vec![
        ("Welcome to Solitaire Quest!".to_string(), 40.0),
        (String::new(), 20.0),
        (
            "Drag cards between piles. Press D to draw, U to undo.".to_string(),
            22.0,
        ),
        (
            "Press H or ? at any time to see the full controls.".to_string(),
            22.0,
        ),
        (String::new(), 20.0),
        ("Press any key to begin".to_string(), 20.0),
    ];

    commands
        .spawn((
            OnboardingScreen,
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.92)),
            ZIndex(230),
        ))
        .with_children(|b| {
            for (line, size) in lines {
                b.spawn((
                    Text::new(line),
                    TextFont {
                        font_size: size,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.87, 0.0)),
                ));
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_plugin::SettingsPlugin;

    fn headless_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(SettingsPlugin::headless())
            .add_plugins(OnboardingPlugin);
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<ButtonInput<MouseButton>>();
        app
    }

    fn count_screens(app: &mut App) -> usize {
        app.world_mut()
            .query::<&OnboardingScreen>()
            .iter(app.world())
            .count()
    }

    #[test]
    fn first_run_spawns_banner() {
        let mut app = headless_app();
        app.update(); // PostStartup runs
        assert_eq!(count_screens(&mut app), 1);
    }

    #[test]
    fn returning_player_does_not_see_banner() {
        let mut app = headless_app();
        // Mark already-completed before PostStartup runs.
        app.world_mut()
            .resource_mut::<SettingsResource>()
            .0
            .first_run_complete = true;
        app.update();
        assert_eq!(count_screens(&mut app), 0);
    }

    #[test]
    fn keypress_dismisses_and_sets_flag() {
        let mut app = headless_app();
        app.update();
        assert_eq!(count_screens(&mut app), 1);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        app.update();

        assert_eq!(count_screens(&mut app), 0);
        assert!(
            app.world()
                .resource::<SettingsResource>()
                .0
                .first_run_complete,
            "first_run_complete should flip to true"
        );
    }

    #[test]
    fn mouseclick_dismisses_banner() {
        let mut app = headless_app();
        app.update();
        assert_eq!(count_screens(&mut app), 1);

        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();

        assert_eq!(count_screens(&mut app), 0);
    }
}
