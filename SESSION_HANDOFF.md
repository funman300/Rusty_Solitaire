# Solitaire Quest — UX Overhaul Session Handoff

**Last updated:** 2026-05-02 (session 7, late-late) — Third UX iteration round complete on top of v0.12.0. Six post-handoff candidates shipped plus two code-review fixes. Ready to tag v0.13.0.

## Status at pause

- **HEAD:** doc-commit closing this round (CHANGELOG + handoff). Local master has the impending tag at this commit.
- **Working tree:** clean apart from untracked `CARD_PLAN.md` (intentional).
- **Build:** `cargo clippy --workspace --all-targets -- -D warnings` clean.
- **Tests:** **1053 passed / 0 failed** across the workspace (+22 from v0.12.0's 1031 baseline).
- **Tags on origin:** `v0.9.0`, `v0.10.0`, `v0.11.0`, `v0.12.0`. v0.13.0 is the next tag.

## Where we are

Post-v0.12.0 the handoff listed six "next-round candidates" — every one shipped today plus two code-review fixes (font handling unified to bundled-only, sccache wiring removed). v0.13.0 is the right slice.

The candidate list is exhausted again. Direction is open.

### Design direction (unchanged)

- **Tone:** Balatro — chunky readable type, theatrical hierarchy, satisfying micro-interactions.
- **Palette:** Midnight Purple base + Balatro yellow primary + warm magenta secondary.
- See `~/.claude/projects/-home-manage-Rusty-Solitare/memory/project_ux_overhaul_2026-04.md` (machine-local).

### Canonical remote

`github.com/funman300/Rusty_Solitaire` is the canonical repo. Always push there.

## Session 7 round 3 (shipped 2026-05-02 late-late) — v0.13.0

| Area | Commit | What landed |
|---|---|---|
| Font fix | `17f9b51` | Code-review fix: bundle FiraMono via `include_bytes!()` in both `font_plugin` and `svg_loader`; drop `load_system_fonts`, drop the lenient resolver, drop the CSS-generic fallbacks. New `bundled_font_resolver` always returns the single bundled face. Parse failure aborts with a clear error. |
| sccache removal | `13dd44b` | Code-review fix: deleted `.cargo/config.toml` and the `.cargo` directory. Plain `cargo build` works without per-project setup. |
| Wave 1 bundle | `ddc8f27` | **Tooltip-delay slider** in Settings → Gameplay (0.0–1.5 s, 0.1 s steps, "Instant" label at zero). **Win-streak fire animation** at thresholds [3, 5, 10] via new `WinStreakMilestoneEvent`. **Score-breakdown reveal on win modal** with per-row stagger (Base / Time bonus / No-undo / Multiplier / Total), respects `AnimSpeed::Instant`. |
| Card-back theming | `7ed4f2c` | The active theme's `back.svg` now actually drives the face-down sprite. Legacy `back_N.png` picker remains as a fallback for themes without a back; Settings caption surfaces when the override is in effect. |
| Drag-with-keyboard | `a0fc0d2` | Tab → Enter → arrows → Enter completes a move without a mouse. New `KeyboardDragState` resource; mutual exclusion with mouse drag via `KEYBOARD_DRAG_TOUCH_ID` sentinel. Help + onboarding hotkey lists updated. |
| Right-click radial | `b37f0cb` | Hold RMB on a face-up card → ring of icons at the cursor, one per legal destination; release over an icon → `MoveRequestEvent`. New `RadialMenuPlugin`. Help controls reference gains a "Mouse" section. |

## Open punch list — release prep

1. **Push** the unpushed commits to origin (5 commits now: 17f9b51, 13dd44b, ddc8f27, 7ed4f2c, a0fc0d2, b37f0cb, plus the impending doc commit).
2. **Tag v0.13.0** at the doc-commit HEAD.
3. **Desktop packaging** per `ARCHITECTURE.md §17`. The Arch PKGBUILD exists in `/home/manage/solitaire-quest-pkgbuild/` (separate repo). Pending: app icon, macOS `.icns` + notarisation cert, Windows `.ico` + Authenticode cert, AppImage recipe.

## Open punch list — UX iteration (next-round candidates)

The v0.13.0 list is exhausted. Fresh candidates for a future round:

- **In-game daily-challenge calendar** — currently the daily challenge fires once on launch; a Settings or Profile-side calendar showing past days' completion / streak status would make the progression visible.
- **Card-art preview in the theme picker** — Settings → Cosmetic shows theme name only; rendering the theme's Ace-of-Spades + back side-by-side as a thumbnail would make picking faster.
- **Per-mode high-score readout** in the Stats screen. Currently lifetime stats roll all modes together.
- **Auto-save in-progress games** in Zen / Time Attack so players who close the window mid-session don't lose their state.
- **Configurable scoring weights** for the curious — Settings → Gameplay slider for time-bonus magnitude. Cosmetic but power-user appealing.
- **Replay a winning game** — record the seed + move list at win time and offer "watch replay" from the Stats screen.

## Card-theme system (CARD_PLAN.md, fully shipped)

Seven phases landed across `b8fb3fb` → `924a1e2` in v0.11.0; v0.13.0's `7ed4f2c` finally consumes the per-theme `back.svg`. End-to-end:

- **Bundled default theme** ships in the binary via `embedded://` — 52 hayeah/playing-cards-assets SVGs + a midnight-purple `back.svg`.
- **User themes** under `themes://`. Drop a directory containing `theme.ron` + 53 SVGs.
- **Importer** at `solitaire_engine::theme::import_theme(zip)` validates archives and atomically unpacks.
- **Picker UI** in Settings → Cosmetic; the active theme's `back` overrides the legacy `back_N.png` picker when present.

## Resume prompt

```
You are a senior Rust + Bevy developer working on Solitaire Quest.
Working directory: <Rusty_Solitaire clone path on this machine — local
directory may still be named Rusty_Solitare from earlier; that's fine>.
Branch: master. Direction is OPEN — three UX iteration rounds shipped
and v0.13.0 is ready to tag.

State: HEAD at the doc-commit closing session 7 round 3. Local master
is several commits ahead of origin and unpushed. Working tree clean
apart from untracked CARD_PLAN.md (intentional).
Build: cargo clippy --workspace --all-targets -- -D warnings clean.
Tests: 1053 passed / 0 failed.

READ FIRST (in order, before doing anything):
  1. SESSION_HANDOFF.md  — full state, session 7 changelog, punch list
  2. CHANGELOG.md        — release-by-release record
  3. CLAUDE.md           — hard rules (UI-first, no panics, etc.)
  4. ARCHITECTURE.md     — crate responsibilities + data flow
  5. ~/.claude/projects/<this-project>/memory/MEMORY.md
                         — saved feedback / project context (machine-local;
                           may be missing on a fresh machine)

DECISION TO ASK THE PLAYER FIRST:
  A. Push and cut v0.13.0 now.
  B. Smoke-test the new feel layer first (theme-aware backs, keyboard
     drag, right-click radial, score-breakdown reveal, streak fire,
     tooltip-delay slider), then tag.
  C. Skip the tag for another iteration round — see "next-round
     candidates" in SESSION_HANDOFF for fresh ideas.
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
