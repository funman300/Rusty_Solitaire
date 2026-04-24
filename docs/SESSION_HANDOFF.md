# Solitaire Quest — Session Handoff

> Last updated: 2026-04-21
> Branch: `master` — pushed to https://git.aleshym.co/funman300/Rusty_Solitare.git

---

## What Has Been Built

### Phase 1 — Workspace Setup ✅ COMPLETE

All seven Cargo crates created and compiling cleanly:

| Crate | Status | Purpose |
|---|---|---|
| `solitaire_core` | Fully implemented | Pure Rust game logic — NO Bevy, NO network |
| `solitaire_sync` | Stub | Shared API types (`SyncPayload`, `SyncResponse`) |
| `solitaire_data` | Stub | `SyncError` enum + `SyncProvider` trait |
| `solitaire_engine` | Stub | Bevy ECS systems — all plugins added in Phase 3 |
| `solitaire_server` | Stub | Axum sync server — implemented in Phase 8C |
| `solitaire_gpgs` | Compile-time stub | Google Play Games bridge — Android only, JNI in Phase: Android |
| `solitaire_app` | Working | Opens blank Bevy window titled "Solitaire Quest" at 1280×800 |

Fast compile profiles, `assets/` directory structure, and `.env.example` are all in place.

### Phase 2 — Core Game Engine ✅ COMPLETE

`solitaire_core` is fully implemented with 68 passing tests and zero clippy warnings.

**Modules:**
- `card.rs` — `Suit` (Clubs/Diamonds/Hearts/Spades, `is_red()`/`is_black()`), `Rank` (Ace–King, `value() -> u8`), `Card` (id, suit, rank, face_up)
- `pile.rs` — `PileType` (Stock, Waste, Foundation(Suit), Tableau(usize)), `Pile` (new, top)
- `error.rs` — `MoveError`: InvalidSource, InvalidDestination, EmptySource, RuleViolation(String), UndoStackEmpty, GameAlreadyWon, StockEmpty
- `deck.rs` — `Deck::new()`, `Deck::shuffle(seed: u64)` using seeded `StdRng` (cross-platform deterministic), `deal_klondike(deck) -> ([Pile; 7], Pile)`
- `rules.rs` — `can_place_on_foundation(card, pile, suit)`, `can_place_on_tableau(card, pile)`
- `scoring.rs` — `score_move(from, to)`, `score_undo()` (-15), `compute_time_bonus(elapsed_seconds)` (700_000/s)
- `game_state.rs` — `DrawMode`, `GameState` with full game loop

**GameState public API:**
```rust
GameState::new(seed: u64, draw_mode: DrawMode) -> Self
GameState::draw(&mut self) -> Result<(), MoveError>
GameState::move_cards(&mut self, from: PileType, to: PileType, count: usize) -> Result<(), MoveError>
GameState::undo(&mut self) -> Result<(), MoveError>
GameState::check_win(&self) -> bool
GameState::check_auto_complete(&self) -> bool
GameState::compute_time_bonus(&self) -> i32
GameState::undo_stack_len(&self) -> usize
```

**Key GameState rules:**
- Undo stack capped at 64 entries (oldest evicted)
- Score never goes below 0
- Waste recycling is unlimited — `StockEmpty` only when both stock AND waste are simultaneously empty
- Recycle (waste → stock) pushes a snapshot so it can be undone
- Newly exposed top card of source pile is flipped face-up automatically on `move_cards`
- Win: all 4 foundations at 13 cards
- Auto-complete: stock empty + waste empty + all tableau cards face-up

---

## Commit History

```
b8dc7cb fix(core): remove stock_recycled limit, replace unwrap, snapshot on recycle, fix derives
58f1465 feat(core): add GameState with draw, move_cards, undo, win/auto-complete detection
43194b0 fix(core): use StdRng doc comment, replace expect() with debug_assert in deal_klondike
17bbec0 feat(core): add pile, error, deck, rules, scoring modules with tests
fcf878b feat(core): add Card, Suit, Rank types with tests
f84d7c5 fix(workspace): add derives/docs per code review, remove unused thiserror from solitaire_sync
684f077 feat(workspace): initialize all seven crates with stubs and blank Bevy window
```

---

