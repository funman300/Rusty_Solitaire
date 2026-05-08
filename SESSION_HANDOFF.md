# Solitaire Quest — Session Handoff

**Last updated:** 2026-05-07 — v0.20.0 cut and tagged at `41a009a`,
two post-cut commits sit on top of the tag, and a constructive
`replay_overlay` WIP is checked out in the working tree. The cut
itself shipped two through-lines: a full **Terminal visual-identity
port** (token system, modal scaffold, gameplay-feedback, toasts,
table / card chrome, splash cursor) and the **Android persistence
shim** that closes the `dirs::data_dir() = None` pitfall flagged in
CLAUDE.md §10. Since the cut, two more pieces landed: the rules-
based desktop-adaptation spec (closes the spec gap exposed when we
noticed 23 of 24 mockups were mobile-only) and the splash boot-
screen port (full mockup-spec splash with header, boot log,
progress bar, palette swatches, version footer, ~496 LOC of
`splash_plugin.rs` rewrite + `SplashFadable` scaffold refactor).

## Status at pause

- **HEAD locally:** `cacb19c` (splash boot-screen port).
- **HEAD on origin:** `41a009a` (the v0.20.0 cut). Local master is
  **2 commits ahead of origin** — `39b8496` (desktop-adaptation
  spec) and `cacb19c` are not yet pushed. Decide whether to roll
  these into v0.20.1 / v0.21.0-candidates before pushing.
- **Working tree:** dirty —
  `solitaire_engine/src/replay_overlay.rs` carries a constructive
  WIP for a 1px scrub-bar at the bottom edge of the replay banner
  (~120 LOC). Compiles with one missing piece: `update_scrub_fill`
  is referenced in the plugin's `add_systems` chain but the
  function body was never written. **The working tree does not
  compile. HEAD itself is clean** (verified by stashing the WIP
  and running `cargo check -p solitaire_engine` against the
  committed state — passes). Resume by writing the missing
  function (see "Open punch list → replay_overlay scrub bar").
- **`artwork/` directory:** still untracked. Intentional.
- **Build at HEAD (WIP stashed):**
  `cargo clippy --workspace --all-targets -- -D warnings` clean.
- **Tests at HEAD (WIP stashed):** **1178 passing / 0 failing**
  across the workspace. Up from 1176 at the v0.20.0 cut: the
  splash boot-screen port adds two new tests
  (`splash_renders_terminal_boot_screen_content` and
  `fadables_start_transparent_and_reach_full_alpha`).
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
- Pulsing cursor on the "ready_" line (would fight the global
  fade timeline).
- "RUSTY SOLITAIRE" wordmark from the mockup (the actual product
  is "Solitaire Quest"; the mockup leaked the repo name).

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

### In flight (resume here first)

- **`replay_overlay` scrub bar.** The working tree carries a
  WIP that adds a 1px-tall scrub bar at the bottom edge of the
  replay banner — track in `BORDER_SUBTLE`, fill in
  `ACCENT_PRIMARY`, width = `cursor / total` of the bar. The
  banner has been restructured from a single row (`flex-row`,
  `justify-between`) to a column with the existing content row +
  the new scrub bar. New marker `ReplayOverlayScrubFill`,
  `scrub_pct` helper function, and a reference to a system
  `update_scrub_fill` in the plugin's `add_systems` chain — but
  **the function body was never written**, so the working tree
  doesn't compile. Resume by:
  1. Writing `fn update_scrub_fill(state, mut q)` that reads
     `ReplayPlaybackState` and writes
     `Node::width = Val::Percent(scrub_pct(&state))` on every
     `ReplayOverlayScrubFill` entity, with a `state.is_changed()`
     early-exit (mirrors the existing `update_progress_text`
     shape).
  2. Adding two tests: scrub fill at 0 % when cursor = 0; scrub
     fill at 100 % on `Completed`.
  3. Commit message draft already implied by the WIP scope:
     `feat(engine): scrub-bar fill on the replay overlay`.
  WIP-only mockup elements deliberately left out: WIN MOVE
  marker (needs a `win_move_index` data-layer field that doesn't
  exist), 0/25/50/75/100 % notch labels (aesthetic-only), full
  playback toolbar / move-log / mini tableau (screen-takeover
  redesign, not a banner enhancement).

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

