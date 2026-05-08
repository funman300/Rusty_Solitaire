# Solitaire Quest — Session Handoff

**Last updated:** 2026-05-08 — **v0.21.4 cut and tagged at
`23ff62c`**, working tree clean, all post-tag work pushed to
origin.

v0.21.4 is a patch release with one through-line:
**replay-scrubbing accessibility**. The replay overlay used to be
pure-passive — start, watch, wait. v0.21.4 adds the scaffolding
for *navigating within* a replay: a WIN MOVE marker on the scrub
bar so the player can see at a glance where the winning move
sits, plus pause / resume / step controls (with a Space keyboard
accelerator) so they can stop on any move and inspect the board.
Also lands the additive `Replay::win_move_index: Option<usize>`
data field that makes the marker possible — serde-default so
older on-disk replays load with `None` and simply don't get a
marker (no schema bump).

Three commits on the B-2 replay screen-takeover redesign arc
land here. The remaining sub-pieces (screen-takeover layout,
move-log scroller, mini-tableau preview) share a layout-reflow
prerequisite the banner can't carry, so they're deferred to a
future cycle as a single multi-session arc.

Full v0.21.4 detail lives in `CHANGELOG.md` § [0.21.4]. This
file from here on focuses on what's *open* post-cut and how to
resume.

## Status at pause

- **HEAD locally:** see `git rev-parse HEAD`. The cut commit is
  `23ff62c`; any post-cut docs edits ride on top of that.
- **HEAD on origin:** matches local. v0.21.4 is fully on origin.
- **Working tree:** clean. No WIP outstanding.
- **`artwork/` directory:** still untracked. Intentional.
- **Build:** `cargo clippy --workspace --all-targets -- -D warnings`
  clean.
- **Tests:** **1243 passing / 0 failing** across the workspace
  (1228 in v0.21.4 + 4 from `fe68861`'s scrub-notch tests + 4
  from `d322abf`'s notch-label tests + 4 from `1873b3f`'s
  keybind-footer tests + 3 from `90e24d9`'s ESC-accelerator
  tests). Detail in `CHANGELOG.md` § [0.21.4] § Stats; post-cut
  delta tracked here.
- **Tags on origin:** `v0.9.0` through `v0.21.4`. v0.21.4 is on
  `23ff62c`; v0.21.3 stays on `3d92a91`; v0.21.2 stays on
  `f23df3b`; v0.21.1 stays on `daa655a`; v0.21.0 stays on
  `04f9bf9`; v0.20.0 stays on `41a009a`.
- **Tags on origin:** `v0.9.0` through `v0.21.3`. v0.21.3 is on
  `3d92a91`; v0.21.2 stays on `f23df3b`; v0.21.1 stays on
  `daa655a`; v0.21.0 stays on `04f9bf9`; v0.20.0 stays on
  `41a009a`.

## Since the v0.21.4 cut

- **`fe68861` — `feat(replay): add quarter-mark notches to scrub
  bar`.** First finite step toward B-2's screen-takeover layout.
  Five 1px vertical ticks at 0/25/50/75/100 % give the player
  visual anchor points without needing to mentally bisect the
  bar. Pure helper `scrub_notch_positions()` returns the fixed
  array; spawn loop lives next to the WIN MOVE marker spawn so
  the lifecycles match. Notches paint in `BORDER_SUBTLE`
  (matches unfilled-track colour) and rely on extending past the
  1px track (5px tall, anchored 2px above track top) for
  visibility — same trick the WIN MOVE marker uses. Spawned
  *after* the WIN MOVE marker so a notch and the marker landing
  on the same percentage paint the marker on top. Mirrors the
  notch ladder in `docs/ui-mockups/replay-overlay-mobile.html`.
  4 new tests; 1228 → 1232.