## What Is Next

### Phase 3 — Bevy Rendering & Interaction (`solitaire_engine`)

This is the next phase to implement. Key tasks:

- Add `GameStateResource`, `DragState`, `SyncStatusResource` Bevy resources
- Add Bevy events: `MoveRequestEvent`, `DrawRequestEvent`, `UndoRequestEvent`, `NewGameRequestEvent`, `StateChangedEvent`, `GameWonEvent`
- `CardPlugin` — spawn card entities with 2D sprites, drag-and-drop input
- `TablePlugin` — pile markers, table background, layout calculation from window size
- `AnimationPlugin` — card slide (lerp 0.15s), flip (scale X 0.2s), win cascade, toast
- `GamePlugin` — wire `GameStateResource`, route input events to `solitaire_core::GameState`
- Responsive layout: recalculate positions on `WindowResized`
- Keyboard shortcuts: U=undo, N=new game, D=draw, Escape=pause

See the full spec in the master prompt (originally pasted by the user) or in `ARCHITECTURE.md` section 5.

### Phases 4–8 (in order after Phase 3)

| Phase | Scope |
|---|---|
| Phase 4 | Statistics (`StatsSnapshot`, persist to `stats.json`, stats screen in egui) |
| Phase 5 | Achievements (20+ achievements, `AchievementPlugin`, toast queue) |
| Phase 6 | XP/levels, daily challenges, weekly goals, special modes |
| Phase 7 | Audio (`bevy_kira_audio`), polish, hints, onboarding, pause menu |
| Phase 8A–C | Local storage + `SyncProvider` + self-hosted Axum server + client |
| Phase 8D | GPGS stub fully wired into settings UI (already compiles, just UI) |

---

## Important Implementation Notes

### Versions (Cargo.toml workspace deps)

- `bevy = "0.15"` (resolved to 0.15.3) — UI via built-in `bevy::ui`, no bevy_egui
- `kira = "0.9"` — audio via `kira` crate directly, no bevy_kira_audio or AssetServer
- `rand = "0.8"` — note: `small_rng` feature is NOT enabled; use `StdRng`, not `SmallRng`

### Asset strategy

- No `AssetServer` — assets embedded at compile time using `include_bytes!()`
- Fonts: `Font::try_from_bytes(include_bytes!("../assets/fonts/main.ttf"))`
- Audio: load from `&[u8]` via `kira` `StaticSoundData::from_cursor()`
- Card rendering: procedural (`bevy::prelude::Sprite` + `Text2d`) — no sprite sheets required

### Hard rules (from CLAUDE.md)
- `solitaire_core` and `solitaire_sync` must NEVER gain Bevy or network dependencies
- No `unwrap()` or `panic!()` in game logic — use `Result<_, MoveError>` everywhere
- All state transitions return `Result` — `debug_assert!` is acceptable for structural invariants
- `SyncPlugin` must NEVER match on `SyncBackend` enum inside a Bevy system — always call through the `SyncProvider` trait
- Atomic file writes only: write to `.tmp` then `rename()`
- `cargo clippy --workspace -- -D warnings` must pass clean
- `cargo test --workspace` must pass clean

### Lessons from this session
- `rand = "0.8"` without `features = ["small_rng"]` means `SmallRng` is unavailable — use `StdRng`
- `tower-governor` uses underscores in the crate name (not hyphens in Cargo.toml)
- When implementing `draw()` in `GameState`: recycle is unlimited, stop condition is BOTH piles empty simultaneously
- Recycle must push a snapshot (so it can be undone) even though it doesn't count as a "move"

---

## Implementation Plan Document

The detailed task-by-task plan for Phases 1 and 2 is at:
`docs/superpowers/plans/2026-04-20-phase1-2-workspace-core.md`

For Phase 3 onwards, write a new plan using the `superpowers:writing-plans` skill before starting implementation.

---

## Running the Project

```bash
# Check everything compiles
cargo check --workspace

# Run all tests (68 tests, all should pass)
cargo test --workspace

# Lint (must be zero warnings)
cargo clippy --workspace -- -D warnings

# Run the game (blank window for now — rendering added in Phase 3)
cargo run -p solitaire_app --features bevy/dynamic_linking
```
