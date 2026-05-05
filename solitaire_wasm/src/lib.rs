//! WebAssembly bindings for browser-side replay playback.
//!
//! The web replay player at `<server>/replays/<id>` fetches a [`Replay`]
//! JSON via `GET /api/replays/:id`, hands it to [`ReplayPlayer::new`],
//! and then advances frame-by-frame with [`ReplayPlayer::step`]. Each
//! step applies one [`ReplayMove`] to the underlying `GameState` and
//! returns the resulting pile snapshot as JSON for the JS layer to
//! render.
//!
//! The state machine is the same Rust [`solitaire_core::GameState`]
//! the desktop client uses, so the two implementations cannot drift —
//! same seed + same input list = same pile state at every step,
//! regardless of which platform replays the game.
//!
//! The crate intentionally does **not** depend on `solitaire_data`
//! (which pulls `dirs`, `keyring`, `reqwest`, and other non-wasm
//! crates) — instead it defines a minimal `Replay` mirror with the
//! same serde shape as `solitaire_data::Replay`. The JSON wire format
//! is the contract.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use solitaire_core::card::Suit;
use solitaire_core::game_state::{DrawMode, GameMode, GameState};
use solitaire_core::pile::PileType;
use wasm_bindgen::prelude::*;

/// Mirrors the variants of `solitaire_data::ReplayMove` v2 (atomic
/// player inputs, post-StockClick refinement). Only the JSON shape
/// matters for cross-crate compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayMove {
    Move {
        from: PileType,
        to: PileType,
        count: usize,
    },
    StockClick,
}

/// Mirrors `solitaire_data::Replay` v2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Replay {
    #[serde(default)]
    pub schema_version: u32,
    pub seed: u64,
    pub draw_mode: DrawMode,
    pub mode: GameMode,
    pub time_seconds: u64,
    pub final_score: i32,
    pub recorded_at: NaiveDate,
    pub moves: Vec<ReplayMove>,
}

/// JS-friendly snapshot of a `GameState` at a particular replay step.
#[derive(Debug, Clone, Serialize)]
pub struct StateSnapshot {
    pub step_idx: usize,
    pub total_steps: usize,
    pub score: i32,
    pub move_count: u32,
    pub is_won: bool,
    pub stock: Vec<CardSnapshot>,
    pub waste: Vec<CardSnapshot>,
    /// Length 4 — one per foundation slot, in slot order (0..=3). The
    /// claimed suit (if any) is the bottom card's suit.
    pub foundations: [Vec<CardSnapshot>; 4],
    /// Length 7 — one per tableau column (0..=6).
    pub tableaus: [Vec<CardSnapshot>; 7],
}

/// One card, projected for the JS card renderer. `face_up = false`
/// means the card back is drawn; in that case `suit` and `rank` are
/// still set (so the renderer doesn't need separate "unknown" data),
/// just hidden visually.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct CardSnapshot {
    pub id: u32,
    /// `"clubs" | "diamonds" | "hearts" | "spades"`.
    pub suit: &'static str,
    /// 1-13, where 1 is Ace and 13 is King.
    pub rank: u8,
    pub face_up: bool,
}

impl From<&solitaire_core::card::Card> for CardSnapshot {
    fn from(c: &solitaire_core::card::Card) -> Self {
        Self {
            id: c.id,
            suit: match c.suit {
                Suit::Clubs => "clubs",
                Suit::Diamonds => "diamonds",
                Suit::Hearts => "hearts",
                Suit::Spades => "spades",
            },
            rank: c.rank.value(),
            face_up: c.face_up,
        }
    }
}

/// Browser-side replay state machine. Owns a live `GameState` and the
/// replay's move list; each `step()` applies the next move.
#[wasm_bindgen]
pub struct ReplayPlayer {
    game: GameState,
    moves: Vec<ReplayMove>,
    step_idx: usize,
}

// Native-callable methods. Used by both the wasm-bindgen interface
// below and by unit tests, which can't go through `serde_wasm_bindgen`
// (it panics on non-wasm targets).
impl ReplayPlayer {
    /// Construct from a raw replay JSON string. Returns the parsing
    /// error as a `String` so the wasm-bindgen wrapper can convert
    /// it to a `JsValue` and tests can assert on it directly.
    pub fn from_json(replay_json: &str) -> Result<Self, String> {
        let replay: Replay =
            serde_json::from_str(replay_json).map_err(|e| format!("invalid replay JSON: {e}"))?;
        let game =
            GameState::new_with_mode(replay.seed, replay.draw_mode.clone(), replay.mode);
        Ok(Self {
            game,
            moves: replay.moves,
            step_idx: 0,
        })
    }