- **Card-face / suit / card-back artwork regeneration.** The
  Terminal spec calls for dark `#1a1a1a` cards with light suit
  pips (pink for hearts/diamonds, foreground gray for spades/
  clubs); the runtime path still renders the legacy white-card
  PNG artwork. The fallback constants in `card_plugin`
  (`CARD_FACE_COLOUR`, `RED_SUIT_COLOUR`, `BLACK_SUIT_COLOUR`,
  `CARD_FACE_COLOUR_RED_CBM`, `card_back_colour` palette) are
  intentionally unmigrated and should swap in lockstep with the
  artwork. Largest visible payoff remaining in the visual-
  identity arc.
- **Splash boot-loader scanline overlay.** `cacb19c` shipped the
  rest of the boot screen but skipped the scanline overlay
  (1px lines at 2 px pitch in `#1a1a1a` over the whole splash,
  30 % opacity). Needs a tiled-pattern asset (a 2 × 2 px PNG) or
  a custom shader. Pure aesthetic, no behaviour change.
- **Splash cursor pulse.** The "ready_" line's mockup pulses a
  cyan 6 × 12 px block at the end of the text. `cacb19c`
  skipped this because a per-element pulse fights the global
  `SplashFadable` fade timeline. Either layer the pulse on top
  of the fade (multiply alphas) or accept the static cursor.
- **Replay-overlay full redesign.** The scrub-bar WIP above is
  the *minimum* of the mockup. The full mockup
  (`docs/ui-mockups/replay-overlay-mobile.html`) is a screen-
  takeover with a mini-tableau preview, playback controls,
  move-log scroll, status bar, and a WIN MOVE marker. That's a
  multi-session redesign with data-layer impact (move log
  scroller, win-move detection). The current banner-overlay
  behaviour is intentionally preserved for now.
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
Always push there. **Local master is currently 2 commits ahead
of origin** — `git push` is the next durability step (or roll
the post-cut commits into v0.20.1).

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
Branch: master. v0.20.0 is tagged at 41a009a; two post-cut commits
sit on top locally (39b8496 desktop-adaptation spec, cacb19c splash
boot-screen port) — these have NOT been pushed yet.

State: HEAD locally at cacb19c. Working tree is dirty:
solitaire_engine/src/replay_overlay.rs carries a constructive WIP
for a 1px scrub-bar at the bottom of the replay banner. The WIP
references a function `update_scrub_fill` in the plugin's
add_systems chain but the body was never written — `cargo check`
fails on the working tree until the function is added. HEAD itself
(WIP stashed) is clean: 1178 tests pass, clippy clean.

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
  A. Finish the replay_overlay scrub-bar WIP. Write
     `update_scrub_fill`, add tests, commit. Tractable in one
     session; the WIP is fully scoped (see SESSION_HANDOFF.md →
     "In flight").
  B. Push the post-cut commits to origin. Either as-is on master
     or rolled into a v0.20.1 cut (CHANGELOG entry + tag).
     Mechanical, but local master diverges from origin until done.
  C. Card-face artwork regeneration. Generate Terminal-aesthetic
     card PNGs (dark face, light suit pips), then migrate
     CARD_FACE_COLOUR / RED_SUIT_COLOUR / BLACK_SUIT_COLOUR /
     CARD_FACE_COLOUR_RED_CBM in lockstep. Largest visible
     payoff remaining in the visual-identity arc. Multi-session.
  D. Splash scanline overlay + cursor pulse. The two pieces of
     the mockup `cacb19c` skipped. Pure polish; no behaviour
     change.
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