- **`d322abf` — `feat(replay): add percentage labels under
  scrub-bar notches`.** First **layout-changing** commit in B-2's
  screen-takeover arc. Banner height grew from 60 → 76 px to make
  room for a 16 px label row beneath the 1 px scrub track; the
  top row's `flex_grow: 1.0` still consumes the same 59 px so no
  ripples on existing content. Pure helper `scrub_notch_labels()`
  returns the fixed `["0%", "25%", "50%", "75%", "100%"]` array,
  paired index-for-index with `scrub_notch_positions()`. Spawn
  loop applies an "endpoints flush, middle three percent-anchored"
  positioning pattern (Bevy 0.18 UI has no clean
  `translate-x: -50%` primitive, so endpoints flush against
  banner edges and middle three accept slight right-of-notch
  offset). Label colour is `TEXT_SECONDARY` (mockup's
  `BORDER_SUBTLE` reads as too low-contrast at 12 px against
  `BG_ELEVATED_HI`). 4 new tests; 1232 → 1236.
- **`1873b3f` — `feat(replay): add keybind-hint footer to
  overlay banner`.** Second layout-changing commit in B-2's arc.
  Banner grew from 76 → 92 px to fit a 16 px footer row at the
  bottom edge with a vim-style mode line on the left
  (`▌ NORMAL │ replay`) and a keybind-hint on the right
  (`[SPACE] pause/resume`). Surfaces the existing Space
  accelerator visually so CLAUDE.md §3.3's UI-first contract
  holds for keyboard accelerators too. Footer lists *only
  wired* keybinds — future commits that wire ESC for stop or
  ← / → for prev/next will extend the right-hand text in
  lockstep. Two pure helpers (`keybind_footer_mode_text`,
  `keybind_footer_hint_text`) keep the static text testable;
  shared `font_handle_for_labels` clone covers both label and
  footer text spawns. 1px top border in `BORDER_SUBTLE`
  separates the footer from the labels row. 4 new tests;
  1236 → 1240.
- **`90e24d9` — `feat(replay): wire ESC accelerator for stop,
  gate pause modal`.** ESC during an active replay now stops it
  (mirrors the Stop button click). New `handle_stop_keyboard`
  system in `replay_overlay.rs` parallels `handle_pause_keyboard`
  in shape. Cross-plugin coordination via `pause_plugin::toggle_pause`:
  added a fourth defer-if check
  (`replay_state.is_some_and(|s| s.is_playing())`) right after
  `other_modal_scrims` and before `selection`. Symmetric to the
  existing modal-stack defer pattern. Footer hint extended from
  `[SPACE] pause/resume` → `[SPACE] pause/resume · [ESC] stop`
  in lockstep with the wiring; the only-wired-keybinds
  discipline holds. 3 new tests + 1 updated helper-pin test;
  1240 → 1243.

Banner geometry is now mutable — every prior B-2 commit fit
inside fixed 60 px space, but the notch-labels commit
established the "grow the container, add a new flex-column
child" precedent and the keybind-footer commit applied it
again. The next sub-pieces need significantly more vertical
room and follow the same shape.

Next finite step on B-2: choices are
1. **Wire ← / → for prev/next move** — needs a "step
   backwards" path in `replay_playback`, which currently only
   supports forward stepping. Backwards stepping is non-trivial:
   the `Replay` carries the move list but no intermediate game
   states, so rewinding means either replaying-from-start to
   `cursor - 1` or hooking into the game's undo system. New
   state plumbing either way. Footer hint would extend to
   `[SPACE] pause/resume · [ESC] stop · [← →] step` in
   lockstep with the wiring.
2. **Move-log scroller / mini-tableau preview** — both need
   a much larger banner-height grow (effectively the takeover
   container itself). Bigger arcs; the natural place to land
   the layout reflow that turns the banner into a takeover.
3. **HC-mode coverage for the new banner pieces** — labels
   and footer texts currently use `TEXT_SECONDARY`; under HC
   mode they should bump to `TEXT_PRIMARY` (or use the
   `HighContrastBorder` marker pattern for the 1 px borders).
   Small accessibility polish.

Recommended order: option 3 (HC polish) is the smallest next
step and keeps the cadence; option 1 (← / →) is the right
medium-scope next-feature; option 2 is the multi-session arc
that closes B-2.

