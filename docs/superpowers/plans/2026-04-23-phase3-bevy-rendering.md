# Phase 3 — Bevy Rendering & Interaction

> Status: In progress (started 2026-04-23)
> Crate: `solitaire_engine`
> Depends on: `solitaire_core` (complete), `bevy = 0.15`, `bevy_egui = 0.30`

---

## Scope

Make the game playable with a graphical interface. This phase takes `solitaire_engine` from an empty stub to a full Bevy rendering + input layer wired to `solitaire_core::GameState`.

Out of scope (later phases):
- Persistence (`StatsSnapshot`, file I/O) — Phase 4
- Achievements toast content — Phase 5
- Audio — Phase 7
- Sync — Phase 8

---

## Sub-phases

### 3A — Plumbing & event wiring

**Modules under `solitaire_engine/src/`:**

- `lib.rs` — re-exports plugins, types
- `resources.rs`
  - `GameStateResource(pub GameState)` — wraps `solitaire_core::GameState` directly (no `solitaire_data` layer yet)
  - `DragState { cards: Vec<u32>, origin_pile: PileType, cursor_offset: Vec2, origin_z: f32 }` (starts empty)
  - `SyncStatusResource(pub SyncStatus)` where `SyncStatus` is `Idle|Syncing|LastSynced(DateTime<Utc>)|Error(String)`
- `events.rs`
  - `MoveRequestEvent { from: PileType, to: PileType, count: usize }`
  - `DrawRequestEvent`
  - `UndoRequestEvent`
  - `NewGameRequestEvent { seed: Option<u64> }`
  - `StateChangedEvent`
  - `GameWonEvent { score: i32, time_seconds: u64 }`
  - `CardFlippedEvent(pub u32)`
  - `AchievementUnlockedEvent(pub AchievementRecord)` — placeholder, unused until Phase 5
- `game_plugin.rs` — `GamePlugin`:
  - On `Startup`: init `GameStateResource::new(system_time_seed, DrawMode::DrawOne)`
  - Systems: `handle_draw`, `handle_move`, `handle_undo`, `handle_new_game`
  - Each fires `StateChangedEvent` on success; `GameWonEvent` when `check_win()` flips to true
  - Errors: log via `tracing`, do not panic
- Register in [solitaire_app/src/main.rs](../../../solitaire_app/src/main.rs)

**Tests:** event-routing unit tests that drive `GamePlugin` in a headless `App::new()` and verify resource mutations.

**Exit:** `cargo test --workspace` green, `cargo clippy --workspace -- -D warnings` clean. Running the app still shows a blank window (no rendering yet), but pressing nothing crashes anything.

Commit: `feat(engine): add resources, events, and GamePlugin event routing`

---

### 3B — Layout + TablePlugin

**Modules:**

- `layout.rs` — pure function `compute_layout(window: Vec2) -> Layout`
  - `Layout { card_size: Vec2, pile_positions: HashMap<PileType, Vec2> }`
  - card_width = window.x / 9.0
  - card_height = card_width * 1.4
  - Row 1: stock, waste, [gap], 4 foundations
  - Row 2: 7 tableau columns below
- `LayoutResource(pub Layout)` — a Bevy resource
- `table_plugin.rs` — `TablePlugin`:
  - Spawns background rectangle (dark green `#0f5132`)
  - Spawns 13 `PileMarker` sprite entities for empty-pile placeholders
  - System `on_window_resized`: recompute `LayoutResource`, reposition pile markers

**Tests:** `compute_layout` at 800×600, 1280×800, 1920×1080 — all 13 piles within bounds, non-overlapping.

**Exit:** Window shows a green table with 13 translucent pile outlines that resize with the window.

Commit: `feat(engine): add layout, LayoutResource, and TablePlugin`

---

### 3C — CardPlugin rendering (procedural)

**Decision:** Phase 3 uses procedural cards (rounded white rectangle + rank/suit text). Real PNG assets can be slotted in later by replacing the sprite setup; API shape stays stable.

**Modules:**

- `card_plugin.rs` — `CardPlugin`:
  - Component `CardEntity { card_id: u32 }`
  - `StateChangedEvent` handler: sync entities with `GameStateResource` — spawn missing, despawn removed, reposition all
  - Position: `LayoutResource.pile_positions[pile] + Vec3::Z * stack_index`
  - Face-up: white rect + text of rank+suit glyph (red for hearts/diamonds, black for clubs/spades)
  - Face-down: blue rect with a subtle pattern overlay
  - No assets loaded — text uses Bevy's default font (or shipped system font if needed)

