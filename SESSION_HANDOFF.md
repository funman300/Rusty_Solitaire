# Solitaire Quest — UX Overhaul Session Handoff

**Last updated:** 2026-05-02 (session 7) — UX iteration round complete: every item from session 6's UX punch list has shipped, plus a font-fallback fix surfaced by a second-machine smoke test. Six commits on top of session 6's `c4970b1`. Direction now opens for the next round — release prep or another UX pass, the player's call.

## Status at pause

- **HEAD:** `655dfde`. Local master is **3 commits ahead** of `origin/master` (`f6c9166`, `f712b89`, `655dfde` unpushed; `fdb6c2e` and `95df542` already pushed).
- **Working tree:** clean. (`CARD_PLAN.md` is untracked but intentionally so — it's a plan doc, not source.)
- **Build:** `cargo clippy --workspace --all-targets -- -D warnings` clean.
- **Tests:** **982 passed / 0 failed** across the workspace (+20 from session 6's 962 baseline).
- **Tags on origin:** `v0.9.0`, `v0.10.0`. Stale local-only `v0.1.0` is still safe to `git tag -d v0.1.0`.

## Where we are

Session 6's UX punch list was four items. All four shipped today, plus an unrelated font-fallback fix from a second-machine smoke test.

The card-theme system, HUD restructure, modal scaffold, and the four big UX feel items (foundations, drop shadows, drop highlights, stock badge) are all in. Direction is open — the deferred release-prep items (`v0.11.0` cut, README/CHANGELOG refresh, desktop packaging) are still on the table, and a fresh round of UX iteration is also available.

### Design direction (unchanged)

- **Tone:** Balatro — chunky readable type, theatrical hierarchy, satisfying micro-interactions.
- **Palette:** Midnight Purple base + Balatro yellow primary + warm magenta secondary.
- See `~/.claude/projects/-home-manage-Rusty-Solitare/memory/project_ux_overhaul_2026-04.md` (auto-memory; on a different machine, recreate this fresh from the README + ARCHITECTURE.md).

### Canonical remote

`github.com/funman300/Rusty_Solitaire` is the canonical repo. Always push there. (Earlier sessions used `Rusty_Solitare` — single-i typo — as the repo name; the rename to `Rusty_Solitaire` happened in session 7. Local clone directories may still be named `Rusty_Solitare`; that's just a directory name and works fine.)

## Session 7 (shipped 2026-05-02)

| Area | Commit | What landed |
|---|---|---|
| Font fallback | `fdb6c2e` | `shared_fontdb` now `include_bytes!()`s `assets/fonts/main.ttf` (FiraMono) and pins every CSS generic to `"Fira Mono"` so unmatched named families on minimal Linux installs / fresh Wayland sessions / chroots don't drop card rank/suit text. Surfaced when a second-machine pull rendered cards without glyphs. |
| Unlock foundations | `95df542` | `PileType::Foundation(Suit)` → `Foundation(u8)` (slot 0..3). `Pile::claimed_suit()` derives the claim from the bottom card — no separate field, no claim-stuck-after-undo class of bugs. `can_place_on_foundation` drops its suit parameter. `next_auto_complete_move` prefers a slot whose claimed suit matches the candidate before falling back to the first empty slot for an Ace. Empty foundation markers render as plain placeholders (no "C/D/H/S"). HUD selection label and hint toast read `claimed_suit()` and fall through to "Foundation N" / "move to foundation" when the slot is empty. Save-format invalidation: `GameState.schema_version` bumped 1 → 2; old `game_state.json` files silently fall through to "fresh game on launch." Stats / progress / achievements / settings live in separate files and are unaffected. 9 new tests. |
| Drop overlay | `f6c9166` | The pre-existing `update_drop_highlights` system tinted `PileMarker` sprites green for valid drops, but markers were occluded by stacked cards — invisible during real play. New `update_drop_target_overlays` spawns a soft-fill + 3 px outlined box ABOVE cards for every legal target (full fanned column for tableaux, card-sized for foundations / empty tableaux). `Z_DROP_OVERLAY = 50` sits above static cards but below `DRAG_Z = 500` so the dragged card never gets occluded. Reuses `STATE_SUCCESS` hue. The original marker-tint system is untouched. 3 new tests. |
| Drop shadows | `f712b89` | Each `CardEntity` spawns a `CardShadow` child sprite — neutral black at 25 % alpha, sized `card_size + 4 px`, offset `(2, -3)`, local z `-0.05`. `update_card_shadows_on_drag` snaps shadows in `DragState.cards` to a lifted state (40 % alpha, `(4, -6)` offset, `(8, 8)` padding). `resize_cards_in_place` extended to keep shadows cheap on window resize. `update_card_entity`'s `despawn_related` is followed by a fresh `add_card_shadow_child` so flips / theme swaps re-attach shadows. Pure `card_shadow_params(is_dragged)` helper unit-tested. 4 new tests. |
| Stock badge | `655dfde` | A small `·N` chip at the top-right corner of the stock pile shows the remaining count. `update_stock_count_badge` spawns a top-level world entity whose `Transform.translation` is recomputed each tick from `LayoutResource`, so window resize / theme swap don't strand it. Hides via `Visibility::Hidden` when the stock empties — the existing `↺` `StockEmptyLabel` takes over and they never co-render. `Z_STOCK_BADGE = 30` sits between cards and `Z_DROP_OVERLAY`. 4 new tests. |

## Open punch list — release prep (still deferred unless player chooses now)

1. **Cut `v0.11.0`** — meaningful slice since `v0.10.0`: full card-theme system (CARD_PLAN phases 1–7 + theme picker + hayeah art), HUD overhaul (band + fade), session 6's four bug fixes, and session 7's font fallback + four UX feel wins. (`git tag -d v0.1.0` first to clean up the stale local tag.)
2. **README + CHANGELOG refresh** — README was last touched at `a6b8348` before the Settings picker shipped; doesn't mention card themes, the auto-fade, or any of session 7's UX work.
3. **Desktop packaging** per `ARCHITECTURE.md §17`. The Arch PKGBUILD exists in `/home/manage/solitaire-quest-pkgbuild/` (separate repo, no remote yet). Pending: app icon, macOS `.icns` + notarisation cert, Windows `.ico` + Authenticode cert, AppImage recipe.

## Open punch list — UX iteration (next-round candidates)

The session-6 list is exhausted. Candidates for a next round, none formally requested by the player:

- **Animated focus ring** (currently a static overlay; could pulse on focus change).
- **Achievement onboarding pass** — show first-time players the achievement panel after their first win.
- **Mode-switch keyboard shortcut** from inside the Mode Launcher (today only mouse opens it).
- **Runtime aspect-ratio fidelity** — hayeah SVGs are ~1.45 h/w; engine layout assumes 1.4. Cards render ~3 % squashed vertically. Cosmetic.
- **Foundation completion celebration** — when a foundation reaches its King, do a small flourish (sparkle, lift, sound). The auto-complete cascade already covers the win moment, but per-foundation closure is currently silent.
- **Drag-cancel return animation** — illegal drops snap cards back instantly. A short ease-back tween ("springs back to where it came from") would feel more forgiving.

## Card-theme system (CARD_PLAN.md, fully shipped)

Seven phases landed across `b8fb3fb` → `924a1e2`. End-to-end:

- **Bundled default theme** ships in the binary via `embedded://` — 52 hayeah/playing-cards-assets SVGs (MIT) + a midnight-purple `back.svg` (original work).
- **User themes** live under `themes://` rooted at `solitaire_engine::assets::user_theme_dir()`. Drop a directory containing `theme.ron` + 53 SVG files; appears in the registry on next launch.
- **Importer** at `solitaire_engine::theme::import_theme(zip)` validates an archive (20 MB cap, zip-slip rejection, manifest validation, every SVG round-tripped through the rasteriser) and atomically unpacks.
- **Picker UI** in Settings → Cosmetic — one chip per registered theme; selection persists to `settings.json` as `selected_theme_id` and propagates to live sprites via `react_to_settings_theme_change` → `sync_card_image_set_with_active_theme` → `StateChangedEvent`.

## Resume prompt

```
You are a senior Rust + Bevy developer working on Solitaire Quest.
Working directory: <Rusty_Solitaire clone path on this machine — local
directory may still be named Rusty_Solitare from earlier; that's fine>.
Branch: master. Direction is OPEN — the session-6 UX punch list is
fully shipped. The player will choose between cutting v0.11.0, doing
release prep (README/CHANGELOG/packaging), or starting a new UX
iteration round.

State: HEAD=655dfde. Local master is 3 commits ahead of origin
(f6c9166, f712b89, 655dfde unpushed; fdb6c2e and 95df542 already
pushed). Working tree clean apart from untracked CARD_PLAN.md
(intentional).
Build: cargo clippy --workspace --all-targets -- -D warnings clean.
Tests: 982 passed / 0 failed.

READ FIRST (in order, before doing anything):
  1. SESSION_HANDOFF.md  — full state, session 7 changelog, punch list
  2. CLAUDE.md           — hard rules (UI-first, no panics, etc.)
  3. ARCHITECTURE.md     — crate responsibilities + data flow
  4. ~/.claude/projects/<this-project>/memory/MEMORY.md
                         — saved feedback / project context (machine-local;
                           may be missing on a fresh machine)

DECISION TO ASK THE PLAYER FIRST:
  A. Push the 3 unpushed commits and cut v0.11.0?
  B. Skip the tag for now, refresh README + CHANGELOG, then tag?
  C. Skip release prep entirely and start a new UX iteration round?
     If C, see the session-7 next-round candidates list (animated
     focus ring, achievement onboarding, mode-switch keyboard
     shortcut, aspect-ratio fidelity, foundation completion flourish,
     drag-cancel return tween).

WORKFLOW NOTES:
  - Commits use:
      git -c user.name=funman300 -c user.email=root@vscode.infinity \
          commit -m "..."
  - Sub-agents stage + verify only; orchestrator commits.
  - Every commit must pass build / clippy / test before pushing.
  - Push to GitHub (origin) — that is the canonical remote.

OPEN AT THE START: ask which of A / B / C. Don't pick unilaterally —
this is a directional choice, not a tactical one.
```