## Open punch list

### Phase Android (build + persistence shipped; runtime gaps remain)

- **APK launch verification on AVD / device.** `adb install` then
  `adb logcat` against the `bevy_test` AVD or an x86_64 device.
  The build works and persistence is wired, but no end-to-end
  device run has been logged. Shakes out runtime bugs the build +
  unit tests can't catch.
- **JNI ClipboardManager bridge.** Replaces the Android stub for
  the Stats "Copy share link" toast. `arboard` doesn't ship an
  Android backend; small custom JNI call.
- **Android Keystore for credentials.** `keyring` is target-gated
  to a stub returning `KeychainUnavailable`; replace with Android
  Keystore via JNI when sync auth ships on mobile.
- **Google Play Games (gpgs) integration.** Listed as a
  Phase-Android target since Phase 1; now unblocked by the build
  target.
- **Cosmetic `cargo apk build --lib` workaround.** Post-sign
  panic doesn't affect the APK on disk but produces noisy stderr.
  Either upstream a cargo-apk fix or document `--lib` as
  canonical in the runbook.

### Visual-identity follow-ups (post-v0.21.0)

The visual-identity arc is effectively complete: token system,
chrome migration, splash boot screen, replay-overlay banner,
card-face artwork (both rendering paths), and the `ACCENT_PRIMARY`
palette refresh all shipped in v0.20.0 + v0.21.0. What stays open:

- **Replay-overlay screen-takeover redesign.** The full mockup
  (`docs/ui-mockups/replay-overlay-mobile.html`) calls for a
  mini-tableau preview, playback controls, move-log scroll, and
  a WIN MOVE marker on the scrub bar. Banner-local pieces all
  shipped in v0.21.0 (`c84d9f4` + `6204db8` + `54005d5` +
  `e080b49`); the floating MOVE chip above the focused card
  shipped in v0.21.2 (`2fb2d63`). The WIN MOVE scrub-bar marker
  shipped post-v0.21.3 in `ab857bb` (data field) + `52befa6`
  (UI). Playback controls (pause / resume / step + Space
  accelerator) shipped post-v0.21.3 in `fbe48ac`. Quarter-mark
  scrub notches (5 ticks at 0/25/50/75/100 %) shipped
  post-v0.21.4 in `fe68861` — first decoration step toward the
  takeover layout. Percentage labels under each notch shipped
  post-v0.21.4 in `d322abf` — first **layout-changing** commit
  (banner 60 → 76 px to make room for a 16 px label row).
  Keybind-hint footer (vim-style mode line + `[SPACE]
  pause/resume`) shipped post-v0.21.4 in `1873b3f` (banner
  76 → 92 px). ESC accelerator wiring (with cross-plugin gate
  in `pause_plugin::toggle_pause`) shipped post-v0.21.4 in
  `90e24d9`; footer hint extended to
  `[SPACE] pause/resume · [ESC] stop` in lockstep. Banner
  geometry is now mutable. What still needs to land: ← / →
  scrub keys (needs new backwards-step path), HC-mode
  coverage for the new banner pieces (labels + footer
  texts), then the bigger pieces — a move-log scroller and a
  mini-tableau preview — both screen-takeover-only pieces
  that need a much larger banner height grow (effectively the
  takeover container itself). Multi-session.
- *Floating `MOVE N/M` chip above the focused card during
  playback — closed 2026-05-08 by `2fb2d63`.* World-space
  `Text2d` entity sibling to the banner overlay; uses the same
  `LayoutResource` pile coordinates so it survives window
  resizes without UI/camera math.
- *Toast Warning variant wiring — closed 2026-05-08 by `279e23d`.*
  Daily-challenge-expiry toast fires once per `daily.date` when
  within 30 min of UTC midnight reset and today is incomplete.
  `ToastVariant` is now fully load-bearing (every variant has at
  least one real driver). Future Warning drivers can either reuse
  the generic `WarningToastEvent(String)` carrier or add their
  own domain message + `animation_plugin` handler.
