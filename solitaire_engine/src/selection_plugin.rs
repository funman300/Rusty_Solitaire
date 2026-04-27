//! Keyboard-driven card selection (Task #68).
//!
//! Pressing `Tab` cycles through piles that have a face-up draggable top card.
//! Pressing `Enter` or `Space` fires a [`MoveRequestEvent`] to the best
//! available destination (foundation first, then tableau), then clears the
//! selection. Pressing `Escape` clears the selection without moving.
//!
//! The selected card is highlighted by a cyan [`SelectionHighlight`] outline
//! sprite parented to the selected card entity. The highlight is despawned when
//! the selection is cleared.

use bevy::input::ButtonInput;
use bevy::prelude::*;
use solitaire_core::card::Suit;
use solitaire_core::pile::PileType;

use crate::card_plugin::CardEntity;
use crate::events::MoveRequestEvent;
use crate::game_plugin::GameMutation;
use crate::input_plugin::best_destination;
use crate::layout::LayoutResource;
use crate::pause_plugin::PausedResource;
use crate::resources::GameStateResource;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Tracks which pile currently has keyboard focus.
///
/// `None` means no pile is selected.
#[derive(Resource, Debug, Default)]
pub struct SelectionState {
    /// The pile whose top face-up card is currently selected, or `None`.
    pub selected_pile: Option<PileType>,
}

/// Marker component placed on the outline sprite used as the keyboard-selection
/// highlight.
///
/// Exactly one entity with this marker should exist at any time. It is
/// despawned when the selection is cleared.
#[derive(Component, Debug)]
pub struct SelectionHighlight;

/// Registers the keyboard selection resources and systems.
pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionState>()
            .add_systems(
                Update,
                (
                    handle_selection_keys.before(GameMutation),
                    update_selection_highlight.after(GameMutation),
                ),
            );
    }
}

// ---------------------------------------------------------------------------
// Pile cycle order
// ---------------------------------------------------------------------------

/// The ordered list of piles that are considered for keyboard cycling.
///
/// Order: Waste → Foundation×4 → Tableau 0–6.
fn cycled_piles() -> Vec<PileType> {
    let mut piles = vec![
        PileType::Waste,
        PileType::Foundation(Suit::Clubs),
        PileType::Foundation(Suit::Diamonds),
        PileType::Foundation(Suit::Hearts),
        PileType::Foundation(Suit::Spades),
    ];
    for i in 0..7_usize {
        piles.push(PileType::Tableau(i));
    }
    piles
}