**Exit:** A freshly dealt game renders — stock (24 cards face-down), 7 tableau columns in standard 1/2/3/.../7 face-down + 1 face-up, empty foundations.

Commit: `feat(engine): add CardPlugin with procedural card rendering`

---

### 3D — Keyboard input & click-to-draw

**Modules:**

- `input_plugin.rs` — `InputPlugin`:
  - Keyboard system: `KeyCode::KeyU` → `UndoRequestEvent`, `KeyN` → `NewGameRequestEvent{seed: None}`, `KeyD` → `DrawRequestEvent`, `Escape` → pause-stub event
  - Mouse system: on left-click, if cursor over stock pile → `DrawRequestEvent`

**Exit:** Pressing D cycles stock↔waste on-screen; N deals a new game; U undoes.

Commit: `feat(engine): add InputPlugin with keyboard and stock-click`

---

### 3E — Drag & drop

**Modules:**

- Extend `input_plugin.rs` with drag systems:
  - `start_drag`: on left mouse-down, ray-hit the top card (or run of face-up cards) of a pile; populate `DragState`; elevate z
  - `follow_cursor`: while `DragState.cards` non-empty, move those entities to cursor position + per-card stack offset
  - `end_drag`: on mouse-up, determine target pile; early-validate with `can_place_on_tableau` / `can_place_on_foundation`; fire `MoveRequestEvent` (backend also validates)
  - On `MoveError` via `StateChangedEvent` non-emission: snap cards back with a short lerp (uses `CardAnim` from 3F)
- Multi-card tableau drag: grabbing card N pulls N..=top if all face-up

**Exit:** Full game playable with mouse. `GameWonEvent` fires on a win. No animations yet on invalid drop (just snap back instantly in 3E, smooth in 3F).

Commit: `feat(engine): add drag-and-drop input with multi-card tableau support`

---

### 3F — AnimationPlugin (polish)

**Modules:**

- `animation_plugin.rs` — `AnimationPlugin`:
  - Component `CardAnim { start: Vec3, target: Vec3, elapsed: f32, duration: f32 }` — linear lerp 0.15s for moves
  - Flip: `CardFlip { elapsed: f32, duration: f32, flips_to_face_up: bool }` — scale-X 1→0→1 over 0.2s, toggle `face_up` at midpoint, fire `CardFlippedEvent`
  - Win cascade: on `GameWonEvent`, iterate foundation cards and schedule `CardAnim` to random off-screen targets with staggered 0.05s starts
  - Toast component scaffold: egui popup placeholder, wired to `AchievementUnlockedEvent` (no content yet)

**Exit:** Valid moves animate smoothly; flipping a tableau card shows a flip; winning plays a cascade.

Commit: `feat(engine): add AnimationPlugin with slide, flip, and win cascade`

---

## Cross-cutting rules

- `solitaire_core` and `solitaire_sync` gain NO new dependencies.
- No `unwrap()` / `panic!()` in new Bevy systems — log errors via `tracing::warn!` and continue.
- `cargo test --workspace` and `cargo clippy --workspace -- -D warnings` green after EVERY sub-phase.
- Every commit follows `type(scope): description` convention.
- One `Plugin` per responsibility; cross-system communication is Events only.

---

## Open questions resolved

- **Procedural vs. sourced card art**: procedural for Phase 3.
- **`GameStateResource` layer**: wraps `solitaire_core::GameState` directly.
- **Phases 4–8 plugins** (Audio/UI/Achievement/Sync): not in Phase 3.
- **New-game seed**: system time when `None`, explicit when `Some(u64)`.
- **Commit cadence**: one per sub-phase.

---

## Risks

- Bevy 0.15 API drift from older tutorials — verify each API call as written.
- `bevy_egui` 0.30 may require slightly different system ordering than earlier versions — pin to workspace versions, don't downgrade.
- Procedural card text depends on Bevy's default font; if rendering is unreadable, drop in a `.ttf` to `assets/fonts/main.ttf` as a follow-up (still Phase 3, not 3F).
