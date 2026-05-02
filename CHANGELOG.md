# Changelog

All notable changes to Solitaire Quest are documented here. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this
project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

_Nothing yet._

## [0.12.0] — 2026-05-02

UX feel polish round on top of v0.11.0. Six small-but-tangible
improvements that make the play surface feel more responsive,
forgiving, and discoverable, plus the doc refresh that should have
ridden along with v0.11.0.

### Added

- **Foundation completion flourish.** When a King lands on a
  foundation (Ace-through-King for that suit), a brief celebration
  fires: King card scale-pulses 1.0 → 1.15 → 1.0 over 0.4 s, the
  foundation marker tints `STATE_SUCCESS` for the first half then
  fades, and a synthesised C6→E6→G6 bell ping plays (~240 ms,
  octave above `win_fanfare`'s root so the fourth completion + win
  cascade layer cleanly). New `FoundationCompletedEvent { slot,
  suit }` carries the trigger so future systems can hook in.
- **Drag-cancel return tween.** Illegal drops glide each dragged
  card back to its origin slot over 150 ms with a quintic ease-out
  curve (`MotionCurve::Responsive`, zero overshoot — reads forgiving
  rather than jittery). The audio cue (`card_invalid.wav`) still
  fires for negative feedback. Right-click and double-click invalid
  paths still use `ShakeAnim` since there's no motion to interpolate.
- **Focus ring breathing.** The keyboard focus ring's alpha modulates
  with a 1.4 s sin curve over [0.65, 1.0] of its native value so the
  indicator catches the eye on focus changes without competing with
  gameplay. Honours `AnimSpeed::Instant` by reverting to the static
  outline for reduced-motion users.
- **First-win achievement onboarding toast.** After the player's
  very first win, a one-shot info toast surfaces "First win! Press
  A to see your achievements." `Settings.shown_achievement_onboarding`
  persists the seen state so the cue never re-fires (legacy
  `settings.json` files load to `false` via `#[serde(default)]`).
- **Mode Launcher digit shortcuts.** Pressing M opens the Home modal
  (the Mode Launcher); inside it, pressing 1–5 launches each mode
  directly without needing Tab + Enter. Locked modes (Zen, Challenge,
  Time Attack at level < 5) are silent no-ops. Modal-scoped — digit
  keys outside the launcher fire nothing.

### Fixed

- **Card aspect ratio matches hayeah SVGs.** `CARD_ASPECT` 1.4 →
  1.4523 to match the bundled artwork's natural 167.087 × 242.667
  dimensions. Cards previously rendered ~3.6 % vertically squashed.
  The vertical-budget math in `compute_layout` uses `CARD_ASPECT`
  algebraically so the worst-case-tableau-fits-on-screen guarantee
  adapts automatically.

### Documentation

- **README refresh** with v0.11.0+ features (card themes, HUD
  overhaul, drag feel, unlocked foundations) and a corrected controls
  table — the previous table inverted Z/U for undo and listed H for
  help when F1 is the binding.
- **CHANGELOG.md** added (this file), covering v0.9.0–v0.12.0 with
  Keep a Changelog 1.1.0 conventions.

### Stats

- 1007 passing tests (was 982 at v0.11.0).
- Zero clippy warnings under `--workspace --all-targets -- -D warnings`.

## [0.11.0] — 2026-05-02

The biggest release since 0.10.0. Headline threads: a runtime card-theme
system, an HUD restructure that reclaims the play surface, and a round of
UX feel polish surfaced by smoke testing.

### Added

