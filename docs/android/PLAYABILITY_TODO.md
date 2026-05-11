# Android Playability TODO

**Started:** 2026-05-10 — first hardware screenshot of v0.22.3 APK
running on a real device showed the desktop HUD projected onto a
360 dp portrait viewport with no mobile adaptation. This list
tracks the work needed to make the APK genuinely playable, not
just "boots without crashing."

**Context:** v0.22.3 (signed release APK) builds and launches.
JNI bridges (clipboard, keystore) compile but are untested on
hardware. The work below is UI/UX port work — no architectural
rewrites required.

---

## Reading from the v0.22.3 screenshot

| Region | Observation |
|--------|-------------|
| Top ~5 % | System bar (clock, signal, battery) overlapped by game HUD — no safe-area inset |
| HUD text row | `Score:0 Pause Esc Help A Modes [] New_Game N Moves:0 0:08` all overlapping — desktop layout crammed into 360 dp |
| Keyboard hints | `Esc`, `A`, `[]`, `N` shown next to buttons — meaningless on touch |
| Foundations row | Leftmost foundation (♥) clipped left; rightmost tableau column (♠ 4) clipped right |
| Card backs | Face-down cards render as solid red squares, not back-art texture |
| Vertical use | Cards occupy top ~30 % only; bottom 70 % empty black — no portrait-aware layout |
| Bottom edge | No accommodation for Android gesture / home-indicator area |

---

## P0 — Blocking playability

- [x] **Safe-area insets (top + bottom).** *Closed 2026-05-10 by
  `b9aa262`.* `SafeAreaInsets` resource + `SafeAreaInsetsPlugin`
  query `WindowInsets.getInsets(systemBars())` via JNI on Android;
  HUD anchors carry `SafeAreaAnchoredTop { base_top }` and the
  change-detection fix-up system re-applies `base_top + insets.top`
  whenever the resource updates. Bottom inset is captured but not
  yet consumed (waits for bottom-anchored UI).
- [x] **Mobile HUD layout.** *Closed 2026-05-10.* Both the left HUD
  column and the right action button row are now capped at
  `max_width: 50 %` and the button row + tier-row child Nodes carry
  `flex_wrap: Wrap`. On a 360 dp viewport the 6-button row breaks
  to multiple lines (right-justified) and the tier rows wrap
  individually instead of overflowing into the action column. On
  desktop (≥ 1280 px) the 50 % cap is wider than any natural row
  width so the existing single-line layout is unchanged.
- [x] **Card-back asset not rendering.** *Closed 2026-05-10 by
  `fcc7337`.* `AssetPlugin::file_path = "../assets"` was set
  unconditionally to fix the desktop `cargo run -p solitaire_app`
  CWD relativity, but on Android cargo-apk packages the same
  directory into the APK at `assets/` and Bevy's
  AndroidAssetReader is already rooted there — prepending `../`
  walked the reader out of the APK assets root and every load
  failed silently. The face-down branch then fell through to the
  `card_back_colour(0)` solid-red brick fallback. Gated the
  override behind `#[cfg(not(target_os = "android"))]`.
- [x] **Viewport overflow.** *Closed 2026-05-10.* `compute_layout`
  was clamping the input window up to `MIN_WINDOW = 800 × 600`,
  so a 360 dp phone got laid out as if it were 800-wide and the
  outer piles fell outside the actual viewport. Lowered the floor
  to 320 × 400 (below the smallest reasonable phone) so real
  Android resolutions flow through without clamping, while keeping
  a sentinel to guard against degenerate / startup-zero windows.
  New regression test `phone_portrait_layout_fits_horizontally`
  asserts all 13 piles fit a 360 × 800 viewport.

## P1 — Touch UX

- [x] **Suppress keyboard-hint labels on Android.** *Closed
  2026-05-10.* `spawn_action_button` now nulls the `hotkey`
  argument on Android via a `#[cfg(target_os = "android")]` rebind,
  so the U / Esc / F1 / N chips next to the action row labels
  disappear on touch builds. Other hint sites (onboarding panel,
  pause-modal `Esc` hint, mode-card hotkey chips on the home
  screen, replay overlay footer, modal toggle hints in
  profile/stats/leaderboard/settings, help screen) survive — they
  live behind navigation and a touch user reaches them less often.
  Track as a P3 sweep when more screens are audited on hardware.
- [x] **Thumb-sized hit targets.** *Closed 2026-05-10.* Action
  button Node carries `min_width: Val::Px(48.0), min_height:
  Val::Px(48.0)` — meets Material's 48 dp baseline on touch and is
  a no-op for buttons whose content already exceeds 48 px in
  either axis. Applied universally rather than cfg-gated since
  Material's guideline applies to all input modes. Cards, pile
  markers, modal close buttons not yet audited — track as P3 if
  they fall below threshold on hardware.
