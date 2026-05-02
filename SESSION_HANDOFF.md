# Solitaire Quest — UX Overhaul Session Handoff

**Last updated:** 2026-05-02 (session 7, late) — Second UX iteration round complete. Six small UX feel items shipped on top of v0.11.0 and the README/CHANGELOG refresh that should have ridden along. Ready to tag v0.12.0.

## Status at pause

- **HEAD:** `7dba772` plus the impending CHANGELOG/handoff doc commits. Local master is **3 commits ahead** of `origin/master` (`9887343`, `ca5788f`, `7dba772` unpushed); doc commits to follow.
- **Working tree:** clean apart from this doc + CHANGELOG, both intentional.
- **Build:** `cargo clippy --workspace --all-targets -- -D warnings` clean.
- **Tests:** **1007 passed / 0 failed** across the workspace (+25 from session 7 morning's 982 baseline).
- **Tags on origin:** `v0.9.0`, `v0.10.0`, `v0.11.0`. Local has `v0.11.0` too. v0.12.0 is the next tag.

## Where we are

v0.11.0 shipped the headline structural changes (card themes, HUD overhaul, four UX feel wins, font fallback). The second UX round — six smaller items — is also done now. v0.12.0 is the right slice for them; together with the README refresh and CHANGELOG add it makes a clean release.

The post-v0.11.0 UX candidate list is exhausted. Direction is open again.

### Design direction (unchanged)

- **Tone:** Balatro — chunky readable type, theatrical hierarchy, satisfying micro-interactions.
- **Palette:** Midnight Purple base + Balatro yellow primary + warm magenta secondary.
- See `~/.claude/projects/-home-manage-Rusty-Solitare/memory/project_ux_overhaul_2026-04.md` (auto-memory; on a different machine, recreate this fresh from the README + ARCHITECTURE.md).

### Canonical remote

`github.com/funman300/Rusty_Solitaire` is the canonical repo. Always push there. (Local clone directories may still be named `Rusty_Solitare`; that's just a directory name and works fine.)

## Session 7 round 1 (shipped 2026-05-02 morning) — v0.11.0

| Area | Commit | What landed |
|---|---|---|
| Font fallback | `fdb6c2e` | FiraMono bundled into the SVG fontdb so cards render rank/suit text on machines without Bitstream Vera Sans / Arial. Surfaced when a second-machine pull lost glyphs. |
| Unlock foundations | `95df542` | `PileType::Foundation(Suit)` → `Foundation(u8)` with claim derived from the bottom card. Save schema 1 → 2; pre-v2 saves silently fall through to fresh game. |
| Drop overlay | `f6c9166` | Soft fill + 3 px outline drawn ABOVE stacked cards for every legal target during drag. Replaces the hidden pile-marker tint. |
| Drop shadows | `f712b89` | Each card casts a 25 % black shadow; lifts to 40 % with bigger offset/halo while in the active drag set. |
| Stock badge | `655dfde` | "·N" chip at the top-right of the stock so players can see how close they are to a recycle. Hides at zero. |

Tagged as `v0.11.0` (commit `063269c` plus URL refresh).

## Session 7 round 2 (shipped 2026-05-02 afternoon) — v0.12.0

| Area | Commit | What landed |
|---|---|---|
| Aspect ratio | `13aa0fd` | `CARD_ASPECT` 1.4 → 1.4523 to match hayeah SVG dimensions; cards no longer ~3.6 % squashed. Vertical-budget math adapts via the constant. |
| Foundation flourish | `69ce9af` | King-on-foundation celebration: scale-pulse on the King, marker tints `STATE_SUCCESS`, synthesised C6→E6→G6 bell ping (~240 ms). New `FoundationCompletedEvent`. |
| Drag-cancel tween | `525fe0f` | Illegal drops glide each card back to its origin over 150 ms with a quintic ease-out (Responsive curve, zero overshoot). Audio cue still fires. ShakeAnim retained for non-drag rejection paths. |
| Focus pulse | `9887343` | Focus ring breathes at 1.4 s sin period over [0.65, 1.0] of native alpha. Static under `AnimSpeed::Instant`. |
| Achievement onboarding | `ca5788f` | First-win toast "First win! Press A to see your achievements." plus persisted `shown_achievement_onboarding` flag so the cue fires exactly once. |
| Mode Launcher shortcuts | `7dba772` | Digit 1–5 inside the Mode Launcher launches Classic / Daily / Zen / Challenge / TimeAttack. Locked modes silent no-op. Modal-scoped. |
| Docs (rode along) | `d8c7034`, `9f095c4` | README refresh for v0.11.0 features and corrected controls table; CHANGELOG.md added covering v0.9.0–v0.11.0. |

The first three items in this round (`13aa0fd`, `69ce9af`, `525fe0f`) shipped before the v0.11.0 tag's commit window closed; treating them as v0.12.0 since v0.11.0 was already cut at `063269c`.

## Open punch list — release prep

1. **Tag v0.12.0** — meaningful slice since v0.11.0: six UX feel items + the README/CHANGELOG refresh. Tag at the doc-commit HEAD that closes this round.
2. **Push to origin** — three-plus commits unpushed.
3. **Desktop packaging** per `ARCHITECTURE.md §17`. The Arch PKGBUILD exists in `/home/manage/solitaire-quest-pkgbuild/` (separate repo, no remote yet). Pending: app icon, macOS `.icns` + notarisation cert, Windows `.ico` + Authenticode cert, AppImage recipe.

## Open punch list — UX iteration (next-round candidates)

The v0.12.0 list is exhausted. Candidates for a future round:

- **Card-back theme support** — the current theme system swaps face SVGs but not the back. Players asked for animated backs in passing.
- **Streak fire animation** in the HUD when win-streak crosses 3, 5, 10. Foundation flourish suggests the per-suit completion pattern; streak milestones are the lifetime equivalent.
- **Score-breakdown reveal** at win — show base / time-bonus / no-undo bonus / mode multiplier as the score animates up. Currently the win modal just shows the final number.
- **Right-click radial menu** for power users: hold right-click on a card → quick-drop options without dragging.
- **Drag-with-keyboard** — Tab to a card, Enter to "lift", arrow keys to choose destination, Enter to drop. Keyboard-only completion of a game.
- **Settings: tooltip-delay slider** so power users can disable the 0.5 s hover delay. Cheap.

## Card-theme system (CARD_PLAN.md, fully shipped)

Seven phases landed across `b8fb3fb` → `924a1e2` in v0.11.0:

- **Bundled default theme** ships in the binary via `embedded://` — 52 hayeah/playing-cards-assets SVGs (MIT) + a midnight-purple `back.svg` (original work).
- **User themes** live under `themes://` rooted at `solitaire_engine::assets::user_theme_dir()`. Drop a directory containing `theme.ron` + 53 SVG files; appears in the registry on next launch.
- **Importer** at `solitaire_engine::theme::import_theme(zip)` validates an archive and atomically unpacks.
- **Picker UI** in Settings → Cosmetic.

## Resume prompt

```
You are a senior Rust + Bevy developer working on Solitaire Quest.
Working directory: <Rusty_Solitaire clone path on this machine — local
directory may still be named Rusty_Solitare from earlier; that's fine>.
Branch: master. Direction is OPEN — both UX iteration rounds shipped
and v0.12.0 is ready to tag.

State: HEAD at the doc-commit closing session 7 round 2. Local master
is several commits ahead of origin and unpushed. Working tree clean
apart from untracked CARD_PLAN.md (intentional).
Build: cargo clippy --workspace --all-targets -- -D warnings clean.
Tests: 1007 passed / 0 failed.

READ FIRST (in order, before doing anything):
  1. SESSION_HANDOFF.md  — full state, session 7 changelog, punch list
  2. CHANGELOG.md        — release-by-release record
  3. CLAUDE.md           — hard rules (UI-first, no panics, etc.)
  4. ARCHITECTURE.md     — crate responsibilities + data flow
  5. ~/.claude/projects/<this-project>/memory/MEMORY.md
                         — saved feedback / project context (machine-local;
                           may be missing on a fresh machine)

DECISION TO ASK THE PLAYER FIRST:
  A. Push the unpushed commits and cut v0.12.0 now.
  B. Smoke-test the new feel layer first (foundation flourish, drag
     tween, focus pulse, mode digits), then tag.
  C. Skip the tag for another iteration round — see "next-round
     candidates" in SESSION_HANDOFF for ideas.
  D. Take the deferred desktop-packaging item (needs artwork +
     signing certs from the user).

WORKFLOW NOTES:
  - Commits use:
      git -c user.name=funman300 -c user.email=root@vscode.infinity \
          commit -m "..."
  - Sub-agents stage + verify only; orchestrator commits.
  - Every commit must pass build / clippy / test before pushing.
  - Push to GitHub (origin) — that is the canonical remote.

OPEN AT THE START: ask which of A / B / C / D. Don't pick unilaterally.
```