- *Toast Error variant wiring — closed 2026-05-08 by `68d50b5`.*
  `MoveRejectedEvent` now fires a 2-second pink-bordered
  "Invalid move" toast as the third leg of the
  audio + visual + text rejection-feedback stool.
- *High-contrast accessibility mode — closed 2026-05-08 by
  `c5787c6` + `07e0357` (engine + UI) + v0.21.2's HC chrome
  rollout (`c9af1ea` + `d87761d` + `ec804d5`) + post-cut
  dynamic-paint rollout (`c153363`).* Card text rendering plus
  8 static-border chrome surfaces (modal scaffold, tooltip,
  onboarding key chips, help panel key chips, stats panel
  cells, home Level/XP/Score row, home mode buttons, home
  mode-hotkey chips, 4 settings panel surfaces) all boost
  borders to `BORDER_SUBTLE_HC` under HC via the
  `HighContrastBorder` marker. The previously-carved-out
  dynamic-paint sites are now also covered: HUD action buttons
  and modal buttons take the same marker (their paint cycles
  only mutate `BackgroundColor`, so no race); the radial menu
  rim folds HC into its per-frame spawn via
  `radial_rim_outline` so the focused rim boosts to
  `BORDER_SUBTLE_HC` under HC (preserving focused-vs-resting
  hierarchy that naive marker substitution would invert).
- *Reduced-motion mode — closed 2026-05-08 by `c5787c6` +
  v0.21.2's `ed152e2`.* `effective_slide_secs` forces 0 on
  card animations; `pulse_splash_cursor` skips the per-frame
  pulse multiplier; `spawn_splash` skips the scanline overlay
  entirely. Future scope: gate any future card-lift z-bump
  animation, warning-chip pulse (when one materialises).

### Carried forward from v0.19.0

- *App icon round — closed 2026-05-08 by `3eb3a26` + `716a025`.*
  Runtime `Window::icon` wired (Linux/macOS/Windows); 9-size
  PNG hierarchy at `assets/icon/icon_<size>.png` covers Linux
  hicolor + downstream `.icns`/`.ico` packaging needs. The
  `.ico` and `.icns` bundle-format files themselves are *not*
  generated — both would need new crate deps (`ico` and
  `icns` respectively) and only matter at app-bundle time
  (cargo-bundle / packaging), not at `cargo run`. Open if the
  project later ships as a packaged macOS / Windows app.

### Other small candidates

- **Prev/Next selector chips spawn site.** v0.19.0's `9b065e5`
  noted Prev/Next markers exist in `stats_plugin` but no spawn
  site renders them today — the Shareable badge therefore lands
  on the single-replay caption. If/when Prev/Next is plumbed,
  the badge will need to follow.
- **Toast queue / immediate unification.** The two toast paths
  (`spawn_queued_toast` for `InfoToastEvent` queue; `spawn_toast`
  for fire-and-forget) now share visual treatment but remain
  separate functions because they serve different temporal
  needs (sequential vs. parallel). If overlap becomes a UX
  issue, merge into one queue with priority lanes.

### Process notes

- **The desktop-adaptation spec is the canonical reference for
  geometry decisions** when porting any future plugin. Read
  `docs/ui-mockups/desktop-adaptation.md` first; apply the
  universal rules to every surface; consult the per-screen
  table for the priority surfaces. The 9 missing-plugin screens
  (splash now ported; eight remaining) inherit the universal
  rules without dedicated guidance.
- **Stitch `generate_variants` is unreliable for layout-only
  adaptation prompts** as of 2026-05-07. The first call timed
  out and no variant ever landed in `list_screens`. If a future
  session wants visual desktop mockups, prefer
  `generate_screen_from_text` with a fresh narrow prompt per
  screen rather than `generate_variants` against existing
  mobile screens.
