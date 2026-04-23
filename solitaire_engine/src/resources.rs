//! Bevy resources owned by the engine crate.

use bevy::math::Vec2;
use bevy::prelude::Resource;
use chrono::{DateTime, Utc};
use solitaire_core::game_state::GameState;
use solitaire_core::pile::PileType;

/// Wraps the currently active `GameState`. Single source of truth for the in-progress game.
#[derive(Resource, Debug, Clone)]
pub struct GameStateResource(pub GameState);

/// Tracks an in-progress drag operation.
///
/// When `cards` is empty there is no active drag. When non-empty, the listed cards
/// are being moved by the user and should be rendered at the cursor position.
#[derive(Resource, Debug, Clone, Default)]
pub struct DragState {
    pub cards: Vec<u32>,
    pub origin_pile: Option<PileType>,
    pub cursor_offset: Vec2,
    pub origin_z: f32,
}

impl DragState {
    /// Returns true when no drag is currently in progress.
    pub fn is_idle(&self) -> bool {
        self.cards.is_empty()
    }

    /// Clears the drag state.
    pub fn clear(&mut self) {
        self.cards.clear();
        self.origin_pile = None;
        self.cursor_offset = Vec2::ZERO;
        self.origin_z = 0.0;
    }
}

/// Current sync activity — shown in the settings screen.
///
/// Defined here rather than in `solitaire_data` because it is a UI/runtime
/// status value, not part of the persistence layer.
#[derive(Debug, Clone, Default)]
pub enum SyncStatus {
    #[default]
    Idle,
    Syncing,
    LastSynced(DateTime<Utc>),
    Error(String),
}

/// Bevy resource wrapping the current `SyncStatus`.
#[derive(Resource, Debug, Clone, Default)]
pub struct SyncStatusResource(pub SyncStatus);
