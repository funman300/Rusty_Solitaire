# Solitaire Quest — Session Handoff

**Last updated:** 2026-05-08 — v0.20.0 cut and tagged at `41a009a`,
all post-cut commits pushed to origin (HEAD = `dd101b3`), working
tree clean.
The cut itself shipped two through-lines: a full **Terminal visual-
identity port** (token system, modal scaffold, gameplay-feedback,
toasts, table / card chrome, splash cursor) and the **Android
persistence shim** that closes the `dirs::data_dir() = None` pitfall
flagged in CLAUDE.md §10. Since the cut, the post-tag work split
into two arcs: (1) splash boot-screen port + replay-overlay
banner enrichments + desktop-adaptation spec — closing Resume-prompt
Options B and C (see "Since the v0.20.0 cut" entries below); and
(2) **the card-face artwork regeneration arc — Option D, closed
2026-05-08** — full Terminal cards rendering on every face, plus
three follow-up fixes that surfaced during sign-off (default-theme
SVG override, table backgrounds, top-bar overlap), plus a
glyph-orientation tweak (no 180° inverted-corner rotation).

## Status at pause

- **HEAD locally:** see `git rev-parse HEAD`. Most recent narrative
  entry below names the latest substantive commit; this status line
  intentionally avoids hard-coding the SHA so a docs-only edit
  doesn't immediately stale the handoff.
- **HEAD on origin:** matches local. All post-cut commits pushed
  through `dd101b3`. Decide whether to roll the post-tag work
  into v0.20.1 / v0.21.0-candidates the next time a release is cut.
- **Working tree:** clean. No WIP outstanding.
- **`artwork/` directory:** still untracked. Intentional.
- **Build:** `cargo clippy --workspace --all-targets -- -D warnings`
  clean.
- **Tests:** **1184 passing / 0 failing** across the workspace.
  Net delta from the 1180 baseline: splash polish added two
  (`build_scanline_image_has_expected_2x2_rgba_bytes`,
  `scanline_overlay_spawns_and_fades_with_splash`); the
  card-face migration added one (`card_face_svg_pin` integration
  test) and consolidated two (`face_colour` CBM tests folded
  into `text_colour` CBM tests, net −2 then +1 from pin);
  call it +4 net.
- **Tags on origin:** `v0.9.0` through `v0.20.0`. v0.20.0 is on
  `41a009a`.

## Since the v0.20.0 cut (un-pushed)

### `39b8496` `docs(ui): add Terminal desktop-adaptation spec`

`docs/ui-mockups/desktop-adaptation.md` — 283 lines covering
viewport assumptions, seven universal adaptation rules, and per-
screen geometry rules for the priority surfaces (Game Table, Win
Summary, Settings, Help, Pause, Home, Splash, Stats, and the
modal-pattern screens Profile / Achievements / Theme Picker /
Daily Challenge). Closes the spec gap — 23 of 24 mockups were
mobile-only, but the v0.20.0 token-port pass was already layout-
agnostic so nothing shipped broken. The spec matters for *next*
ports.

**Why rules > visual mockups for this gap:** Stitch's
`generate_variants` API timed out on the layout-only adaptation
prompt (server-side flake, not a prompt-shape issue — confirmed
by polling `list_screens` with no new variant landing). A markdown
rules file applies to every screen including the 9 missing-plugin
surfaces (splash, challenge, time-attack, weekly-goals,
leaderboard, sync, level-up, replay-overlay, radial-menu) that
aren't in the Stitch project at all. It's also referenceable from
code comments and commit messages without loading an image.

### `cacb19c` `feat(engine): port the splash to the Terminal boot-screen treatment`

Implements the full mockup-spec splash from
`docs/ui-mockups/splash-mobile.html` plus the desktop adaptation
rules:

- **Header**: cursor block (96 px `▌`), wordmark ("Solitaire
  Quest"), 192 px divider, "TERMINAL EDITION" subtitle.
- **Boot log**: three ✓ check rows (`assets loaded`,
  `theme: terminal`, `progress restored`) + a `▌ ready_` line.
  Capped at 480 px width on desktop (else 70 % viewport).
- **Progress bar**: 1 px track (`BORDER_SUBTLE`) with a 100 %-
  width cyan (`ACCENT_PRIMARY`) fill + `DONE · 247 ASSETS`
  caption. Capped at 720 px on desktop (else 80 %).
- **Footer**: `BASE16-EIGHTIES` label, eight palette swatches
  (12 × 12 px each — one per named token in the design system),
  version line.

**Refactored the alpha-fade scaffold** from per-marker queries
(`SplashTitle` / `SplashSubtitle` / `SplashCursor`) to a single
`SplashFadable { base_color: Color }` + `SplashFadableBg`
variant. ~15 fadable elements share one global query each;
adding more is one component-attach, not three new query types.

**Skipped, with rationale captured in the commit:**
- Scanline overlay (needs a tiled-pattern asset or custom shader).
  *Open in "Visual-identity follow-ups" below.*
- Pulsing cursor on the "ready_" line (would fight the global
  fade timeline). *Open in "Visual-identity follow-ups" below.*
- "RUSTY SOLITAIRE" wordmark from the mockup (the actual product
  is "Solitaire Quest"; the mockup leaked the repo name). *Closed
  — the in-engine wordmark stays "Solitaire Quest".*

### `c84d9f4` `feat(engine): scrub fill bar + per-frame updater for replay overlay`

Closes the WIP described in the prior handoff. Adds the 1 px cyan
scrub bar called for in `docs/ui-mockups/replay-overlay-mobile.html`:
a track in `BORDER_SUBTLE` spans the bottom edge of the banner and
the cyan `ACCENT_PRIMARY` fill mirrors `cursor / total` via a new
`ReplayOverlayScrubFill` component + `update_scrub_fill` system.
The pure `scrub_pct` helper is shared between the spawn path
(initial fill width) and the per-frame updater so the first paint
already reflects state instead of popping `0 → cursor` on the
first tick — same shape as the existing `format_progress` /
`update_progress_text` split. Two new tests cover the four corners
of `scrub_pct` and an end-to-end drive of `ReplayPlaybackState`
asserting `Node.width` on the unique scrub-fill entity. Same
change-detection guard as the text updaters, so an idle replay
leaves the node untouched.

Header text treatment (closed by `6204db8` immediately below),
move-log scroll, MOVE chip, and WIN MOVE callout from the same
mockup are still open — separate commits.

### `6204db8` `feat(engine): port replay banner label to ▌ cursor-block treatment`

Aligns the replay overlay's headline with the splash boot-screen
idiom landed in `cacb19c`: `Replay` → `▌ replay` and
`Replay complete` → `▌ replay complete`. The cursor block (`▌`,
U+258C) prefixed to a lowercased label reads as a Terminal output
line rather than a generic UI title, tightening the family
resemblance between the two top-level overlay surfaces. Pure
text-content change; no behavioural shift, no new components, no
new systems.

**Mockup deviation (intentional):** the source mockup string in
`docs/ui-mockups/replay-overlay-mobile.html` is `▌replay.tsx`. The
`.tsx` is a prototyping leak — Stitch renders in React, so the
mockup author reached for a familiar filename — and was dropped
for the in-engine version since the codebase is Rust. The `▌` +
lowercase pattern is what reads as a Terminal-output-line; the
extension is incidental. (Same shape as the "RUSTY SOLITAIRE"
wordmark deviation noted under `cacb19c` — the mockup leaked the
repo name; the actual product is "Solitaire Quest".)

### `54005d5` `feat(engine): add GAME #YYYY-DDD caption beneath the replay headline`

Adds the right-anchored game-identifier piece of the replay-overlay
mockup, adapted to live *under* the existing "▌ replay" headline as
a `TYPE_CAPTION` (11 px) / `TEXT_SECONDARY` subtitle. Format is
`GAME #{year}-{ordinal:03}` (e.g. `GAME #2026-122` for a replay
recorded 2026-05-02) — year + chrono ordinal gives a compact,
monotonically-increasing identifier matching the mockup's
`GAME #2024-127` motif. New `ReplayOverlayGameCaption` marker, new
pure helper `format_game_caption(state) -> Option<String>` (None
for Inactive / Completed since the replay is consumed in those
branches; spawn-time fall-through to empty string).

**Layout impact:** `BANNER_HEIGHT` bumped 48 → 60 px so the new
left column (headline + 2 px gap + caption ≈ 39 px content) fits
under the scrub bar with room to spare. +12 px banner mass is the
deliberate cost of the new content; no other plugin observes
`BANNER_HEIGHT` so the change is local.

Two new tests (1180 → 1182): `format_game_caption_covers_state_corners`
pins the three branches plus the zero-pad-to-3-digits invariant
for early-January ordinals; `overlay_game_caption_shows_replay_date`
drives `ReplayPlaybackState` end-to-end.

### `e080b49` `feat(engine): restyle replay progress text as Terminal MOVE chip`

Closes the centre-text half of the replay-overlay enrichments. The
plain "Move N of M" text becomes a 1px `ACCENT_PRIMARY`-bordered
chip containing "MOVE N/M" — uppercase + slash separator reads as
a Terminal output line and matches the floating-chip motif in
`docs/ui-mockups/replay-overlay-mobile.html`. The chip lives
in-banner rather than floating above the focused card (the
screen-takeover treatment that requires plumbing cursor → card
identity remains deferred).

**Implementation note:** `BorderColor` in Bevy 0.18 is a per-side
struct, not a tuple — `BorderColor::all(ACCENT_PRIMARY)` is the
correct constructor. Worth pinning for next time we touch a
border-painted UI surface. The `ReplayOverlayProgressText` marker
stays on the inner Text rather than the new chip Node so
`update_progress_text` keeps repainting unchanged — a deliberate
"markers belong on the entity that updates change" choice.

Test count unchanged (1182); `overlay_progress_text_reflects_cursor`
swapped its assertion from "Move 5 of 10" to "MOVE 5/10".

This pair (`54005d5` + `e080b49`) closes Option C from the
SESSION_HANDOFF Resume prompt's banner-local enrichments. Floating-
chip-above-focused-card and the full screen-takeover redesign
remain — both data-layer or cross-plugin and intentionally still
open.

### `29136d8` `feat(engine): add pulsing trailing cursor to splash "▌ ready_" line`

Closes the cursor-pulse half of the splash polish arc deferred in
`cacb19c`. The "▌ ready_" line now ends with a 6×12 px cyan Node
that pulses on a 1 s sine cadence, multiplied with the global
splash fade timeline so the cursor never reaches full alpha while
the rest of the splash is still fading in.

**The "multiply, don't override" pattern.** Two systems write the
same `BackgroundColor` per frame: `advance_splash` writes the
global-fade alpha, `pulse_splash_cursor` overwrites with
`global_alpha × pulse_factor`. Both derive from `SplashAge` on the
root, so the writes are commensurate — the second one isn't
"fighting" the first, just refining it. This is the cleanest fix
for the "fight the global fade timeline" warning the original
`cacb19c` skip note flagged.

**Defensive division guard.** `cursor_pulse_factor(age, period, min)`
short-circuits to `1.0` when `period <= 0.0` so a future
misconfiguration produces a steady cursor rather than NaN
propagation (NaN in alpha = invisible UI, hard to debug). Worth
mirroring on every trig/division helper, not just this one.

One new test (1182 → 1183): `cursor_pulse_factor_corners` pins the
peak (factor = 1 at age = period / 4), trough (factor = min at age =
period × 3 / 4), and the zero/negative-period guard.

### `a27cf5a` `feat(engine): add tiled scanline overlay to splash`

Closes the scanline half of the splash polish arc. A fullscreen
`ImageNode` tiles a runtime-generated 2×2 RGBA8 texture over the
splash content — top row transparent, bottom row `#1a1a1a` at
~30 % alpha — producing the 1 px-pitch horizontal scanline pattern
called for in `docs/ui-mockups/splash-mobile.html`.

**Texture-α × tint-α composite for fade integration.** The 30 %
alpha is baked into the texture pixels, not the `ImageNode.color`
tint. `advance_splash`'s new third query writes
`(1, 1, 1, global_alpha)` into the tint each tick; the GPU
multiplies texture-α by tint-α, so the visible composite is
`0.3 × global_alpha`. Cleaner than building a "multiplicative
fadable" abstraction in the ECS — the GPU already does this
multiplication for free.

**Bevy 0.18 API surprises (worth pinning):**
- `RenderAssetUsages` re-exports under `bevy::asset::`, not
  `bevy::render::render_asset::`. Type name unchanged; module
  path moved.
- `TextureFormat::pixel_size()` returns `Result<usize, _>` rather
  than the bare `usize` you'd expect for a static format query.
  Annoying enough that the `debug_assert_eq!` against the buffer
  length just hard-codes the `2 × 2 × 4 = 16` literal.

Headless test fixture now also `init_resource::<Assets<Image>>()`
since `MinimalPlugins` doesn't pull `AssetPlugin` — same pattern
`settings_plugin::tests` already used. Without it, the
`Option<ResMut<Assets<Image>>>` parameter on `spawn_splash` would
fall through and the scanline overlay would silently skip,
defeating the new tests.

Two new tests (1183 → 1185):
`build_scanline_image_has_expected_2x2_rgba_bytes` locks the
texture pixels literally so a future tweak can't drift the
appearance silently; `scanline_overlay_spawns_and_fades_with_splash`
asserts spawn placement under `SplashRoot` and the new
fade-images branch's correctness end-to-end.

This pair (`29136d8` + `a27cf5a`) closes Option B from the
SESSION_HANDOFF Resume prompt — both splash polish pieces now
shipped.

### `5623368`…`dd101b3` — Option D card-face migration arc

Closed 2026-05-08 across nine commits. The full Terminal card
artwork now renders end-to-end. Detail breakdown lives in the
"Visual-identity follow-ups" punch-list entry below; the short
version:

- Migration plan + pipeline tooling: `5623368` (plan doc),
  `3a4bb63` (single-card PoC proving the `usvg`/`resvg` pipeline
  at per-card grain), `babe5cc` (full
  `solitaire_engine/examples/card_face_generator.rs` example
  emitting 52 faces + 5 backs into `assets/cards/`), `48b28d2`
  (the `card_face_svg_pin` integration test pinning rasteriser
  output via inline FNV-1a hashing of raw RGBA8 bytes — the
  pin's bootstrap pattern, "empty `EXPECTED` → run → paste",
  is the maintenance interface for future intentional changes).
- Lockstep step 4+5: `e8bf9d7`. New PNG bytes + the 5
  `card_plugin` constants (`CARD_FACE_COLOUR`,
  `RED_SUIT_COLOUR`, `BLACK_SUIT_COLOUR`,
  `CARD_FACE_COLOUR_RED_CBM` → `RED_SUIT_COLOUR_CBM`,
  `card_back_colour`) + signature shifts in one commit.
  `face_colour` deleted — Terminal face is uniformly
  `CARD_FACE_COLOUR` regardless of CBM, so the function
  collapsed to a constant. `text_colour` gained a
  `color_blind: bool` parameter (red→cyan suit-glyph swap when
  CBM is on). Four `face_colour` CBM tests folded into two
  `text_colour` CBM tests in lockstep.
- Three follow-ups that surfaced during sign-off, all from the
  same "fallback path the migration walked past" pattern:
  `a14200a` regenerated the embedded **default-theme SVGs** at
  `solitaire_engine/assets/themes/default/*.svg`; those bytes
  `include_bytes!()`-embed into the binary and override
  `assets/cards/*.png` at startup, so the PNG migration alone
  didn't change what production rendered. `8719f77`
  regenerated `assets/backgrounds/bg_*.png` to flat Terminal
  near-black (5 solid-colour PNGs via a new
  `solitaire_engine/examples/background_generator.rs` example).
  `ae84dc1` cleared the **top-bar overlap** at portrait/narrow
  window widths by swapping the action-button row's hardcoded
  `font_size: 16.0` to `TYPE_BODY` (a typography-migration
  miss) and stepping horizontal padding from `VAL_SPACE_3`
  to `VAL_SPACE_2`.
- Glyph-rendering fix: `af414b6`. The bundled `FiraMono`
  doesn't carry usable U+2660-2666 glyphs at the requested
  size — `usvg` was silently substituting tiny "tofu" marks.
  Switched suit glyphs from `<text>` elements to inline SVG
  `<path>` elements via a new `suit_path_d` helper. Path-based
  rendering bypasses the font system entirely; same bytes on
  every machine, no fontdb dependency, no substitution risk.
  Same path data renders correctly whether filled (♥ ♠) or
  outlined (♦ ♣ — the always-on color-blind glyph
  differentiation).
- Glyph-orientation tweak: `dd101b3`. Removed the 180° rotation
  from the bottom-right large suit glyph at user request. Both
  glyphs now render upright. `design-system.md` § Game Cards
  line 220 updated in lockstep — the deliberate deviation from
  the traditional inverted-corner-indicator convention is
  documented in the spec, not just the code.

The pin test fired exactly twice during this arc (once for the
text→path switch, once for the unrotation) and rebaselined
cleanly each time via the empty-then-paste pattern. The 5
`back_*` hashes stayed identical across both rebaselines —
secondary signal that the FNV-1a fingerprinting is purely
deterministic on rasteriser output.

This arc closes Option D from the SESSION_HANDOFF Resume prompt
and effectively completes the Terminal visual-identity port —
only the toast warning/error variant slots remain wired-but-
unused.

## What shipped in v0.20.0 (frozen at `41a009a`)

### Terminal visual-identity port

Top-down stack — every commit downstream of the token system
reads from it, so swapping the palette is now a one-file edit:

- **`ui_theme` token system** (`0d477ac`). base16-eighties
  palette, 5-rung type scale, 7-rung 4-multiple spacing scale,
  3-step radius, 14-rung z-index hierarchy, full motion budget,
  4 invariant-pinning unit tests. Card-shadow alphas pinned to 0
  (Terminal achieves depth via 1px borders + tonal layering).
- **Modal scaffold already on tokens** — `ui_modal` was ported
  in the same commit's wake; three stale "loud yellow" /
  "magenta secondary" doc comments fixed.
- **Gameplay feedback → semantic state tokens** (`ceec4fc`).
  Selection / valid-drop tints route through `ACCENT_PRIMARY` /
  `STATE_WARNING` / `STATE_SUCCESS`.
- **Toasts** (`a137607`). New `ToastVariant` enum
  (Info / Warning / Error / Celebration); opaque `BG_ELEVATED`
  + 1px accent border + bottom-anchor. All ten call sites pass
  their semantic variant.
- **`table_plugin` chrome** (`651f406`).
  `PILE_MARKER_DEFAULT_COLOUR` promoted; `cursor_plugin` imports
  it, replacing a "kept in sync" doc comment with a compile-
  enforced invariant. `HINT_PILE_HIGHLIGHT_COLOUR` →
  `STATE_WARNING`.
- **`card_plugin` chrome** (`d752870`). Drag-elevation shadow
  routes through `CARD_SHADOW_*` tokens. `RIGHT_CLICK_HIGHLIGHT_COLOUR`
  → `STATE_SUCCESS`. Stock recycle "↺" text → `TEXT_PRIMARY @ 0.7α`.
  Card-face / suit / card-back palette intentionally NOT migrated
  (artwork dependency — see open-list item below).
- **Splash cursor** (`cdcadda`). The signature `▌` cyan glyph
  (96 px) added above the wordmark, matching the spec.
  *Subsequently expanded post-cut by `cacb19c` into the full
  boot-screen treatment.*
- **Hint-source / dest pairing** (`9891ae4`). `input_plugin`'s
  source-card tint now matches the destination pile's
  `STATE_WARNING`.
- **Design system + 24-mockup library** (`fa7f98a`).
  `docs/ui-mockups/design-system.md` + 24 Stitch mockups (HTML +
  PNG) covering every screen plus 9 missing-plugin surfaces.
- **`card_shadow_params` test aligned** (`1d1543e`). Drag-vs-
  idle shadow assertion loosened to `>=` to accept the Terminal
  "no shadow" intent without losing the regression-guard.

### Android persistence

- **`solitaire_data::data_dir` shim** (`4b51e50`). New
  `solitaire_data::platform::data_dir()` falls through to
  `dirs::data_dir()` on desktop and returns the per-app sandbox
  at `/data/data/com.solitairequest.app/files` on Android — no
  JNI needed (package id pinned in `[package.metadata.android]`).
  Six `solitaire_data` callsites + `solitaire_engine/assets/user_dir.rs`
  migrated. Settings, stats, achievements, replays, game-state,
  time-attack sessions, and user themes now persist on Android.

### Inherited from earlier in the cycle (pre-session)

- Android build target + APK (`fb8b2ac`), runbook (`59424a3`),
  F3 FPS overlay (`690e1d2`), Smart Window Size opt-out
  (`e1b8766`), Shareable badge (`9b065e5`), Help cheat-sheet
  M/P/Enter rows (`35516d3`), `pull_failure_sets_error_status`
  flake fix (`67c150b`).

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

### Visual-identity follow-ups (opened by v0.20.0's port)

- *Card-face / suit / card-back artwork regeneration — closed
  2026-05-08 by the commit chain `5623368` → `dd101b3`.* The
  Terminal spec called for dark `#1a1a1a` cards with light suit
  pips (pink for hearts/diamonds, foreground gray for spades/
  clubs). Closed across nine commits over two arcs:
  - **Plan + tooling (`5623368`–`48b28d2`):** migration plan
    doc, single-card PoC, full `card_face_generator` example
    (52 faces + 5 backs into `assets/cards/`), and the
    `card_face_svg_pin` integration test pinning rasteriser
    output via FNV-1a so future `usvg`/`resvg` upgrades surface
    as test failures rather than silent visual drift.
  - **Lockstep step 4+5 (`e8bf9d7`):** PNGs + the 5 `card_plugin`
    constants + signature shifts in one commit.
    `CARD_FACE_COLOUR_RED_CBM` renamed to `RED_SUIT_COLOUR_CBM`
    and repurposed from a face-tint to a suit-glyph swap (the
    Terminal face is uniform `CARD_FACE_COLOUR` regardless of
    CBM; CBM only swaps red suits to cyan in the glyph itself).
    `face_colour` deleted, `text_colour` gained a `color_blind`
    parameter.
  - **Three follow-ups that surfaced during sign-off:**
    `a14200a` regenerated the **default-theme SVGs** at
    `solitaire_engine/assets/themes/default/*.svg` — those
    `include_bytes!()`-embed into the binary and override
    `assets/cards/*.png` at runtime, so the PNG migration alone
    didn't change what production rendered. `8719f77`
    regenerated `assets/backgrounds/bg_*.png` to flat Terminal
    near-black (5 solid-colour PNGs via a new
    `background_generator` example). `ae84dc1` cleared the
    **top-bar overlap** at portrait/narrow window widths by
    swapping the action-button row's hardcoded `font_size: 16.0`
    to `TYPE_BODY` and stepping horizontal padding from
    `VAL_SPACE_3` to `VAL_SPACE_2`.
  - **Glyph-rendering fix (`af414b6`):** suit glyphs render as
    inline SVG paths (not `<text>`) because the bundled
    `FiraMono` doesn't carry usable U+2660-2666 at the
    requested size — `usvg` was silently substituting tiny
    "tofu" marks. Path-based rendering bypasses the font system
    entirely; same bytes on every machine. The pin test
    rebaselined cleanly via the empty-then-paste pattern.
  - **Glyph-orientation tweak (`dd101b3`):** removed the 180°
    rotation from the bottom-right large suit glyph at user
    request — both glyphs now render in the same upright
    orientation. `design-system.md` § Game Cards line 220
    updated in lockstep to document the deliberate deviation
    from the traditional inverted-corner-indicator convention.
- *Splash boot-loader scanline overlay — closed by `a27cf5a`.*
  Runtime-generated 2 × 2 RGBA8 texture tiled via
  `NodeImageMode::Tiled`; per-pixel alpha × tint alpha gives
  multiplicative fade integration without new abstractions.
- *Splash cursor pulse — closed by `29136d8`.* Trailing 6 × 12 px
  cyan Node, sine-pulsed, multiplied with the global splash fade
  (the "multiply, don't override" pattern that resolves the
  original `cacb19c` skip-rationale).
- **Replay-overlay enrichments beyond the scrub bar.** Banner-local
  pieces of the mockup (`docs/ui-mockups/replay-overlay-mobile.html`)
  all shipped: scrub bar (`c84d9f4`), `▌ replay` cursor-block label
  (`6204db8`), `GAME #YYYY-DDD` caption (`54005d5`), `MOVE N/M`
  chip restyle (`e080b49`). What's still open are the cross-plugin
  / data-layer pieces: a `MOVE N/M` chip *floating above the
  focused card* during playback (would need to thread the cursor
  through to the card layer — `update_progress_text` writes the
  banner chip but the card-position lookup belongs in `card_plugin`).
  The full mockup's screen-takeover treatment — mini-tableau
  preview, playback controls, move-log scroll, WIN MOVE marker on
  the scrub bar — is a multi-session redesign with
  data-layer impact (move-log scroller; the WIN MOVE marker
  needs a `win_move_index` field on `Replay` that doesn't yet
  exist). Banner-overlay behaviour is intentionally preserved
  for now.
- **Toast Warning / Error variants.** The `ToastVariant` enum
  has slots for `Warning` (gold) and `Error` (pink) but no
  in-engine event uses them yet. Wire when a warning- or error-
  flavoured toast event materialises.

### Carried forward from v0.19.0

- **App icon round.** `Window::icon` not yet wired; no
  `.icns` / `.ico` / Linux hicolor PNG hierarchy. The 11-size
  icon export the v0.19 handoff referenced is *not* currently
  in `artwork/` (current `artwork/` holds the reverted Rusty
  Pixel card PNGs and is intentionally untracked); icon-export
  needs to be re-run before this item can be picked up.
  Half-day task once the PNGs are back in place. No cert
  dependency.

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
Always push there. **Local master has unpushed post-cut commits**
— run `git log --oneline origin/master..HEAD` for the live list;
`git push` is the next durability step (or roll the post-cut
commits into v0.20.1).

### Design direction (Terminal — base16-eighties)

- **Tone:** retro-terminal / synthwave — flat depth (no box-shadows),
  monospaced-forward typography (JetBrains Mono / FiraMono), tight
  16 px edge margins, 8 px card radius.
- **Palette:** near-black surface ramp (`#151515` / `#202020` /
  `#2a2a2a` / `#353535`), cyan primary CTA (`#6fc2ef`), lime
  success (`#acc267`), gold warning (`#ddb26f`), pink error /
  suit-red (`#fb9fb1`), lavender celebration (`#e1a3ee`), teal
  info (`#12cfc0`).
- **Two-color suits.** Red = `#fb9fb1`, black = `#d0d0d0`.
  Outlined glyphs for diamonds & clubs are *always on*; the
  Settings "color-blind mode" toggle only swaps red → cyan.

## Resume prompt

```
You are a senior Rust + Bevy developer working on Solitaire Quest.
Working directory: <Rusty_Solitaire clone path on this machine>.
Branch: master. v0.20.0 is tagged at 41a009a; the post-cut work
through dd101b3 is pushed to origin (Options B, C, D all closed).
Run `git log --oneline 41a009a..HEAD` to see what landed since the
tag — substantives: desktop-adaptation spec, splash boot-screen
port, replay-overlay banner enrichments, and the full card-face
artwork arc (52 faces + 5 backs as Terminal SVG-rasterised PNGs,
default-theme SVGs in lockstep, table backgrounds flattened,
top-bar layout fix, glyph orientation upright).

State: HEAD locally — see `git rev-parse HEAD`. Working tree is
clean. All workspace tests pass (~1180+; check with
`cargo test --workspace`), clippy clean.

READ FIRST (in order, before doing anything):
  1. SESSION_HANDOFF.md  — this file
  2. CHANGELOG.md        — [0.20.0] section is the most recent cut
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
  A. Push the post-cut commits to origin. Either as-is on master
     or rolled into a v0.20.1 cut (CHANGELOG entry + tag).
     Mechanical, but local master diverges from origin until done.
  B. *Closed by `29136d8` + `a27cf5a`.* Both splash polish
     pieces shipped (cursor pulse + scanline overlay). No further
     splash work pending unless a new mockup detail surfaces.
  C. *Closed by `54005d5` + `e080b49`.* Banner-local replay-overlay
     pieces all shipped (scrub bar, ▌ label, GAME caption, MOVE
     chip). Remaining are cross-plugin (floating MOVE chip above
     the focused card — needs cursor → card-position plumbing) or
     multi-session (full screen-takeover redesign — move-log
     scroll, mini tableau, WIN MOVE marker, data-layer impact).
     Either belongs in its own decision tree the next time replay
     work surfaces.
  D. *Closed 2026-05-08 by `5623368`…`dd101b3`.* The full
     card-face / suit / card-back / default-theme / table-
     background / top-bar / glyph-orientation arc landed across
     nine commits. Terminal cards rendering on every face (dark
     `#1a1a1a` background, pink/gray suit glyphs as inline SVG
     paths, scanline-pattern cyan-accent backs); both rendering
     paths (`assets/cards/*.png` and the bundled-default theme
     SVGs at `solitaire_engine/assets/themes/default/*.svg`) in
     lockstep; pin test (`card_face_svg_pin`) guards against
     future rasteriser drift. Visual-identity arc effectively
     complete — only the toast warning/error variant slots
     remain wired-but-unused.
  E. App icon round — re-run artwork/Icon Export.html (the
     export PNGs are not currently in `artwork/`), then wire
     Window::icon + generate .icns / .ico. Half-day task. No
     cert dependency.
  F. APK launch verification on AVD / device + the JNI bridges
     it would shake out (ClipboardManager, Keystore).

WORKFLOW NOTES:
  - Use the system git config (already correct).
  - When attributing playtester feedback in commits/docs, use
    "Quat" not "Rhys" (saved feedback memory).
  - Sub-agents stage + verify only; orchestrator commits.
  - Every commit must pass build / clippy / test before pushing.
  - Push to GitHub (origin) — gh auth setup-git wired on
    primary dev box; verify on laptop before first push.

OPEN AT THE START: ask which of A–F. Don't pick unilaterally.
```
