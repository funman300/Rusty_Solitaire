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

- [ ] **Safe-area insets (top + bottom).** Query `WindowInsets` via JNI
  (or `winit::window::Window::safe_area()` if exposed in our pinned
  winit) and push HUD down by status-bar height + bottom UI up by
  nav-bar height. Likely lives in `solitaire_engine` layout system,
  gated `#[cfg(target_os = "android")]`.
- [ ] **Mobile HUD layout.** Wrap to two rows, drop redundant text, or
  move secondary actions (Help, Modes) into a hamburger / drawer.
  Current single-row layout requires desktop width.
- [ ] **Card-back asset not rendering.** Face-down cards show as red
  rectangles. Investigate: is `CardImageSet::back` resolving on
  Android? Is the texture being uploaded? Is the sampler wrong? Check
  AssetServer load path under Android — does it find the embedded /
  packaged texture?
- [ ] **Viewport overflow.** Leftmost foundation and rightmost tableau
  pile clipped. `LayoutResource` must recompute on Android using
  actual surface size (post-inset) instead of any desktop default
  width assumption.

## P1 — Touch UX

- [ ] **Suppress keyboard-hint labels on Android.** Gate the `Esc / A /
  N / []` accelerator chips behind `#[cfg(not(target_os = "android"))]`
  in the HUD spawn site(s).
- [ ] **Thumb-sized hit targets.** HUD buttons sized for mouse;
  Material guideline minimum is 44–48 dp. Increase button paddings
  on touch builds.
- [ ] **Portrait-first card spacing.** Stretch tableau piles vertically
  to fill height; reduce inter-pile gaps so 7 columns fit in 360 dp.
- [ ] **Double-tap auto-move visible feedback.** `handle_double_tap`
  exists since `395a322` — verify it triggers on hardware and add a
  brief source-card flash / highlight to confirm to the user.

## P2 — Polish

- [ ] **Drag responsiveness on touch.** Bevy default touch-to-mouse
  mapping can lag; confirm drag start threshold isn't too high for a
  finger.
- [ ] **Long-press menu.** Alternative to right-click (which doesn't
  exist on touch). Wire to the existing right-click-highlight system.
- [ ] **HUD typography.** Reduce text sizes for `Score:`, `Moves:`,
  timer so they fit cleanly in one row.
- [ ] **Orientation lock.** Set `android:screenOrientation="portrait"`
  in cargo-apk manifest (or design a landscape layout).

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
