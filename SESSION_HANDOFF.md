# Solitaire Quest — Session Handoff

**Last updated:** 2026-05-07 — v0.20.0 cut. Two through-lines closed
in this cycle: a full **Terminal visual-identity port** (token system
in `ui_theme` plus downstream chrome migrations across modal scaffold,
gameplay-feedback, toasts, and the table / card / splash surfaces)
and the **Android persistence shim** that closes the
`dirs::data_dir() = None` pitfall flagged in CLAUDE.md §10. The
Android *build* target landed earlier in the cycle (`fb8b2ac`); this
session paid down the persistence half so a real APK can survive a
cold start. The 24 Stitch-rendered mockups are now in-tree under
`docs/ui-mockups/`; future plugin work diffs against the matching
mockup before touching pixels.

## Status at pause

- **HEAD on origin:** the v0.20.0 docs commit (the one that lands
  this file + CHANGELOG cut). Tag not yet pushed; cut whenever
  feels right.
- **Working tree:** clean apart from the still-untracked `artwork/`
  directory (intentional — the card PNGs there are mid-flight for
  the Terminal aesthetic and committing now would freeze a
  transitional state).
- **Build:** `cargo clippy --workspace --all-targets -- -D warnings`
  clean.
- **Tests:** **1176 passing / 0 failing** across the workspace.
  Six new tests this cycle: four `ui_theme` invariant guards
  (type / spacing / z-index scales + `scaled_duration`), one
  toast-variant-border-mapping pair, and four palette-tracking
  guards on `MARKER_VALID` / `HINT_PILE_HIGHLIGHT_COLOUR` /
  `RIGHT_CLICK_HIGHLIGHT_COLOUR` / toast-border distinctness. No
  known flakes.
- **Tags on origin:** `v0.9.0` through `v0.19.0`. v0.20.0 not yet
  tagged.

## What shipped in v0.20.0

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
- **Splash boot-loader richness.** The mockup
  (`docs/ui-mockups/splash-mobile.html`) calls for a scanline
  overlay, ✓ lime check log lines, pulsing cursor, ROOT@SOLITAIRE
  prompt, and a loading bar — none of which v0.20.0's
  cursor-glyph-only port pulled in. Aesthetic feature, its own
  commit.
- **Replay-overlay redesign.** The mockup
  (`docs/ui-mockups/replay-overlay-mobile.html`) envisions a
  much richer surface (terminal `▌replay.tsx` header, move log
  scroll, MOVE 47/87 chip, WIN MOVE callout, status bar) versus
  the current top banner. Aesthetic feature.
- **Toast Warning / Error variants.** The new `ToastVariant`
  enum has slots for `Warning` (gold) and `Error` (pink) but no
  in-engine event uses them yet (the four current toast events
  all map to Info or Celebration). Wire when a warning- or
  error-flavoured toast event materialises.

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

- **Token-port pattern.** v0.20.0's chrome-migration commits
  set a reusable shape for "centralized design system applied
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
- **Audit before migrating wide.** Before touching any plugin,
  grep for the literal pattern (`Color::srgb\(|Color::srgba\(|
  Color::WHITE|Color::BLACK`) and classify each hit as domain
  vs. chrome. Most plugins after the modal scaffold port turned
  out to be 100 % token-correct already; the audit prevents
  wasted churn.

### Canonical remote

`github.com/funman300/Rusty_Solitaire` is the canonical repo.
Always push there.

### Design direction (now Terminal — base16-eighties)

- **Tone:** retro-terminal / synthwave — flat depth (no box-shadows),
  monospaced-forward typography (JetBrains Mono / FiraMono), tight
  16 px edge margins, 8 px card radius.
- **Palette:** near-black surface ramp (`#151515` / `#202020` / `#2a2a2a`
  / `#353535`), cyan primary CTA (`#6fc2ef`), lime success
  (`#acc267`), gold warning (`#ddb26f`), pink error / suit-red
  (`#fb9fb1`), lavender celebration (`#e1a3ee`), teal info
  (`#12cfc0`).
- **Two-color suits.** Red = `#fb9fb1`, black = `#d0d0d0`. Outlined
  glyphs for diamonds & clubs are *always on*; the Settings
  "color-blind mode" toggle only swaps red → cyan.

(Was: Midnight Purple base + Balatro yellow primary + warm magenta.
Replaced this cycle.)

## Resume prompt

```
You are a senior Rust + Bevy developer working on Solitaire Quest.
Working directory: <Rusty_Solitaire clone path on this machine>.
Branch: master. v0.20.0 just cut on 2026-05-07; CHANGELOG's new
[Unreleased] section is empty pending the next cycle's threads.

State: HEAD on the v0.20.0 docs commit. Tag not pushed yet — last
pushed tag is v0.19.0. Working tree clean apart from the
intentionally-untracked `artwork/`.

READ FIRST (in order, before doing anything):
  1. SESSION_HANDOFF.md  — this file
  2. CHANGELOG.md        — [0.20.0] section is the most recent cut
  3. CLAUDE.md           — unified-3.0 rule set
  4. CLAUDE_SPEC.md      — formal architecture spec
  5. ARCHITECTURE.md     — crate responsibilities + data flow
  6. docs/ui-mockups/    — design system + 24-mockup library
                           (Terminal aesthetic — landed in fa7f98a)
  7. docs/android/*      — Android setup + build runbook
  8. ~/.claude/projects/<this-project>/memory/MEMORY.md
                         — saved feedback / project context
                           (machine-local; may be missing on a
                           fresh machine)

DECISION TO ASK THE PLAYER FIRST:
  A. Push v0.20.0 tag — `git tag v0.20.0 && git push --tags`. If
     the player wants the cut formalised before any new work.
  B. APK launch verification — `adb install` + `adb logcat` on
     bevy_test AVD or an x86_64 device. Now that persistence is
     wired (4b51e50), shake out remaining runtime bugs.
  C. Card-face artwork regeneration — generate Terminal-aesthetic
     card PNGs (dark face, light suit pips), then migrate
     CARD_FACE_COLOUR / RED_SUIT_COLOUR / BLACK_SUIT_COLOUR /
     CARD_FACE_COLOUR_RED_CBM in lockstep. Largest visible
     payoff remaining in the visual-identity arc.
  D. Splash boot-loader richness — port the scanline overlay,
     ✓ check log, pulsing cursor, ROOT@SOLITAIRE prompt, and
     loading bar from docs/ui-mockups/splash-mobile.html. Pure
     polish; no behavioural change.
  E. App icon round — re-run artwork/Icon Export.html (the
     export PNGs are not currently in `artwork/`), then wire
     Window::icon + generate .icns / .ico. Half-day task. No
     cert dependency.
  F. JNI ClipboardManager / Keystore bridge — replaces the
     Android stubs for Stats clipboard share + sync auth.

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
