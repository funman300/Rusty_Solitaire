# Solitaire Quest — Session Handoff

**Last updated:** 2026-05-08 — **v0.21.6 cut and tagged at
`f63db76`**, working tree clean, all post-tag work pushed to
origin.

v0.21.6 is a patch release with through-line:
**Move Log panel + scrub-UX polish**. v0.21.5 closed out the
keyboard-accelerator surface (Space / Esc / ← / →) and the
keybind footer; v0.21.6 builds on that with two parallel
threads — accessibility + scrub-on-hold polish for the v0.21.5
surfaces, plus a brand-new Move Log panel anchored to the
viewport's bottom edge that gives players a 5-row recent-and-
upcoming move history alongside the existing top-edge banner.

The Move Log panel is the first replay-overlay surface that
*isn't* attached to the banner — it lives at a separate screen
anchor (`bottom: 0`) with its own spawn/despawn lifecycle.
Establishes the "multi-anchor replay UI" pattern that the
remaining B-2 sub-piece (mini-tableau preview) will inherit.

Six commits on the B-2 replay screen-takeover redesign arc land
here, bringing the post-v0.21.4 total to 12. The remaining B-2
piece — mini-tableau preview that dims the gameplay tableau
during replay — is the only major sub-piece still open.

Full v0.21.6 detail lives in `CHANGELOG.md` § [0.21.6]. This
file from here on focuses on what's *open* post-cut and how to
resume.

## Status at pause

- **HEAD locally:** see `git rev-parse HEAD`. The cut commit is
  `f63db76`; any post-cut docs edits ride on top of that.
- **HEAD on origin:** matches local. v0.21.6 is fully on origin.
- **Working tree:** clean. No WIP outstanding.
- **`artwork/` directory:** still untracked. Intentional.
- **Build:** `cargo clippy --workspace --all-targets -- -D warnings`
  clean.
- **Tests:** **1273 passing / 0 failing** across the workspace.
  Detail in `CHANGELOG.md` § [0.21.6] § Stats.
- **Tags on origin:** `v0.9.0` through `v0.21.6`. v0.21.6 is on
  `f63db76`; v0.21.5 stays on `a2432df`; v0.21.4 stays on
  `23ff62c`; v0.21.3 stays on `3d92a91`; v0.21.2 stays on
  `f23df3b`; v0.21.1 stays on `daa655a`; v0.21.0 stays on
  `04f9bf9`; v0.20.0 stays on `41a009a`.

## Since the v0.21.6 cut

No threads in flight. Working tree clean as of 2026-05-08. New
work since the cut would land here as commit narratives; for
the v0.21.6 contents themselves, see `CHANGELOG.md` § [0.21.6].

Open next-step menu (Move Log + scrub-UX + keyboard accelerator
coverage + accessibility are all complete):
1. **Mini-tableau preview** — the only remaining major B-2
   sub-piece. Mockup shows a 240 px-tall band at 50 % opacity
   showing the gameplay tableau peeking through the replay
   chrome. Implementation needs to add a settings-aware dim
   overlay or alpha modulation on the tableau cards during
   replay. Architectural — touches `card_plugin` rendering.
   Multi-session.
2. **Move Log auto-scroll** — only relevant if the panel's
   row count grows beyond the current 5-row fixed window.
   Currently the prev-2 / active / next-2 layout fits all
   visible content, so auto-scroll is unneeded. Becomes
   relevant if a future commit expands the panel's row
   capacity (e.g. 10-row scrolling list).
3. **Polish: notch label centering.** Bevy 0.18 lacks a
   clean `translate-x: -50%` primitive so middle three
   labels sit slightly right-of-notch. Could use a child
   Text wrapper with computed left-margin compensation.
   Tiny commit, requires visual review.
4. **Polish: WIN MOVE marker HC bump.** Currently uses
   `STATE_SUCCESS` lime which stays visible under HC, but a
   contrast bump under HC would make it even more legible
   alongside the bumped notches. Optional.

Recommended order: option 1 (mini-tableau preview) is the
big remaining piece that closes B-2 — best tackled in a
fresh session because it crosses into `card_plugin`. Options
3 and 4 are visual polish that benefit from user review.

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
  accelerator) shipped post-v0.21.3 in `fbe48ac`. v0.21.5
  bundled six more commits under "replay-overlay scrubbing
  affordances + accessibility" (scrub notches + labels +
  keybind footer + ESC and ← / → accelerators + HC border).
  v0.21.6 bundled six more under "Move Log panel + scrub-UX
  polish" — bottom-edge Move Log panel with prev/active/next
  rows + active highlight, HC-mode coverage for the scrub
  track + notches, continuous scrub on key-held arrows. Banner
  height grew 60 → 76 → 92 px across two layout-changing
  commits in v0.21.5; Move Log panel grew 56 → 84 → 112 px
  across the v0.21.6 move-log commits. Per-commit detail in
  `CHANGELOG.md` § [0.21.5] and § [0.21.6]. The only major
  B-2 piece left is the mini-tableau preview — the mockup's
  "Game Peek Band" at 50 % opacity. Architectural; touches
  `card_plugin` rendering.
  Multi-session.
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
Branch: master. v0.21.6 is tagged at f63db76 (cut 2026-05-08, a
patch release rolling up Move Log panel + scrub-UX polish:
brand-new bottom-edge Move Log panel with prev / active / next
row context + active-row highlight, plus HC-mode coverage for
scrub track + notches and continuous scrub on key-held arrow
keys). v0.21.5 stays at a2432df, v0.21.4 at 23ff62c, v0.21.3
at 3d92a91, v0.21.2 at f23df3b, v0.21.1 at daa655a, v0.21.0 at
04f9bf9. Working tree clean. See CHANGELOG.md § [0.21.6] for
full detail.

State: HEAD locally — see `git rev-parse HEAD`. The cut commit
is f63db76; any post-cut docs edits ride on top of that.
Workspace tests: 1273 passing / 0 failing. Clippy clean.

READ FIRST (in order, before doing anything):
  1. SESSION_HANDOFF.md  — this file
  2. CHANGELOG.md        — [0.21.6] section is the most recent cut
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
  B. Replay-overlay screen-takeover redesign — nearly complete
     after 12 commits across v0.21.4-6. Scrub bar with notches
     + labels + WIN MOVE marker, pause / resume / step / stop
     buttons, Space + Esc + ← / → keyboard accelerators with
     continuous scrub on hold, keybind-hint footer, full HC-mode
     coverage on the banner pieces, and a brand-new bottom-edge
     Move Log panel with a 5-row prev/active/next window all
     ship. The only remaining major B-2 sub-piece is the
     **mini-tableau preview** — the mockup's "Game Peek Band"
     at 50 % opacity showing the tableau through the replay
     chrome. Implementation needs a settings-aware dim overlay
     or alpha modulation on the tableau cards during replay.
     Architectural — touches `card_plugin` rendering. Best
     tackled in a fresh session because it crosses into a
     plugin the recent B-2 work hasn't touched. Mockup at
     `docs/ui-mockups/replay-overlay-mobile.html`.
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
