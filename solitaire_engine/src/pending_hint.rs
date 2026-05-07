//! Async H-key hint solver, modelled on `PendingNewGameSeed` in
//! `game_plugin`.
//!
//! The synchronous version (v0.17.0) called
//! `solitaire_core::solver::try_solve_from_state` on the main thread on
//! every H press. Median latency was ~2 ms but pathological positions
//! can hit the `SolverConfig::default()` cap at ~120 ms, which is a
//! noticeable input-stall on the same frame the player sees the hint
//! request.
//!
//! This module hosts the resource and polling system that move the
//! solver call onto `AsyncComputeTaskPool`. `handle_keyboard_hint`
//! (input_plugin) becomes a thin spawn point: snapshot the state,
//! spawn the task, store the handle. The polling system takes the
//! result one frame later and surfaces the hint visuals via the
//! shared `emit_hint_visuals` helper.
//!
//! Cancel-on-replace: a fresh H press while a previous task is in
//! flight drops the previous task. Bevy's `Task` `Drop` cancels
//! cooperatively at the next await point.
//!
//! Stale-state drop: any `StateChangedEvent` (move applied, undo,
//! new game) drops the in-flight task — the position the solver was
//! reasoning about no longer exists, and surfacing a hint for the
//! old state would be confusing.

use bevy::prelude::*;
use bevy::tasks::{futures_lite::future, AsyncComputeTaskPool, Task};
use solitaire_core::game_state::GameState;
use solitaire_core::pile::PileType;
use solitaire_core::solver::{try_solve_from_state, SolverConfig, SolverResult};

use crate::card_plugin::CardEntity;
use crate::events::{HintVisualEvent, InfoToastEvent, StateChangedEvent};
use crate::input_plugin::{emit_hint_visuals, find_heuristic_hint};
use crate::resources::{GameStateResource, HintCycleIndex};

/// In-flight async work for the H-key hint.
///
/// `handle_keyboard_hint` writes here when the player presses H;
/// `poll_pending_hint_task` reads from here, polls the task, and
/// emits the hint visuals once the task completes. At most one task
/// is ever in flight: a fresh H press while a previous task is
/// running drops the previous task and queues the new one.
#[derive(Resource, Default)]
pub struct PendingHintTask {
    /// `Some` while the solver is still working on a verdict.
    inner: Option<HintTask>,
}

impl PendingHintTask {
    /// Whether a hint task is currently in flight.
    pub fn is_pending(&self) -> bool {
        self.inner.is_some()
    }

    /// Drop any in-flight task. Bevy's `Task` `Drop` cancels the
    /// underlying future cooperatively at the next await point.
    pub fn cancel(&mut self) {
        self.inner = None;
    }

    /// Spawn a new solver task for `state` with `config`. Drops any
    /// previously in-flight task first (cancel-on-replace).
    pub fn spawn(&mut self, state: GameState, config: SolverConfig) {
        let move_count_at_spawn = state.move_count;
        let handle = AsyncComputeTaskPool::get().spawn(async move {
            let outcome = try_solve_from_state(&state, &config);
            match outcome.result {
                SolverResult::Winnable => outcome
                    .first_move
                    .map(|mv| HintTaskOutput::SolverMove {
                        from: mv.source,
                        to: mv.dest,
                    })
                    .unwrap_or(HintTaskOutput::NeedsHeuristic),
                SolverResult::Unwinnable | SolverResult::Inconclusive => {
                    HintTaskOutput::NeedsHeuristic
                }
            }
        });
        self.inner = Some(HintTask {
            handle,
            move_count_at_spawn,
        });
    }
}

/// One in-flight hint search plus the snapshot data needed to detect
/// a stale result if the live state moved while the solver ran.
struct HintTask {
    handle: Task<HintTaskOutput>,
    /// `GameState.move_count` at spawn time. The poll system discards
    /// the result if the live move_count has advanced — the player
    /// applied a move while the solver ran, so the hint would be
    /// stale even if the StateChangedEvent drop didn't fire first.
    move_count_at_spawn: u32,
}

/// What the solver task carries back to the main thread.
enum HintTaskOutput {
    /// Solver verdict was `Winnable`; here is the first move on the
    /// solution path.
    SolverMove {
        from: PileType,
        to: PileType,
    },
    /// Solver was `Unwinnable` or `Inconclusive`. The poll system
    /// runs the legacy heuristic against the live `GameState` so the
    /// H key always produces feedback while any legal move exists.
    NeedsHeuristic,
}

/// Drop the in-flight hint task whenever the live `GameState` shifts.
///
/// The position the solver was reasoning about no longer matches the
/// live state, so its result would be stale. Mirrors the semantics
/// of `reset_hint_cycle_on_state_change` for `HintCycleIndex`.
pub fn drop_pending_hint_on_state_change(
    mut state_events: MessageReader<StateChangedEvent>,
    mut pending: ResMut<PendingHintTask>,
) {
    if state_events.read().next().is_some() {
        pending.cancel();
    }
}