- [x] **Portrait-first card spacing.** *Closed 2026-05-11.*
  `compute_layout` now derives an adaptive `tableau_fan_frac` from the
  available vertical space below the tableau row. On height-limited
  (desktop) windows the formula returns ≈ 0.25 and the clamp keeps the
  existing behaviour. On width-limited (portrait phone) windows — where
  card size is constrained by the 9-column horizontal packing — the fan
  fraction expands to fill the viewport (≈ 0.84 at 360 × 800 dp).
  `tableau_facedown_fan_frac` scales proportionally. Both values live in
  the `Layout` struct; `card_plugin::card_positions` and
  `input_plugin::card_position` / `pile_drop_rect` read from the struct
  so rendering and hit-testing stay in sync across viewport sizes.
- [x] **Double-tap auto-move visible feedback.** *Closed 2026-05-11.*
  On a recognised double-tap (priority 1 single-card or priority 2
  stack move), the moved card(s) receive a 0.35 s lime flash
  (`STATE_SUCCESS` tint + `HintHighlight { remaining: 0.35 }`) before
  the move request is written. The flash persists through the card
  animation and is cleaned up by the existing `tick_hint_highlight`
  system. Hardware trigger-verification remains a manual step — connect
  AVD or device and confirm two rapid `TouchPhase::Ended` events within
  0.5 s produce the lime flash.

## P2 — Polish

- [x] **Drag responsiveness on touch.** *Closed 2026-05-11.*
  Two code-side improvements shipped; final feel confirmation still needs
  hardware:
  1. `start_drag` (mouse path) now bails out when a touch is just-pressed
     (`Touches::iter_just_pressed()`), ensuring `touch_start_drag` always
     owns the drag state on touch-screen devices — including Bevy/Winit
     versions that simulate `MouseButton::Left` from the primary touch.
  2. Mobile drag commit threshold lowered 10 px → 8 px, matching Android's
     `ViewConfiguration.getScaledTouchSlop()` spec. Smaller threshold →
     smaller snap-on-commit and faster perceived response.
  **Remaining:** connect AVD or device and verify drag feels responsive
  with no stutter; tune threshold further if needed.
- [x] **Long-press menu.** *Closed 2026-05-11.* New system
  `radial_open_on_long_press` in `radial_menu.rs` counts up while a
  touch is held (`drag.active_touch_id.is_some() && !drag.committed`)
  and opens `RightClickRadialState::Active` after 0.5 s — the same
  state the right-click path uses. Existing radial infrastructure
  then handles everything:
  - `radial_track_cursor` extended to fall back to the first active
    touch when no cursor position is available, so sliding the held
    finger moves the hover ring.
  - `radial_handle_release_or_cancel` extended to confirm/cancel on
    `Touches::iter_just_released()` in addition to right-mouse release.
  - `handle_double_tap` skips when the radial is active (guards a
    narrow edge case where the finger lifts at exactly the same frame
    the 0.5 s threshold fires).
  Hardware verification needed: confirm the 0.5 s hold feel, verify
  sliding to a destination and lifting confirms the move.
- [x] **HUD typography.** *Closed 2026-05-11.* New system
  `update_hud_typography` fires on `WindowResized` and adjusts Tier-1
  font sizes based on viewport width. Below 480 logical px: Score
  `TYPE_HEADLINE` (26) → `TYPE_BODY_LG` (18), Moves/Timer
  `TYPE_BODY_LG` (18) → `TYPE_CAPTION` (11), so all three items fit
  in the 180 dp HUD column on a 360 dp phone. At ≥ 480 px the
  original sizes are restored — desktop/tablet layout unchanged.
  `add_message::<WindowResized>()` added defensively to `HudPlugin`
  so the system works under `MinimalPlugins` in tests.
- [x] **Orientation lock.** *Closed 2026-05-11.* Added
  `[package.metadata.android.application.activity]` section to
  `solitaire_app/Cargo.toml` with `orientation = "portrait"`.
  cargo-apk/ndk-build maps this to `android:screenOrientation="portrait"`
  in the generated `AndroidManifest.xml`. Remove (or add a landscape
  layout) before enabling auto-rotate.

## P3 — Asset density

- [ ] **Density-aware card scaling.** Currently single texture size; on
  a high-DPI phone the cards look small. Scale by
  `Window::scale_factor()` or ship multiple PNG sizes.
- [ ] **App-icon density buckets.** Nine sizes already exist in
  `assets/icon/`; verify the manifest references them so Android's
  launcher picks the right one.

## P4 — Stability / runtime

- [ ] **B0004 ECS hierarchy warnings.** Flagged in
  `SESSION_HANDOFF.md` after APK launch verification — investigate
  whether they cause gameplay bugs on hardware vs. AVD.
- [ ] **AVD functional tests for JNI bridges.** Clipboard (`2c822ba`)
  and Keystore (`f281425`) shipped but never tested on real device
  or AVD.

---

## Notes / decisions

* This list is screenshot-driven; expect more items to surface once
  P0 unblocks actually moving cards on hardware.
* The pattern across all the bugs is "no one ran the relevant code
  path on Android yet." The hard work — Bevy 0.18 on Android,
  JNI bridges, signed CI builds — is done. What's left is a
  coordinated pass of `#[cfg(target_os = "android")]` gates plus
  making `LayoutResource` query the real surface size.
* Where possible, prefer responsive layout (query window size) over
  branching `#[cfg]` blocks. Branches are fine for input methods
  (touch vs. mouse) but not for screen geometry — a foldable or
  desktop window of equivalent size should look the same.