- **Token-port pattern.** v0.20.0's chrome-migration commits
  set a reusable shape for "centralised design system applied
  across N plugins":
  1. Constants module (`ui_theme.rs`) is the source of truth.
  2. Const sites that can't call `Alpha::with_alpha` (not yet
     `const` on stable) use a literal RGB matching the token,
     with a unit test pinning the RGB to the token (e.g.
     `MARKER_VALID`, `HINT_PILE_HIGHLIGHT_COLOUR`,
     `RIGHT_CLICK_HIGHLIGHT_COLOUR`).
  3. Cross-plugin duplication (e.g. `MARKER_DEFAULT` ↔
     `PILE_MARKER_DEFAULT_COLOUR`) collapses to a single
     promoted const re-exported from one plugin and imported
     by the other — replaces "kept in sync" doc comments with a
     compile-time invariant.
  4. Domain colours (suit pips, card faces, lerp helpers) stay
     as literals with a comment naming the rationale; only UI
     chrome routes through tokens.
- **`SplashFadable` scaffolding pattern** (introduced in
  `cacb19c`). Any future overlay that needs to fade `N >> 3`
  elements together should follow the same shape: one tiny
  marker carrying the full-alpha base colour, one global query
  that lerps every marker's alpha each frame, no per-element
  query plumbing. Cleanly outscales the `Without<X>, Without<Y>`
  query exclusion pattern that the old splash was hitting at
  three siblings.

### Canonical remote

`github.com/funman300/Rusty_Solitaire` is the canonical repo.
Always push there. As of v0.21.0 origin matches local; the next
push happens when post-cut work accumulates and is ready to roll
into a v0.21.1 / v0.22.0 cut.

### Design direction (Terminal — base16-eighties)

- **Tone:** retro-terminal / synthwave — flat depth (no box-shadows),
  monospaced-forward typography (JetBrains Mono / FiraMono), tight
  16 px edge margins, 8 px card radius.
- **Palette:** near-black surface ramp (`#151515` / `#202020` /
  `#2a2a2a` / `#353535`), brick-red primary CTA (`#a54242` —
  swapped from cyan `#6fc2ef` in v0.21.0 commit `a292a7e`), lime
  success (`#acc267`), gold warning (`#ddb26f`), pink error /
  suit-red (`#fb9fb1`), lavender celebration (`#e1a3ee`), teal
  info (`#12cfc0`).
- **Two-color suits.** Red = `#fb9fb1`, black = `#d0d0d0`.
  Outlined glyphs for diamonds & clubs are *always on*; the
  Settings "color-blind mode" toggle swaps red → lime `#acc267`
  (was red → cyan pre-v0.21.0; lime is the next-best non-red
  base16-eighties accent now that the primary itself is red).
- **Card glyphs render upright in both corners** — no 180°
  inverted-corner-indicator rotation. Single-orientation
  digital play doesn't benefit from the traditional flip-
  readback convention. `design-system.md` § Game Cards
  documents this deliberate deviation.

## Resume prompt

