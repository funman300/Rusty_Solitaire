# Solitaire Quest — Session Handoff

**Last updated:** 2026-05-08 — **v0.21.0 cut and tagged at `04f9bf9`**,
working tree clean, all post-tag work pushed to origin.

v0.21.0 closes the visual-identity arc opened in v0.20.0. Three
through-lines landed in this cycle: the **card-face / suit /
card-back artwork migration** that v0.20.0 deliberately deferred
(both rendering paths in lockstep — `assets/cards/*.png` fallback
plus the bundled-default theme SVGs at
`solitaire_engine/assets/themes/default/*.svg` that
`include_bytes!()`-embed into the binary), the **splash boot-
screen + replay-overlay polish** that closed Resume-prompt
Options B and C, and a late-cycle **`ACCENT_PRIMARY` palette
swap** from cyan `#6fc2ef` to brick red `#a54242` after a quick
stakeholder review of the shipped art.

Full v0.21.0 detail lives in `CHANGELOG.md` § [0.21.0]. This
file from here on focuses on what's *open* post-cut and how to
resume.

## Status at pause

- **HEAD locally:** see `git rev-parse HEAD`. The cut commit is
  `04f9bf9`; any post-cut docs edits ride on top of that.
- **HEAD on origin:** matches local. v0.21.0 is fully on origin.
- **Working tree:** clean. No WIP outstanding.
- **`artwork/` directory:** still untracked. Intentional.
- **Build:** `cargo clippy --workspace --all-targets -- -D warnings`
  clean.
- **Tests:** **1184 passing / 0 failing** across the workspace
  (net +8 from v0.20.0's 1176 baseline). Detail in
  `CHANGELOG.md` § [0.21.0] § Stats.
- **Tags on origin:** `v0.9.0` through `v0.21.0`. v0.21.0 is on
  `04f9bf9`; v0.20.0 stays on `41a009a`.

## Since the v0.21.0 cut

No threads in flight. Working tree clean as of 2026-05-08. New
work since the cut would land here as commit narratives; for the
v0.21.0 contents themselves, see `CHANGELOG.md` § [0.21.0].

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
  `e080b49`); the screen-takeover is a multi-session redesign
  with data-layer impact (move-log scroller; WIN MOVE needs a
  `win_move_index` field on `Replay` that doesn't yet exist).
- **Floating `MOVE N/M` chip above the focused card during
  playback.** Cross-plugin work — `update_progress_text` writes
  the banner chip but the card-position lookup belongs in
  `card_plugin`. Smaller scope than the screen-takeover.
- **Toast Warning / Error variants.** `ToastVariant` has slots
  for `Warning` (gold) and `Error` (pink) but no in-engine
  event uses them yet. Wire when a warning- or error-flavoured
  toast event materialises.
- **High-contrast accessibility mode.** `design-system.md` line
  274 mentions it (boost foreground from `#d0d0d0` to `#f5f5f5`,
  suit-red from `#fb9fb1` to `#ff8aa0`). Not implemented; no
  Settings toggle.
- **Reduced-motion mode** for card lift / drop transitions.
  `design-system.md` accessibility item, separate from
  artwork. Not implemented.

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
Branch: master. v0.21.0 is tagged at 04f9bf9 (cut 2026-05-08).
Working tree clean. v0.21.0 closed the visual-identity arc that
v0.20.0 deferred — full Terminal cards on both rendering paths
(asset PNGs + bundled-default theme SVGs), splash boot screen,
replay-overlay banner enrichments, and a project-wide ACCENT_PRIMARY
swap from cyan to brick red `#a54242`. See CHANGELOG.md § [0.21.0]
for full detail.

State: HEAD locally — see `git rev-parse HEAD`. All workspace tests
pass (1184+; check with `cargo test --workspace`), clippy clean.

READ FIRST (in order, before doing anything):
  1. SESSION_HANDOFF.md  — this file
  2. CHANGELOG.md        — [0.21.0] section is the most recent cut
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
  A. App icon round — re-run artwork/Icon Export.html (the
     export PNGs are not currently in `artwork/`), then wire
     Window::icon + generate .icns / .ico / Linux hicolor PNG
     hierarchy. Half-day task. No cert dependency. Pure
     desktop work.
  B. APK launch verification on AVD / device — `adb install` +
     `adb logcat` to shake out runtime bugs the build / unit
     tests can't catch. Likely surfaces JNI ClipboardManager
     and Android Keystore stubs that need real bridges. Larger
     scope than A; needs an Android device or emulator running.
  C. Replay-overlay extensions — either the floating `MOVE N/M`
     chip above the focused card (smaller, cross-plugin; needs
     cursor → card-position plumbing in `card_plugin`) or the
     full screen-takeover redesign (multi-session: move-log
     scroll, mini tableau preview, WIN MOVE marker, data-layer
     impact for `Replay::win_move_index`).
  D. Toast Warning / Error variant wiring. UI infrastructure
     exists in `ToastVariant`; no in-engine event uses Warning
     (gold) or Error (pink) yet. Wire when a real warning- or
     error-flavoured event materialises.
  E. Phase 8 (sync) — local storage scaffolding, self-hosted
     Axum server, `SolitaireServerClient` impl, GPGS stub
     wired into Settings. The biggest open arc by scope; rolls
     up several Phase Android dependencies (Keystore,
     ClipboardManager).
  F. High-contrast and reduced-motion accessibility modes.
     Both flagged in design-system.md but unimplemented; needs
     Settings toggles + token alternates.

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

OPEN AT THE START: ask which of A–F. Don't pick unilaterally.
```