- **Runtime card-theme system** (CARD_PLAN phases 1–7).
  - Bundled default theme ships in the binary via `embedded://` — 52
    [hayeah/playing-cards-assets](https://github.com/hayeah/playing-cards-assets)
    SVGs (MIT) plus a midnight-purple `back.svg` as original work.
  - User themes live under `themes://` rooted at `user_theme_dir()`. Drop
    a directory containing `theme.ron` + 53 SVGs and the registry picks
    it up on next launch.
  - Importer at `solitaire_engine::theme::import_theme(zip)` validates
    archives (20 MB cap, zip-slip rejection, manifest validation, every
    SVG round-tripped through the rasteriser) and atomically unpacks.
  - Picker UI in **Settings → Cosmetic**; selection persists as
    `selected_theme_id` and propagates to live sprites.
- **Reserved HUD top band** (64 px) so cards no longer crowd the score
  readout or action buttons; layout's `top_y` shifts down accordingly.
- **Action-bar auto-fade** — buttons fade out when the cursor leaves the
  band, fade back in when it returns. Lerp at ~167 ms.
- **Visible drop-target overlay during drag** — a soft fill plus 3 px
  outline drawn ABOVE stacked cards for every legal target (full fanned
  column for tableaux, card-sized for foundations and empty tableaux).
  Replaces the previously invisible pile-marker tint.
- **Card drop shadows** — every card casts a neutral 25 % black shadow
  with a 4 px halo; cards in the active drag set switch to a lifted
  shadow (40 % alpha, larger offset, bigger halo).
- **Stock remaining-count badge** — small `·N` chip at the top-right of
  the stock pile so the player can see how close they are to a recycle.
  Hides when the stock empties.

### Changed

- **Foundations are unlocked.** `PileType::Foundation(Suit)` →
  `Foundation(u8)` (slot 0..3). The claimed suit is derived from the
  bottom card via `Pile::claimed_suit()` — no separate field, no
  claim-stuck-after-undo bugs. Any Ace lands in any empty slot, and the
  slot then claims that suit. `next_auto_complete_move` prefers a
  claim-matched slot before falling back to the first empty slot for
  Aces. Empty foundation markers render as plain placeholders (no
  "C/D/H/S").
- **HUD selection label** and **hint toast** read `claimed_suit()` and
  fall through to "Foundation N" / "move to foundation" only when the
  slot is empty.

### Fixed

- **`shared_fontdb` now bundles FiraMono.** The hayeah SVGs reference
  `Bitstream Vera Sans` and `Arial` by name. On minimal Linux installs
  / fresh Wayland sessions / chroots where neither is installed AND the
  CSS-generic aliases don't resolve, card rank/suit text vanished. The
  bundled font is loaded into fontdb and pinned as every CSS generic's
  target so the resolver always lands on something real. Surfaced when
  a second-machine pull rendered cards without glyphs.
- **Theme asset path resolution** — `AssetPath::resolve` (concatenates)
  → `resolve_embed` (RFC 1808 sibling resolution). Was producing paths
  like `…/theme.ron/hearts_4.svg` and failing to load every face SVG.
- **Sync exit log spam** — `push_on_exit` silently no-ops on
  `LocalOnlyProvider`'s `UnsupportedPlatform` instead of warn-spamming
  every shutdown.
- **usvg font-substitution warn spam** — custom `FontResolver.select_font`
  appends `Family::SansSerif` and `Family::Serif` to every query so
  unmatched named families silently fall through.

### Migration

- **In-progress saves invalidated.** `GameState.schema_version` bumped
  1 → 2; pre-v2 `game_state.json` files silently fall through to "fresh
  game on launch." Stats, progress, achievements, and settings live in
  separate files and are unaffected.

### Stats

- 982 passing tests (was 819 at v0.10.0).
- Zero clippy warnings under `--workspace --all-targets -- -D warnings`.

## [0.10.0] — 2026-04-29

PNG art pipeline plus a major dependency pass. The first release where
the binary shipped with bundled artwork.

### Added

- **52 individual card face PNGs** generated via `solitaire_assetgen`.
- **Custom font** (FiraMono-Medium) loaded via `AssetServer` at startup
  through the new `FontPlugin`.
- **Card backs and backgrounds** upgraded to 120×168 with richer
  patterns.
- **Ambient audio loop** wired through the kira mixer.
- **Arch Linux PKGBUILDs** for the game client and sync server (under
  the separate `solitaire-quest-pkgbuild` directory).
- **Workspace README, CI workflow, migration guide.**

### Changed

- **Bevy 0.15 → 0.18** workspace migration.
- **kira 0.9 → 0.12** audio backend migration.
- **Edition 2024**, MSRV pinned to **Rust 1.95**.
- **rand 0.9** upgrade.
- **Card rendering** moved from `Text2d` overlay to PNG-backed
  `Sprite` with face/back atlases; `Text2d` retained as a headless
  fallback when `CardImageSet` is absent (tests under MinimalPlugins).
- **Asset pipeline** switched from `include_bytes!()` for PNGs/TTFs to
  runtime `AssetServer::load()` so artwork can be swapped without a
  recompile. Audio remains embedded.
- **Removed Google Play Games Services sync backend** — redundant with
  the self-hosted server.

### Fixed

- **Server JWT secret** loaded at startup (was lazy, surfaced as
  intermittent 500s).
- **Daily-challenge race** in the server's seed-generation path.
- **Rate limiter** switched to `SmartIpKeyExtractor` so the limit
  applies per real client IP rather than per upstream proxy.
- **Touch input** uses `MessageReader<TouchInput>` (Bevy 0.18 rename).
- **Sync push/pull races** in async task scheduling.
- **Hot-path allocations** reduced in card-rendering systems.
- **Conflict report coverage** added for sync merge edge cases.

### Stats

- 819 passing tests at tag time.

## [0.9.0] — 2026-04-28

Initial public-tagged release. Established the workspace structure
(`solitaire_core` / `_sync` / `_data` / `_engine` / `_server` / `_app` /
`_assetgen`), the modal scaffold via `ui_modal`, the design-token system
in `ui_theme`, and the four-tier HUD layout. Foundations were
suit-locked at this point; cards rendered as `Text2d` rank/suit overlays
with no PNG artwork yet.

### Added

- Klondike core (Draw One / Draw Three modes).
- Progression system (XP, levels, 18 achievements, daily challenge,
  weekly goals, special modes at level 5).
- Self-hosted sync server (Axum + SQLite + JWT auth).
- All 12 overlay screens migrated to the `ui_modal` scaffold with real
  Primary/Secondary/Tertiary buttons.
- Animation upgrades: `SmoothSnap` slide curves, scoped settle bounce,
  deal jitter, win-cascade rotation.
- Splash screen, focus rings (Phases 1–3), tooltips infrastructure +
  HUD/Settings/popover applications, achievement integration tests,
  destructive-confirm verb unification, leaderboard error/idle states,
  first-launch empty-state polish, hit-target accessibility fix,
  CREDITS.md, persistent window geometry, mode-launcher Home repurpose,
  client-side sync round-trip integration tests.

[Unreleased]: https://github.com/funman300/Rusty_Solitaire/compare/v0.12.0...HEAD
[0.12.0]: https://github.com/funman300/Rusty_Solitaire/compare/v0.11.0...v0.12.0
[0.11.0]: https://github.com/funman300/Rusty_Solitaire/compare/v0.10.0...v0.11.0
[0.10.0]: https://github.com/funman300/Rusty_Solitaire/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/funman300/Rusty_Solitaire/releases/tag/v0.9.0