```
You are a senior Rust + Bevy developer working on Solitaire Quest.
Working directory: <Rusty_Solitaire clone path on this machine>.
Branch: master. v0.21.4 is tagged at 23ff62c (cut 2026-05-08, a
patch release rolling up replay-scrubbing accessibility: WIN MOVE
marker on the scrub bar, pause / resume / step playback controls
with a Space keyboard accelerator, and the additive
`Replay::win_move_index: Option<usize>` data field that makes the
marker possible). v0.21.3 stays at 3d92a91, v0.21.2 at f23df3b,
v0.21.1 at daa655a, v0.21.0 at 04f9bf9. Working tree clean. See
CHANGELOG.md § [0.21.4] for full detail.

State: HEAD locally — see `git rev-parse HEAD`. Post-cut HEAD is
`90e24d9` (four carved-out commits on top of v0.21.4 — scrub-bar
notches `fe68861`, notch labels `d322abf`, keybind-hint footer
`1873b3f`, ESC accelerator + pause-modal gate `90e24d9`). All
workspace tests pass (1243; check with `cargo test --workspace`),
clippy clean.

READ FIRST (in order, before doing anything):
  1. SESSION_HANDOFF.md  — this file
  2. CHANGELOG.md        — [0.21.4] section is the most recent cut
  3. CLAUDE.md           — unified-3.0 rule set
  4. CLAUDE_SPEC.md      — formal architecture spec
  5. ARCHITECTURE.md     — crate responsibilities + data flow
  6. docs/ui-mockups/    — design system + 24-mockup library +
                           desktop-adaptation.md (the rules-based
                           companion to the mockups; read this
                           before any plugin port)
  7. docs/android/*      — Android setup + build runbook
  8. ~/.claude/projects/<this-project>/memory/MEMORY.md
                         — saved feedback / project context
                           (machine-local; may be missing on a
                           fresh machine)

DECISION TO ASK THE PLAYER FIRST:
  A. APK launch verification on AVD / device — `adb install` +
     `adb logcat` to shake out runtime bugs the build / unit
     tests can't catch. Likely surfaces JNI ClipboardManager
     and Android Keystore stubs that need real bridges. Larger
     scope; needs an Android device or emulator running.
  B. Replay-overlay screen-takeover redesign — multi-session
     work. Three sub-pieces shipped in v0.21.4: WIN MOVE
     marker (data field + UI) and pause / step / Space
     playback controls. The smaller floating-MOVE-chip piece
     shipped in v0.21.2 (`2fb2d63`). Post-v0.21.4: quarter-
     mark scrub notches shipped in `fe68861` (5 ticks at
     0/25/50/75/100 %); percentage labels under each notch
     shipped in `d322abf` (banner 60 → 76 px — first layout
     change); keybind-hint footer shipped in `1873b3f`
     (banner 76 → 92 px — vim-style mode line + `[SPACE]
     pause/resume`); ESC accelerator wiring shipped in
     `90e24d9` (cross-plugin gate in `pause_plugin`; footer
     extended to `[SPACE] pause/resume · [ESC] stop`).
     Banner geometry is now mutable. Natural next finite
     steps:
     1. **HC-mode coverage** for the new banner pieces
        (labels + footer texts use `TEXT_SECONDARY`; under
        HC they should bump). Smallest next step.
     2. **Wire ← / → for prev/next move.** Needs a
        backwards-step path in `replay_playback` — either
        replay-from-start or hook into the game's undo
        system. Medium-scope next-feature.
     3. **Move-log scroller / mini-tableau preview** — both
        need a much larger banner-height grow (effectively
        the takeover container itself). Multi-session arcs
        that close B-2.
     Mockup at `docs/ui-mockups/replay-overlay-mobile.html`.
  C. Phase 8 (sync) — local storage scaffolding, self-hosted
     Axum server, `SolitaireServerClient` impl, GPGS stub
     wired into Settings. The biggest open arc by scope; rolls
     up several Phase Android dependencies (Keystore,
     ClipboardManager).

WORKFLOW NOTES:
  - Use the system git config (already correct).
  - When attributing playtester feedback in commits/docs, use
    "Quat" not "Rhys" (saved feedback memory).
  - Sub-agents stage + verify only; orchestrator commits.
  - Every commit must pass build / clippy / test before pushing.
  - Push to GitHub (origin) — gh auth setup-git wired on
    primary dev box; verify on laptop before first push.
  - Token-port pattern: when migrating tokens, walk every
    concrete artifact downstream of the token (PNG textures,
    embedded SVGs, hardcoded literals, comment color names),
    not just the token name. v0.21.0 surfaced three "the
    migration walked past this" follow-ups that all matched
    this shape — codified here so future similar work can
    pattern-match instead of rediscovering.
  - Doc-vs-implementation drift pattern: v0.21.1's pile-marker
    visibility fix (`4d48cad`) implemented an invariant that
    had been declared in a module doc comment but was never
    enforced in code. When future work touches a module with
    a "this does X" doc comment, verify the code actually does
    X and add a test if not. Two layers, two checks.

OPEN AT THE START: ask which of A–C. Don't pick unilaterally.
Note: every remaining option is multi-session by nature (A is
gated on Android tooling, B and C are explicitly multi-session
arcs). A fresh session is a better fit for any of them than the
tail of a long working stretch.
```
