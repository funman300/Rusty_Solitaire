# Solitaire Quest — UX Overhaul Session Handoff

**Last updated:** 2026-05-02 (session 8, post-Quat smoke test) — Quat playtested and reported four bugs + two investigation tasks. Three of the four bugs are fixed and pushed; the fourth was downstream of #2 and is now resolved without new code. Replay-feature WIP is back in the working tree.

## Status at pause

- **HEAD on origin:** `2716472`. Three bug-fix commits pushed (`f1aeb24`, `3eabc14`, `2716472`) on top of the v0.13.0-era HEAD `0001432`.
- **Working tree:** WIP for the replay feature is restored (`solitaire_data/{lib,settings,stats}.rs`, `solitaire_data/src/replay.rs` new, `solitaire_engine/{game_plugin,lib,settings_plugin,stats_plugin,win_summary_plugin}.rs`, `solitaire_sync/{merge,stats}.rs`). Plus untracked `CARD_PLAN.md` (intentional).
- **Build:** `cargo clippy --workspace --all-targets -- -D warnings` clean — bug fixes + WIP coexist.
- **Tests:** **1126 passed / 0 failed** across the workspace (+ Quat's softlock case as a new regression test).
- **Tags on origin:** `v0.9.0`, `v0.10.0`, `v0.11.0`, `v0.12.0`. v0.13.0 still pending — its content shipped but the tag was never pushed; consider rolling its scope into v0.14.0 along with the bug fixes + replay feature.

## Where we are

Post-v0.12.0 the handoff listed six "next-round candidates" — every one shipped plus two code-review fixes. v0.13.0 was prepared but not pushed. Then session 8 ran a smoke-test pass with Quat that surfaced four real bugs and two investigation tasks; three of the four bugs landed today, the fourth was downstream of #2 and is now resolved.

Direction is open.

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

## Session 8 (shipped 2026-05-02 post-Quat)

Quat playtested current `master` and reported 4 bugs + 2 investigation items. Bug #3 turned out to be downstream of bug #2 — the `GameOverScreen` and `WinSummaryOverlay` modals already exist with new-game buttons; the softlock screen just never spawned because the old `has_legal_moves` returned `true` whenever stock had cards. With #2 fixed, the existing screen will fire for Quat's case. Smoke-test verification on the player side is the last step.

| Area | Commit | What landed |
|---|---|---|
| Move validation (#1) | `f1aeb24` | `solitaire_core::rules::is_valid_tableau_sequence(&[Card]) -> bool` checks every adjacent pair in a moved stack descends one rank with alternating colour. Wired into `move_cards`'s tableau-destination branch. Closes the bug where a player could lift an arbitrary multi-card selection and drop it as long as the bottom landed legally. One focused test (single-card / valid-run / same-colour / rank-gap). |
| Deal-tween leak (#4) | `3eabc14` | `handle_new_game` now snaps every existing card sprite to the stock pile's position before writing `StateChangedEvent`. The downstream slide tween in `card_plugin` reads the stock position as its source, so all 52 cards animate from a single point — reads as "dealing from the deck" with no information leak. Gated on `Option<Res<LayoutResource>>` for headless tests. |
| Softlock detection (#2) | `2716472` | `has_legal_moves` rewritten: replaces the early-return-on-non-empty-stock heuristic with a single pass over every card that could ever become a move source (every stock card, every waste card, the face-up top of every tableau column). Each is checked against every foundation and every tableau. Returns `true` only if some card anywhere can land somewhere — otherwise the player is genuinely stuck no matter how many recycles. Fresh-game test renamed; new test reproduces Quat's exact case (foundation 0 at 10, stock holds Hearts 2–5, no legal landing). |
| End-game screen (#3) | — | Resolved as downstream of #2. Verified `GameOverScreen` (game_plugin.rs:636) shows "No more moves available" + final score + Undo + New Game buttons; `WinSummaryOverlay` (win_summary_plugin.rs) shows the breakdown + Play Again. Both pre-existed; the softlock path just wasn't being reached. |

### Investigation findings

**Solver / unwinnable-deals decision (Quat report):** still open — needs your call. Three options Quat outlined: (a) accept some deals are unwinnable, (b) run a solver at deal-time and only ship winnable layouts, (c) offer a "winnable deals only" mode. (b) is the modern-Solitaire standard but adds a dependency or hand-rolled solver (~500–1500 LOC). (c) is the lightest middle ground — keep classic deals available, add a Settings toggle. Recommendation: defer until other UX work settles; doesn't block any release.

**Dependency duplicates (Quat report — 1014 deps):** the biggest single bloat is the audio stack split. Bevy's default features pull `bevy_audio → rodio → cpal 0.15 + alsa 0.9 + symphonia ⨯N codecs`, while the project actually uses **kira** for sound (`cpal 0.17 + alsa 0.10`). Disabling Bevy's default `bevy_audio` feature would eliminate 20+ transitive crates including the rodio + symphonia chain. Other duplicates are minor (bitflags 1.x via `png` is build-tooling only; multiple hashbrown majors are common in the Bevy/wgpu ecosystem and not actionable). Recommendation: a one-line `default-features = false` swap on the workspace `bevy =` line, then re-enable explicitly the features the engine uses (`render`, `bevy_winit`, `2d`, `bevy_window`, `png`, `bevy_text`, `bevy_ui`, `bevy_log`, `bevy_asset`, `default_font`, `bevy_state`, `webgpu`/`webgl2` if targeting wasm). Worth ~50 fewer crates compiled. Defer until the active feature work settles so churn doesn't conflict.

## Open punch list — release prep

1. **Push** the unpushed feature commits to origin (5 still unpushed: `b37f0cb`, `a0fc0d2`, `7ed4f2c`, `ddc8f27`, `13dd44b`/`17f9b51` — those last two were on the v0.13.0 round, never pushed; verify with `git log --oneline origin/master..HEAD` after committing replay).
2. **Roll v0.13.0 + replay + bug fixes into v0.14.0** rather than tagging two close releases. The bug fixes alone aren't a feature release; bundle them with the replay feature when it lands.
3. **Desktop packaging** per `ARCHITECTURE.md §17`. The Arch PKGBUILD exists in `/home/manage/solitaire-quest-pkgbuild/` (separate repo). Pending: app icon, macOS `.icns` + notarisation cert, Windows `.ico` + Authenticode cert, AppImage recipe.
4. **Smoke-test the bug fixes** on the alex machine after pulling: confirm (a) tableau-to-tableau invalid-stack moves are now rejected, (b) the new-game deal animates from a single deck position with no per-card origin leak, (c) softlock with unplayable stock now spawns the GameOverScreen.

## Open punch list — UX iteration (next-round candidates)

Several v0.13.0-era candidates have already shipped to master since the v0.13.0 doc commit: **daily-challenge calendar** (`1a10476`), **card-art thumbnails in the theme picker** (`ba527de`), **auto-save in Time Attack** (`0001432`). Replay is **WIP in the working tree** — `solitaire_data/src/replay.rs` plus modifications across stats/settings/win-summary plugins. Not yet committed.

Fresh candidates not yet started:

- **Per-mode high-score readout** in the Stats screen. Currently lifetime stats roll all modes together.
- **Auto-save Zen mode** alongside Time Attack so close-mid-session resumes work in both.
- **Configurable scoring weights** — Settings → Gameplay slider for time-bonus magnitude. Cosmetic but power-user appealing.
- **Solver-at-deal toggle** (Quat's investigation #1, deferred): per the recommendation in the session-8 findings, add a Settings toggle "Winnable deals only" rather than baking solver-only into all deals. Lightest middle ground.
- **Disable Bevy's default audio feature** (Quat's investigation #2, deferred) to drop ~50 transitive crates. One-line workspace edit then re-enable engine features explicitly. Defer until active feature work settles.

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
Branch: master. Direction is OPEN — Quat's smoke-test bug round
landed (3 fixes pushed, 1 resolved as downstream); replay feature is
WIP in the working tree.

State: origin/master at 2716472 (Quat's softlock fix). Working tree
has uncommitted replay-feature WIP across solitaire_data,
solitaire_engine, solitaire_sync — `solitaire_data/src/replay.rs` is
new. Plus untracked CARD_PLAN.md (intentional). Five feature commits
from v0.13.0 round are unpushed (b37f0cb, a0fc0d2, 7ed4f2c, ddc8f27,
plus the v0.13.0 doc commits) — verify with
`git log --oneline origin/master..HEAD`.
Build: cargo clippy --workspace --all-targets -- -D warnings clean.
Tests: 1126 passed / 0 failed (includes Quat's softlock regression).

READ FIRST (in order, before doing anything):
  1. SESSION_HANDOFF.md  — session 8 changelog + punch list
  2. CHANGELOG.md        — release-by-release record
  3. CLAUDE.md           — hard rules (UI-first, no panics, etc.)
  4. ARCHITECTURE.md     — crate responsibilities + data flow
  5. ~/.claude/projects/<this-project>/memory/MEMORY.md
                         — saved feedback / project context (machine-local;
                           may be missing on a fresh machine)

DECISION TO ASK THE PLAYER FIRST:
  A. Finish the replay-feature WIP, commit, then bundle everything
     (replay + v0.13.0 round + Quat fixes) into v0.14.0.
  B. Smoke-test the bug fixes on alex first to confirm Quat's three
     issues are resolved in real gameplay.
  C. Take the deferred Bevy-audio-feature trim (Quat investigation
     #2): drop default-features and re-enable explicitly. Worth ~50
     fewer crates.
  D. Take the deferred solver toggle (Quat investigation #1): add
     "Winnable deals only" Settings toggle.
  E. Pick from the remaining "next-round candidates" in this doc.
  F. Take the deferred desktop-packaging item (needs artwork +
     signing certs from the user).

WORKFLOW NOTES:
  - Commits use:
      git -c user.name=funman300 -c user.email=root@vscode.infinity \
          commit -m "..."
  - When attributing playtester feedback in commits/docs, use "Quat"
    not "Rhys" (saved feedback memory).
  - Sub-agents stage + verify only; orchestrator commits.
  - Every commit must pass build / clippy / test before pushing.
  - Push to GitHub (origin) — that is the canonical remote.

OPEN AT THE START: ask which of A–F. Don't pick unilaterally.
```