    /// Apply the next move. Returns `None` once the list is exhausted.
    pub fn step_native(&mut self) -> Option<StateSnapshot> {
        if self.step_idx >= self.moves.len() {
            return None;
        }
        let mv = self.moves[self.step_idx].clone();
        let _ = match mv {
            ReplayMove::Move { from, to, count } => self.game.move_cards(from, to, count),
            ReplayMove::StockClick => self.game.draw(),
        };
        self.step_idx += 1;
        Some(self.snapshot())
    }

    fn snapshot(&self) -> StateSnapshot {
        let pile_cards = |t: PileType| -> Vec<CardSnapshot> {
            self.game
                .piles
                .get(&t)
                .map(|p| p.cards.iter().map(CardSnapshot::from).collect())
                .unwrap_or_default()
        };
        let foundations: [Vec<CardSnapshot>; 4] = [
            pile_cards(PileType::Foundation(0)),
            pile_cards(PileType::Foundation(1)),
            pile_cards(PileType::Foundation(2)),
            pile_cards(PileType::Foundation(3)),
        ];
        let tableaus: [Vec<CardSnapshot>; 7] = [
            pile_cards(PileType::Tableau(0)),
            pile_cards(PileType::Tableau(1)),
            pile_cards(PileType::Tableau(2)),
            pile_cards(PileType::Tableau(3)),
            pile_cards(PileType::Tableau(4)),
            pile_cards(PileType::Tableau(5)),
            pile_cards(PileType::Tableau(6)),
        ];
        StateSnapshot {
            step_idx: self.step_idx,
            total_steps: self.moves.len(),
            score: self.game.score,
            move_count: self.game.move_count,
            is_won: self.game.is_won,
            stock: pile_cards(PileType::Stock),
            waste: pile_cards(PileType::Waste),
            foundations,
            tableaus,
        }
    }
}

// JS-facing surface. Thin wrapper around the native API: serialises
// `StateSnapshot` to `JsValue` via `serde_wasm_bindgen` and converts
// `String` errors to `JsValue` strings. Native unit tests bypass this
// layer because `serde_wasm_bindgen::to_value` panics off-target.
#[wasm_bindgen]
impl ReplayPlayer {
    /// Construct from a raw replay JSON string.
    #[wasm_bindgen(constructor)]
    pub fn new(replay_json: &str) -> Result<ReplayPlayer, JsValue> {
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
        Self::from_json(replay_json).map_err(|e| JsValue::from_str(&e))
    }

    /// Snapshot the current `GameState` as a JS object (see `StateSnapshot`).
    pub fn state(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.snapshot()).unwrap_or(JsValue::NULL)
    }

    /// Apply the next move; returns the post-step snapshot, or `null`
    /// once the move list is exhausted.
    pub fn step(&mut self) -> JsValue {
        match self.step_native() {
            Some(snap) => serde_wasm_bindgen::to_value(&snap).unwrap_or(JsValue::NULL),
            None => JsValue::NULL,
        }
    }

    /// Total number of moves the replay contains.
    pub fn total_steps(&self) -> usize {
        self.moves.len()
    }

    /// 0-indexed position of the next move to apply.
    pub fn step_idx(&self) -> usize {
        self.step_idx
    }

    /// Returns `true` once every move has been applied.
    pub fn is_finished(&self) -> bool {
        self.step_idx >= self.moves.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_replay_json() -> String {
        // Minimal v2 replay: seed 42, two stock clicks. Real winning
        // replays will have many more moves; for the test we just
        // verify deserialization + step() advances correctly.
        r#"{
            "schema_version": 2,
            "seed": 42,
            "draw_mode": "DrawOne",
            "mode": "Classic",
            "time_seconds": 60,
            "final_score": 100,
            "recorded_at": "2026-05-02",
            "moves": ["StockClick", "StockClick"]
        }"#
        .to_string()
    }

    /// Constructing from a valid v2 replay JSON must succeed and
    /// initialise step_idx to 0.
    #[test]
    fn new_initialises_step_idx_zero() {
        let player = ReplayPlayer::from_json(&sample_replay_json()).expect("valid JSON");
        assert_eq!(player.step_idx, 0);
        assert_eq!(player.moves.len(), 2);
    }

    /// Each step advances the index; once exhausted, step_native returns None.
    #[test]
    fn steps_advance_then_terminate() {
        let mut player = ReplayPlayer::from_json(&sample_replay_json()).expect("valid JSON");
        assert!(player.step_native().is_some());
        assert_eq!(player.step_idx, 1);
        assert!(player.step_native().is_some());
        assert_eq!(player.step_idx, 2);
        assert!(player.step_native().is_none(), "no further steps");
    }

    /// Malformed JSON returns an error rather than panicking.
    #[test]
    fn invalid_json_returns_error() {
        let result = ReplayPlayer::from_json("not valid json");
        assert!(result.is_err());
    }
}
