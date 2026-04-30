//! Toggleable main menu overlay showing the current game mode and a full
//! keyboard shortcut reference.
//!
//! Press **M** to open or close the overlay.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use solitaire_core::game_state::GameMode;

use crate::font_plugin::FontResource;
use crate::resources::GameStateResource;
use crate::ui_modal::{
    spawn_modal, spawn_modal_actions, spawn_modal_button, spawn_modal_header, ButtonVariant,
};
use crate::ui_theme::{
    ACCENT_PRIMARY, BORDER_SUBTLE, RADIUS_SM, STATE_INFO, TEXT_PRIMARY, TEXT_SECONDARY, TYPE_BODY,
    TYPE_BODY_LG, TYPE_CAPTION, VAL_SPACE_1, VAL_SPACE_2, VAL_SPACE_3, Z_MODAL_PANEL,
};

/// Marker component on the home-menu overlay root node.
#[derive(Component, Debug)]
pub struct HomeScreen;

/// Marker on the "Done" button inside the Home modal.
#[derive(Component, Debug)]
pub struct HomeCloseButton;

/// Registers the M-key toggle and the overlay spawn/despawn logic.
pub struct HomePlugin;

impl Plugin for HomePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (toggle_home_screen, handle_home_close_button));
    }
}

fn toggle_home_screen(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    game: Res<GameStateResource>,
    font_res: Option<Res<FontResource>>,
    screens: Query<Entity, With<HomeScreen>>,
) {
    if !keys.just_pressed(KeyCode::KeyM) {
        return;
    }
    if let Ok(entity) = screens.single() {
        commands.entity(entity).despawn();
    } else {
        spawn_home_screen(&mut commands, &game, font_res.as_deref());
    }
}

fn handle_home_close_button(
    mut commands: Commands,
    close_buttons: Query<&Interaction, (With<HomeCloseButton>, Changed<Interaction>)>,
    screens: Query<Entity, With<HomeScreen>>,
) {
    if !close_buttons.iter().any(|i| *i == Interaction::Pressed) {
        return;
    }
    for entity in &screens {
        commands.entity(entity).despawn();
    }
}

/// Spawns the home-menu modal — a hotkey reference grouped into "Game
/// Controls" and "Screens" sections plus the current game mode badge.
/// A future pass can pivot Home into a true mode launcher (the
/// Modes-popover already covers that path from the action bar).
fn spawn_home_screen(
    commands: &mut Commands,
    game: &GameStateResource,
    font_res: Option<&FontResource>,
) {
    let mode_label = match game.0.mode {
        GameMode::Classic => "Classic",
        GameMode::Zen => "Zen",
        GameMode::Challenge => "Challenge",
        GameMode::TimeAttack => "Time Attack",
    };

    let font_handle = font_res.map(|f| f.0.clone()).unwrap_or_default();
    let font_section = TextFont {
        font: font_handle.clone(),
        font_size: TYPE_BODY_LG,
        ..default()
    };
    let font_row = TextFont {
        font: font_handle.clone(),
        font_size: TYPE_BODY,
        ..default()
    };
    let font_kbd = TextFont {
        font: font_handle,
        font_size: TYPE_CAPTION,
        ..default()
    };

    spawn_modal(commands, HomeScreen, Z_MODAL_PANEL, |card| {
        spawn_modal_header(card, "Solitaire Quest", font_res);

        // Mode badge — current game's mode, ACCENT_PRIMARY so it pops.
        card.spawn((
            Text::new(format!("Current mode: {mode_label}")),
            font_section.clone(),
            TextColor(ACCENT_PRIMARY),
        ));

        // Game controls section.
        card.spawn((
            Text::new("Game Controls"),
            font_section.clone(),
            TextColor(STATE_INFO),
        ));
        for (key, action) in [
            ("N", "New game  (N again confirms)"),
            ("U", "Undo last move"),
            ("Space / D", "Draw from stock"),
            ("G", "Forfeit current game"),
            ("Tab", "Cycle hint highlight"),
            ("Enter", "Auto-complete if available"),
        ] {
            spawn_shortcut_row(card, key, action, &font_row, &font_kbd);
        }

        // Screens section.
        card.spawn((
            Text::new("Screens"),
            font_section.clone(),
            TextColor(STATE_INFO),
        ));
        for (key, action) in [
            ("M", "Main menu (this screen)"),
            ("S", "Statistics"),
            ("A", "Achievements"),
            ("O", "Settings"),
            ("P", "Profile"),
            ("L", "Leaderboard"),
            ("F1", "Help"),
            ("F11", "Toggle fullscreen"),
            ("Esc", "Pause / Resume"),
        ] {
            spawn_shortcut_row(card, key, action, &font_row, &font_kbd);
        }

        spawn_modal_actions(card, |actions| {
            spawn_modal_button(
                actions,
                HomeCloseButton,
                "Done",
                Some("M"),
                ButtonVariant::Primary,
                font_res,
            );
        });
    });
}

/// One row inside Home's controls reference: a kbd-chip + description.
/// Same look as Help's rows so the two screens read consistently.
fn spawn_shortcut_row(
    parent: &mut ChildSpawnerCommands,
    key: &str,
    action: &str,
    font_row: &TextFont,
    font_kbd: &TextFont,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: VAL_SPACE_3,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Node {
                    padding: UiRect::axes(VAL_SPACE_2, VAL_SPACE_1),
                    min_width: Val::Px(80.0),
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(RADIUS_SM)),
                    ..default()
                },
                BorderColor::all(BORDER_SUBTLE),
            ))
            .with_children(|chip| {
                chip.spawn((
                    Text::new(key.to_string()),
                    font_kbd.clone(),
                    TextColor(TEXT_PRIMARY),
                ));
            });
            row.spawn((
                Text::new(action.to_string()),
                font_row.clone(),
                TextColor(TEXT_SECONDARY),
            ));
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_plugin::GamePlugin;
    use crate::table_plugin::TablePlugin;

    fn headless_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(GamePlugin)
            .add_plugins(TablePlugin)
            .add_plugins(HomePlugin);
        app.init_resource::<ButtonInput<KeyCode>>();
        app.update();
        app
    }

    #[test]
    fn pressing_m_spawns_home_screen() {
        let mut app = headless_app();
        assert_eq!(
            app.world_mut()
                .query::<&HomeScreen>()
                .iter(app.world())
                .count(),
            0
        );

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyM);
        app.update();

        assert_eq!(
            app.world_mut()
                .query::<&HomeScreen>()
                .iter(app.world())
                .count(),
            1
        );
    }

    #[test]
    fn pressing_m_twice_closes_home_screen() {
        let mut app = headless_app();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyM);
        app.update();

        {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.release(KeyCode::KeyM);
            input.clear();
            input.press(KeyCode::KeyM);
        }
        app.update();

        assert_eq!(
            app.world_mut()
                .query::<&HomeScreen>()
                .iter(app.world())
                .count(),
            0
        );
    }
}