/// Poll the in-flight hint solver task. When the task resolves, run
/// `emit_hint_visuals` on the result — either the solver's
/// provably-best first move (Winnable verdict) or a heuristic hint
/// over the live state (Unwinnable / Inconclusive).
///
/// Discards the result when `GameState.move_count` has moved past the
/// snapshot taken at spawn time — the player applied a move during
/// the solve and `drop_pending_hint_on_state_change` should have
/// already cleared the resource, but we double-check here for the
/// rare case where the solver task completed in the same frame the
/// move was applied.
#[allow(clippy::too_many_arguments)]
pub fn poll_pending_hint_task(
    mut pending: ResMut<PendingHintTask>,
    game: Option<Res<GameStateResource>>,
    mut hint_cycle: ResMut<HintCycleIndex>,
    mut commands: Commands,
    card_entities: Query<(Entity, &CardEntity, &mut Sprite)>,
    mut info_toast: MessageWriter<InfoToastEvent>,
    mut hint_visual: MessageWriter<HintVisualEvent>,
) {
    let Some(p) = pending.inner.as_mut() else {
        return;
    };
    let Some(output) = future::block_on(future::poll_once(&mut p.handle)) else {
        return;
    };
    let move_count_at_spawn = p.move_count_at_spawn;
    pending.inner = None;

    let Some(g) = game else { return };
    if g.0.move_count != move_count_at_spawn {
        return;
    }

    let (from, to) = match output {
        HintTaskOutput::SolverMove { from, to } => (from, to),
        HintTaskOutput::NeedsHeuristic => {
            match find_heuristic_hint(&g.0, &mut hint_cycle) {
                Some(pair) => pair,
                None => {
                    info_toast.write(InfoToastEvent("No hints available".to_string()));
                    return;
                }
            }
        }
    };
    emit_hint_visuals(
        &g.0,
        &from,
        &to,
        &mut commands,
        card_entities,
        &mut info_toast,
        &mut hint_visual,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::HintVisualEvent;
    use crate::input_plugin::HintSolverConfig;
    use solitaire_core::card::{Card, Rank, Suit};
    use solitaire_core::game_state::{DrawMode, GameState};

    /// Build a minimal Bevy app exercising only the polling system
    /// and the resources/messages it touches.
    fn pending_hint_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<InfoToastEvent>();
        app.add_message::<HintVisualEvent>();
        app.add_message::<StateChangedEvent>();
        app.init_resource::<HintCycleIndex>();
        app.init_resource::<HintSolverConfig>();
        app.init_resource::<PendingHintTask>();
        // Chain the drop-on-state-change system before the poll
        // system, mirroring how `InputPlugin::build` wires them.
        // Without this, system order is unspecified and the
        // state_change_drops_in_flight_task test sometimes sees the
        // poll fire before the drop.
        app.add_systems(
            Update,
            (
                drop_pending_hint_on_state_change,
                poll_pending_hint_task,
            )
                .chain(),
        );
        app
    }

    /// Same near-finished fixture used by the v0.17 hint tests:
    /// foundations hold A..Q for each suit, four Kings sit on
    /// tableau columns 0..3, stock and waste empty.
    fn near_finished_state() -> GameState {
        let mut game = GameState::new(1, DrawMode::DrawOne);
        for slot in 0..4_u8 {
            game.piles
                .get_mut(&PileType::Foundation(slot))
                .unwrap()
                .cards
                .clear();
        }
        for i in 0..7_usize {
            game.piles
                .get_mut(&PileType::Tableau(i))
                .unwrap()
                .cards
                .clear();
        }
        game.piles.get_mut(&PileType::Stock).unwrap().cards.clear();
        game.piles.get_mut(&PileType::Waste).unwrap().cards.clear();
        let suits = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];
        let ranks_below_king = [
            Rank::Ace, Rank::Two, Rank::Three, Rank::Four, Rank::Five,
            Rank::Six, Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
            Rank::Jack, Rank::Queen,
        ];
        for (slot, suit) in suits.iter().enumerate() {
            let pile = game
                .piles
                .get_mut(&PileType::Foundation(slot as u8))
                .unwrap();
            for (i, rank) in ranks_below_king.iter().enumerate() {
                pile.cards.push(Card {
                    id: (slot as u32) * 13 + i as u32,
                    suit: *suit,
                    rank: *rank,
                    face_up: true,
                });
            }
        }
        for (col, suit) in suits.iter().enumerate() {
            game.piles
                .get_mut(&PileType::Tableau(col))
                .unwrap()
                .cards
                .push(Card {
                    id: 100 + col as u32,
                    suit: *suit,
                    rank: Rank::King,
                    face_up: true,
                });
        }
        game
    }

    /// Spawning a task and pumping update() until it completes must
    /// emit a HintVisualEvent. Mirrors the `winnable_seed_search_*`
    /// pattern in game_plugin tests — drives a wall-clock-bounded
    /// loop so the shared AsyncComputeTaskPool can schedule the
    /// future under cargo-test parallelism.
    #[test]
    fn winnable_solver_emits_hint_after_async_completes() {
        let mut app = pending_hint_app();
        app.insert_resource(GameStateResource(near_finished_state()));
        let cfg = app.world().resource::<HintSolverConfig>().0;
        app.world_mut()
            .resource_mut::<PendingHintTask>()
            .spawn(near_finished_state(), cfg);

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        while app.world().resource::<PendingHintTask>().is_pending() {
            app.update();
            std::thread::yield_now();
            if std::time::Instant::now() >= deadline {
                break;
            }
        }
        assert!(
            !app.world().resource::<PendingHintTask>().is_pending(),
            "hint task should have completed within 15 s wall-clock",
        );
        let messages = app.world().resource::<Messages<HintVisualEvent>>();
        let mut cursor = messages.get_cursor();
        let collected: Vec<HintVisualEvent> = cursor.read(messages).cloned().collect();
        assert_eq!(
            collected.len(), 1,
            "exactly one HintVisualEvent must fire when the solver returns Winnable",
        );
        assert!(
            matches!(collected[0].dest_pile, PileType::Foundation(_)),
            "solver hint destination must be a foundation slot; got {:?}",
            collected[0].dest_pile,
        );
    }

    /// A StateChangedEvent fired while the task is in flight must
    /// drop the task; the polling system must not emit any visuals
    /// once the result eventually arrives.
    #[test]
    fn state_change_drops_in_flight_task() {
        let mut app = pending_hint_app();
        app.insert_resource(GameStateResource(near_finished_state()));
        let cfg = app.world().resource::<HintSolverConfig>().0;
        app.world_mut()
            .resource_mut::<PendingHintTask>()
            .spawn(near_finished_state(), cfg);
        assert!(
            app.world().resource::<PendingHintTask>().is_pending(),
            "task is in flight after spawn",
        );

        // Fire a StateChangedEvent before draining the task. The
        // drop-on-state-change system runs in the same Update tick
        // and clears the resource.
        app.world_mut().write_message(StateChangedEvent);
        app.update();

        assert!(
            !app.world().resource::<PendingHintTask>().is_pending(),
            "StateChangedEvent must drop the in-flight hint task",
        );
        // No HintVisualEvent should ever have fired.
        let messages = app.world().resource::<Messages<HintVisualEvent>>();
        let mut cursor = messages.get_cursor();
        assert_eq!(
            cursor.read(messages).count(),
            0,
            "dropped hint task must not emit any visuals",
        );
    }

    /// Cancel-on-replace: spawning a fresh task while a previous one
    /// is in flight must drop the previous task. Only the second
    /// spawn's result is allowed to surface.
    #[test]
    fn second_spawn_drops_first_in_flight_task() {
        let mut app = pending_hint_app();
        app.insert_resource(GameStateResource(near_finished_state()));
        let cfg = app.world().resource::<HintSolverConfig>().0;

        // First spawn.
        app.world_mut()
            .resource_mut::<PendingHintTask>()
            .spawn(near_finished_state(), cfg);
        let first_handle_present = app.world().resource::<PendingHintTask>().is_pending();
        assert!(first_handle_present);

        // Second spawn. The `spawn` helper drops the prior task
        // before assigning the new one — at no point are two tasks
        // in flight.
        app.world_mut()
            .resource_mut::<PendingHintTask>()
            .spawn(near_finished_state(), cfg);
        // Resource still pending (the second task), but the first
        // is gone. We can't directly observe the first handle once
        // it's been overwritten — what we *can* assert is that the
        // resource still holds a single task, and that task
        // eventually completes producing exactly one hint visual.
        assert!(app.world().resource::<PendingHintTask>().is_pending());

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        while app.world().resource::<PendingHintTask>().is_pending() {
            app.update();
            std::thread::yield_now();
            if std::time::Instant::now() >= deadline {
                break;
            }
        }
        assert!(
            !app.world().resource::<PendingHintTask>().is_pending(),
            "second hint task should have completed within 15 s wall-clock",
        );
        let messages = app.world().resource::<Messages<HintVisualEvent>>();
        let mut cursor = messages.get_cursor();
        let collected: Vec<HintVisualEvent> = cursor.read(messages).cloned().collect();
        assert_eq!(
            collected.len(), 1,
            "cancel-on-replace: only the surviving task's result emits a visual",
        );
    }
}
