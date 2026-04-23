//! Bevy integration layer for Solitaire Quest.
//!
//! Currently exposes `GamePlugin` plus the resources and events it owns.
//! Additional plugins (`TablePlugin`, `CardPlugin`, `InputPlugin`,
//! `AnimationPlugin`, etc.) land in later sub-phases of Phase 3.

pub mod events;
pub mod game_plugin;
pub mod resources;

pub use events::{
    CardFlippedEvent, DrawRequestEvent, GameWonEvent, MoveRequestEvent, NewGameRequestEvent,
    StateChangedEvent, UndoRequestEvent,
};
pub use game_plugin::GamePlugin;
pub use resources::{DragState, GameStateResource, SyncStatus, SyncStatusResource};