/// Given a list of *available* piles and the currently selected pile, return
/// the next pile in cycling order, wrapping around.
///
/// If `current` is `None` the first available pile is returned.
/// If `available` is empty, `None` is returned.
pub fn cycle_next_pile(
    available: &[PileType],
    current: Option<&PileType>,
) -> Option<PileType> {
    if available.is_empty() {
        return None;
    }
    let order = cycled_piles();

    let Some(cur) = current else {
        // No current selection: return the first available pile in cycle order.
        return order.iter().find(|p| available.contains(p)).cloned();
    };

    // Find the position of `cur` inside the ordered list, then scan forward
    // for the next available pile (wrapping).
    let cur_pos = order.iter().position(|p| p == cur);
    let start = cur_pos.map_or(0, |pos| pos + 1);

    // Search from `start` forward, wrapping around, skipping `cur`.
    let n = order.len();
    for offset in 0..n {
        let candidate = &order[(start + offset) % n];
        if available.contains(candidate) {
            return Some(candidate.clone());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Handles Tab / Enter / Space / Escape for keyboard card selection.
fn handle_selection_keys(
    keys: Res<ButtonInput<KeyCode>>,
    paused: Option<Res<PausedResource>>,
    game: Res<GameStateResource>,
    mut selection: ResMut<SelectionState>,
    mut moves: EventWriter<MoveRequestEvent>,
) {
    if paused.is_some_and(|p| p.0) {
        return;
    }

    // Build the list of piles that currently have a face-up draggable top card.
    let available: Vec<PileType> = {
        let all = [
            PileType::Waste,
            PileType::Foundation(Suit::Clubs),
            PileType::Foundation(Suit::Diamonds),
            PileType::Foundation(Suit::Hearts),
            PileType::Foundation(Suit::Spades),
            PileType::Tableau(0),
            PileType::Tableau(1),
            PileType::Tableau(2),
            PileType::Tableau(3),
            PileType::Tableau(4),
            PileType::Tableau(5),
            PileType::Tableau(6),
        ];
        all.into_iter()
            .filter(|p| {
                game.0
                    .piles
                    .get(p)
                    .and_then(|pile| pile.cards.last())
                    .is_some_and(|c| c.face_up)
            })
            .collect()
    };

    // Tab — cycle selection.
    if keys.just_pressed(KeyCode::Tab) {
        selection.selected_pile =
            cycle_next_pile(&available, selection.selected_pile.as_ref());
        return;
    }

    // Escape — clear selection.
    if keys.just_pressed(KeyCode::Escape) {
        selection.selected_pile = None;
        return;
    }

    // Enter / Space — execute move for the selected pile's top card.
    let activate =
        keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space);
    if activate {
        if let Some(ref pile) = selection.selected_pile.clone() {
            if let Some(card) = game
                .0
                .piles
                .get(pile)
                .and_then(|p| p.cards.last())
                .filter(|c| c.face_up)
            {
                if let Some(dest) = best_destination(card, &game.0) {
                    moves.send(MoveRequestEvent {
                        from: pile.clone(),
                        to: dest,
                        count: 1,
                    });
                    selection.selected_pile = None;
                }
            }
        }
    }
}

/// Maintains the `SelectionHighlight` outline sprite.
///
/// When a pile is selected, a cyan sprite is placed at the selected card's
/// position. When the selection is cleared the highlight entity is despawned.
fn update_selection_highlight(
    mut commands: Commands,
    selection: Res<SelectionState>,
    game: Res<GameStateResource>,
    layout: Option<Res<LayoutResource>>,
    card_entities: Query<(Entity, &CardEntity)>,
    highlights: Query<Entity, With<SelectionHighlight>>,
) {
    // Always despawn any existing highlight first.
    for entity in &highlights {
        commands.entity(entity).despawn_recursive();
    }

    let Some(ref pile) = selection.selected_pile else {
        return;
    };
    let Some(layout) = layout else {
        return;
    };
    let Some(card) = game
        .0
        .piles
        .get(pile)
        .and_then(|p| p.cards.last())
        .filter(|c| c.face_up)
    else {
        return;
    };

    let card_id = card.id;
    let card_size = layout.0.card_size;

    // Find the entity for the selected card so we can read its position.
    for (entity, card_entity) in &card_entities {
        if card_entity.card_id == card_id {
            // Spawn the highlight as a child of the card entity so it moves
            // with it automatically.
            commands.entity(entity).with_children(|b| {
                b.spawn((
                    SelectionHighlight,
                    Sprite {
                        color: Color::srgba(0.0, 1.0, 1.0, 0.5),
                        custom_size: Some(card_size + Vec2::splat(4.0)),
                        ..default()
                    },
                    // Slightly behind the card face so text labels are still visible.
                    Transform::from_xyz(0.0, 0.0, -0.01),
                    Visibility::default(),
                ));
            });
            break;
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn piles_from(names: &[&str]) -> Vec<PileType> {
        names
            .iter()
            .map(|&n| match n {
                "Waste" => PileType::Waste,
                "T0" => PileType::Tableau(0),
                "T1" => PileType::Tableau(1),
                "T2" => PileType::Tableau(2),
                _ => PileType::Waste,
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // Task #68 — cycle_next_pile pure-function tests
    // -----------------------------------------------------------------------

    #[test]
    fn cycle_next_pile_from_none() {
        // With [Waste, Tableau(0), Tableau(1)] available, starting from None → Waste.
        let available = piles_from(&["Waste", "T0", "T1"]);
        let result = cycle_next_pile(&available, None);
        assert_eq!(result, Some(PileType::Waste));
    }

    #[test]
    fn cycle_next_pile_from_waste() {
        // Starting from Waste → Tableau(0).
        let available = piles_from(&["Waste", "T0", "T1"]);
        let result = cycle_next_pile(&available, Some(&PileType::Waste));
        assert_eq!(result, Some(PileType::Tableau(0)));
    }

    #[test]
    fn cycle_next_pile_wraps() {
        // Starting from Tableau(1) → Waste (wraps back to start).
        let available = piles_from(&["Waste", "T0", "T1"]);
        let result = cycle_next_pile(&available, Some(&PileType::Tableau(1)));
        assert_eq!(result, Some(PileType::Waste));
    }

    #[test]
    fn cycle_next_pile_empty_returns_none() {
        let result = cycle_next_pile(&[], None);
        assert!(result.is_none());
    }

    #[test]
    fn cycle_next_pile_single_element_wraps_to_itself() {
        let available = vec![PileType::Waste];
        let result = cycle_next_pile(&available, Some(&PileType::Waste));
        assert_eq!(result, Some(PileType::Waste));
    }
}
