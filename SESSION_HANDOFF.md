# Solitaire Quest — UX Overhaul Session Handoff

**Last updated:** 2026-05-01 (session 6) — Six commits landed today on top of `v0.10.0`: four bug fixes surfaced by the player's first end-to-end smoke test of the embedded-theme path, plus a HUD-band layout reservation and an auto-fade for the action button bar. Direction has shifted from "cut a release" to "keep iterating on UX."

## Status at pause

- **HEAD:** `c4970b1`. **Pushed to GitHub** (`origin = https://github.com/funman300/Rusty_Solitare.git`).
- **Working tree:** clean. (`CARD_PLAN.md` is untracked but intentionally so — it's a plan doc, not source.)
- **Build:** `cargo clippy --workspace --all-targets -- -D warnings` clean.
- **Tests:** **962 passed / 0 failed / 9 ignored** across the workspace.
- **Tags on origin:** `v0.9.0`, `v0.10.0`. Local-only stale tag `v0.1.0` (points at a doc commit far behind HEAD — safe to `git tag -d v0.1.0`).

## Where we are

The card-theme system, HUD restructure, and modal scaffold are all complete. Today's session was a player-smoke-test pass that surfaced four bugs (theme assets not loading, an exit-time false-warn, two flavours of usvg font-substitution noise) plus two cosmetic wins (HUD crowding the cards, action buttons cluttering the play surface even when idle). All shipped.

The player explicitly said **the direction is more UX iteration, not release prep** — so the v0.11.0 cut and desktop packaging are deferred until they say otherwise.

### Design direction (unchanged)

- **Tone:** Balatro — chunky readable type, theatrical hierarchy, satisfying micro-interactions.
- **Palette:** Midnight Purple base + Balatro yellow primary + warm magenta secondary.
- See `~/.claude/projects/-home-manage-Rusty-Solitare/memory/project_ux_overhaul_2026-04.md` (auto-memory; on a different machine, recreate this fresh from the README + ARCHITECTURE.md).

### Canonical remote

`github.com/funman300/Rusty_Solitare` is the canonical repo. Earlier in this session a self-hosted Gitea remote (`git.aleshym.co/funman300/Rusty_Solitare`) was the source of truth on one machine, which caused commits to silently not reach the machine running the game. **Always push to GitHub.**

## Session 6 (shipped 2026-05-01)

| Area | Commit | What landed |
|---|---|---|
| Theme loader | `ab1d098` | `AssetPath::resolve` (concatenates) → `resolve_embed` (RFC 1808 sibling resolution). Was producing paths like `…/theme.ron/hearts_4.svg` and failing to load every face SVG. |
| Sync exit log | `9a9026e` | `push_on_exit` now silently no-ops on `LocalOnlyProvider`'s `UnsupportedPlatform` instead of warn-spamming "sync push on exit failed" every shutdown. Mirrors the pull path's existing handling. |
| Font warn (filter) | `78cf30e` | Initial fix: populated usvg fontdb with system fonts + LogPlugin filter. Insufficient on its own (warnings still fired). Superseded by next commit. |
| Font warn (resolver) | `efa063f` | Custom `usvg::FontResolver.select_font` appends `Family::SansSerif` and `Family::Serif` to every query so unmatched named families (Arial on Linux) silently fall through to whatever sans-serif fontconfig points at. Reverts the LogPlugin filter. |
| HUD band | `2c72e1f` | `layout::HUD_BAND_HEIGHT = 64` reserved at top; `top_y` shifts down. Translucent `BG_HUD_BAND` strip painted via `hud_plugin::spawn_hud_band`. Cards no longer crowd the action buttons / score text. |
| Action fade | `c4970b1` | `HudActionFade` resource + cursor-tracked auto-hide. Buttons fade out when cursor is below `HUD_BAND_HEIGHT + 32 px`, fade back in when it returns. Lerp at 6/sec ≈ 167 ms transition. Applied in `Last` schedule so paint_action_buttons can't clobber. |

## Open punch list — UX iteration (current direction)

The player's request, in priority order they've expressed:

1. **Unlock foundations** — currently `PileType::Foundation(Suit)` pre-assigns each foundation to a fixed suit (the slots show "C / D / H / S" placeholders). Player wants any Ace to land in any empty foundation; the slot then claims that suit until empty again. Cleanest path: change variant to `Foundation(u8)` (slot 0–3) and track the claimed suit as runtime state on `Pile`. Touches ~80 call sites in `solitaire_core` + `solitaire_engine`. Does NOT cross `solitaire_sync` (PileType is not transmitted), so no API break. **One-time invalidation of in-progress `game_state.json` saves on first launch after upgrade.**
2. **Card drop shadows against the felt** — cards currently read as flat stickers. Subtle 2-3 px shadow under each card, slightly stronger when picked up. "Make the play surface feel physical."
3. **Drop-target highlighting during drag** — when the player is holding a card, legal target piles glow / lift slightly. Highest gameplay-feel win in this list. Currently drops feel guess-y because there's no preview.
4. **Stock-pile remaining-count badge** — small "·N" chip on the corner of the stock so the player knows how many cards remain before a recycle. Currently they recycle blind.

## Open punch list — release prep (deferred per player)

These are still on the table but the player has explicitly deferred them in favour of more UX work:

1. **Cut `v0.11.0`** — meaningful slice since `v0.10.0`: full card-theme system (CARD_PLAN phases 1–7 + theme picker + hayeah art), HUD overhaul (band + fade), and the four bug fixes from session 6. (`git tag -d v0.1.0` first to clean up the stale local tag.)
2. **README + CHANGELOG refresh** — README was last touched at `a6b8348` before the Settings picker shipped; doesn't mention card themes or the auto-fade.
3. **Desktop packaging** per `ARCHITECTURE.md §17`. The Arch PKGBUILD exists in `/home/manage/solitaire-quest-pkgbuild/` (separate repo, no remote yet). Pending: app icon, macOS .icns + notarisation cert, Windows .ico + Authenticode cert, AppImage recipe.

### Optional, deferred (lower priority than the four UX items above)

- Animated focus ring (currently a static overlay; could pulse on focus change).
- Achievement onboarding pass — show first-time players the achievement panel after their first win.
- Mode-switch keyboard shortcut from inside the Mode Launcher (today only mouse opens it).
- Runtime aspect-ratio fidelity: hayeah SVGs are ~1.45 h/w; engine layout assumes 1.4. Cards render ~3 % squashed vertically. Cosmetic.

## Card-theme system (CARD_PLAN.md, fully shipped)

Seven phases landed across `b8fb3fb` → `924a1e2`. End-to-end:

- **Bundled default theme** ships in the binary via `embedded://` — 52 hayeah/playing-cards-assets SVGs (MIT) + a midnight-purple `back.svg` (original work).
- **User themes** live under `themes://` rooted at `solitaire_engine::assets::user_theme_dir()`. Drop a directory containing `theme.ron` + 53 SVG files; appears in the registry on next launch.
- **Importer** at `solitaire_engine::theme::import_theme(zip)` validates an archive (20 MB cap, zip-slip rejection, manifest validation, every SVG round-tripped through the rasteriser) and atomically unpacks.
- **Picker UI** in Settings → Cosmetic — one chip per registered theme; selection persists to `settings.json` as `selected_theme_id` and propagates to live sprites via `react_to_settings_theme_change` → `sync_card_image_set_with_active_theme` → `StateChangedEvent`.

## Resume prompt

```
You are a senior Rust + Bevy developer iterating on UX for Solitaire
Quest. Working directory: <Rusty_Solitare clone path on this machine>.
Branch: master. Current direction is UX iteration, NOT release prep —
the player explicitly deferred v0.11.0 and packaging in favour of more
gameplay-feel work.

State: HEAD=c4970b1, fully pushed to GitHub
(origin = https://github.com/funman300/Rusty_Solitare.git).
Working tree clean apart from untracked CARD_PLAN.md (intentional).
Build: cargo clippy --workspace --all-targets -- -D warnings clean.
Tests: 962 passed / 0 failed / 9 ignored.

READ FIRST (in order, before doing anything):
  1. SESSION_HANDOFF.md  — full state, session 6 changelog, punch list
  2. CLAUDE.md           — hard rules (UI-first, no panics, etc.)
  3. ARCHITECTURE.md     — crate responsibilities + data flow
  4. ~/.claude/projects/<this-project>/memory/MEMORY.md
                         — saved feedback / project context (machine-local;
                           may be missing on a fresh machine)

PUNCH LIST — UX iteration, in priority order the player expressed:
  1. Unlock foundations: any Ace lands in any empty slot, slot claims
     that suit until emptied. Refactor PileType::Foundation(Suit) →
     Foundation(u8). ~80 call sites. Invalidates in-progress saves.
     Does NOT cross solitaire_sync.
  2. Card drop shadows against the felt.
  3. Drop-target highlighting during drag (highest gameplay-feel win).
  4. Stock-pile remaining-count badge.

DEFERRED (do not start without explicit direction):
  - Cut v0.11.0, README/CHANGELOG refresh, desktop packaging.

WORKFLOW NOTES:
  - Commits use:
      git -c user.name=funman300 -c user.email=root@vscode.infinity \
          commit -m "..."
  - Sub-agents stage + verify only; orchestrator commits.
  - Every commit must pass build / clippy / test before pushing.
  - Push to GitHub (origin) — that is the canonical remote.

OPEN AT THE START: ask which punch-list item to start on. The player
prefers being asked over you picking unilaterally; (1) is the largest
in code-touch and they may want to scope it first.
```
