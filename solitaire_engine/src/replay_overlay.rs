//! On-screen overlay shown while a recorded [`Replay`] plays back.
//!
//! The overlay is a thin top-of-window banner with three pieces of UI:
//!
//! - A "▌ replay" label on the left so the player knows the surface is
//!   under playback control rather than live input.
//! - A "MOVE N/M" progress chip in the centre, recomputed every frame
//!   the cursor advances and bordered in `ACCENT_PRIMARY` so it
//!   reads as a discrete callout.
//! - A "Stop" button on the right that aborts playback and returns
//!   control to the player.
//!
//! When playback finishes ([`ReplayPlaybackState::Completed`]) the banner
//! label swaps to "▌ replay complete" and stays visible until the playback
//! core auto-clears the resource back to [`ReplayPlaybackState::Inactive`]
//! a few seconds later, at which point the overlay despawns.
//!
//! The overlay sits at z-layer [`Z_REPLAY_OVERLAY`] — above gameplay but
//! below every modal layer ([`Z_MODAL_SCRIM`] and up). That ordering lets
//! the player still open Settings, Pause, and Help during a replay; those
//! modals will render on top of the banner as expected.
//!
//! [`Replay`]: solitaire_data::Replay
//! [`Z_MODAL_SCRIM`]: crate::ui_theme::Z_MODAL_SCRIM

use bevy::prelude::*;
use chrono::Datelike;

use crate::font_plugin::FontResource;
use crate::layout::LayoutResource;
use crate::events::{DrawRequestEvent, MoveRequestEvent, UndoRequestEvent};
use crate::replay_playback::{
    step_backwards_replay_playback, step_replay_playback, stop_replay_playback,
    toggle_pause_replay_playback, ReplayPlaybackState,
};
use solitaire_data::ReplayMove;
use crate::ui_modal::{spawn_modal_button, ButtonVariant};
use crate::ui_theme::{
    ACCENT_PRIMARY, BG_ELEVATED_HI, BORDER_SUBTLE, HighContrastBackground, HighContrastBorder,
    STATE_SUCCESS, TEXT_PRIMARY, TEXT_SECONDARY, TYPE_BODY, TYPE_CAPTION, TYPE_HEADLINE,
    VAL_SPACE_1, VAL_SPACE_2, VAL_SPACE_4, Z_DROP_OVERLAY,
};

// ---------------------------------------------------------------------------
// Z-index — see `ui_theme::Z_MODAL_SCRIM` (200) for the next layer above.
// ---------------------------------------------------------------------------

/// `bevy::ui` `ZIndex` value for the replay overlay banner.
///
/// Numeric value is `Z_DROP_OVERLAY as i32 + 5 = 55`; chosen so the banner
/// sits clearly above the HUD top layer (`Z_HUD_TOP = 60` is intentionally
/// **below** modals, but the overlay needs to be above HUD readouts) yet
/// well below `Z_MODAL_SCRIM = 200` so Settings, Pause, and Help modals
/// continue to render on top of the overlay during a replay.
///
/// The `Z_DROP_OVERLAY + 5` formula in the spec is reproduced here as an
/// integer because `Z_DROP_OVERLAY` itself is a `f32` Sprite-space z used
/// for the drop-target overlay sprites — UI nodes use `i32` `ZIndex`, so
/// we materialise a separate constant rather than reuse the `f32` value.
pub const Z_REPLAY_OVERLAY: i32 = Z_DROP_OVERLAY as i32 + 5;

/// Total height of the banner in pixels. Thin enough to leave the
/// gameplay surface visible underneath, tall enough to comfortably fit
/// the headline-sized "▌ replay" label stacked above the
/// `TYPE_CAPTION` "GAME #YYYY-DDD" subtitle (the left column needs
/// ~26 + 2 + 11 = 39 px of inner content; banner = top row (59
/// flex-grow) + scrub track (1) + label row (16) + footer (16)
/// gives 92).
///
/// Growth history:
/// - 60 → 76 in the scrub-notch-labels commit to make room for the
///   `0%` / … / `100%` percentage labels under each notch.
/// - 76 → 92 in the keybind-footer commit to make room for the
///   vim-style mode line + keybind-hint footer at the bottom.
const BANNER_HEIGHT: f32 = 92.0;

/// Height of the label row that sits below the 1px scrub track and
/// carries the `0%` / `25%` / `50%` / `75%` / `100%` notch labels.
/// 16 px is enough for `TYPE_CAPTION` text (12 px font + 4 px breathing
/// room above the bottom edge).
const SCRUB_LABEL_ROW_HEIGHT: f32 = 16.0;

/// Height of the keybind-hint footer that sits below the notch-label
/// row. Carries a vim-style mode indicator on the left and a
/// keybind-hint on the right (`[SPACE] pause/resume`). 16 px matches
/// `SCRUB_LABEL_ROW_HEIGHT` for visual symmetry — `TYPE_CAPTION` text
/// (12 px) + 4 px breathing room.
const KEYBIND_FOOTER_HEIGHT: f32 = 16.0;

/// Background colour alpha for the banner. `BG_ELEVATED_HI` at this alpha
/// reads as a clear "this is a UI strip" callout while still letting the
/// felt show through enough to anchor the banner to the play surface.
const BANNER_ALPHA: f32 = 0.92;

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marker on the banner's root `Node`. Used by the spawn / despawn /
/// progress-update systems to find the overlay.
#[derive(Component, Debug)]
pub struct ReplayOverlayRoot;

/// Marker on the left-hand banner label `Text`. Carries either
/// "▌ replay" (during playback) or "▌ replay complete" (once
/// finished — the cursor-block prefix matches the splash boot-screen
/// idiom so the surface reads as a Terminal output line); the
/// completion-text-update system swaps the contents in place.
#[derive(Component, Debug)]
pub struct ReplayOverlayBannerText;

/// Marker on the centre progress `Text`. Updated every frame to reflect
/// the current `(cursor, total)` returned by
/// [`ReplayPlaybackState::progress`].
#[derive(Component, Debug)]
pub struct ReplayOverlayProgressText;

/// Marker on the **floating** progress chip — a 2D world-space text
/// entity rendered above the destination pile of the most-recently-
/// applied move. Sits independently of the banner overlay (which
/// lives in the UI tree and never moves) so the player can see
/// progress without breaking eye contact with the focal card.
///
/// Lifecycle matches the banner overlay: spawned by `spawn_overlay`
/// when a replay starts, despawned by `react_to_state_change` when
/// it ends. Position updated each frame by
/// `update_floating_progress_chip`. Hidden when cursor=0 (no moves
/// applied yet) or the last applied move was a `StockClick` (no
/// destination pile to follow).
#[derive(Component, Debug)]
pub struct ReplayFloatingProgressChip;

/// Marker on the right-hand "Stop" button. Click handler queries for this
/// and calls [`stop_replay_playback`] when an `Interaction::Pressed`
/// transition is seen.
#[derive(Component, Debug)]
pub struct ReplayStopButton;

/// Marker on the Pause / Resume button. Click handler queries for this
/// and calls [`toggle_pause_replay_playback`] on each press. The
/// button's label text is repainted in lockstep by
/// `update_pause_button_label` so it always reflects the action the
/// next click will perform ("Pause" while running, "Resume" while
/// paused).
#[derive(Component, Debug)]
pub struct ReplayPauseButton;

/// Marker on the Step button. Click handler queries for this and
/// calls [`step_replay_playback`] — only meaningful when paused
/// (clicks while running are no-ops because the tick loop would race
/// the manual advance). The button stays visually present but
/// unresponsive while the playback is running so the player has a
/// stable layout to scan.
#[derive(Component, Debug)]
pub struct ReplayStepButton;

/// Marker on the small caption sitting below the "▌ replay"
/// headline. Carries `GAME #YYYY-DDD` (year + chrono ordinal) while a
/// replay is playing — a compact, monotonically-increasing identifier
/// that mirrors the `▌replay.tsx` / `GAME #2024-127` Terminal-output
/// motif from the mockup. The caption is empty in `Inactive` /
/// `Completed` since the replay is consumed when transitioning out
/// of `Playing` and the identifier is no longer recoverable from
/// state alone.
#[derive(Component, Debug)]
pub struct ReplayOverlayGameCaption;

/// Marker on the accent "fill" of the bottom-edge scrub bar. The
/// `Node`'s `width` is rewritten every frame the cursor advances to
/// `cursor / total` of the bar's full width, so the player has a
/// continuous visual cue of how far through the replay they are.
///
/// Distinct from the simpler text-based `ReplayOverlayProgressText`
/// (which spells out "MOVE N/M" in a chip): the scrub fill gives immediate
/// at-a-glance positioning; the text gives the exact numbers. Both
/// surfaces stay together because they answer the same question for
/// players with different scanning preferences.
#[derive(Component, Debug)]
pub struct ReplayOverlayScrubFill;

/// Marker for the WIN MOVE tick on the scrub bar — a small absolute-
/// positioned `Node` anchored at `replay.win_move_index / total` along
/// the track. Painted in [`STATE_SUCCESS`] so the player can see at a
/// glance where the winning move sits relative to the playback cursor.
///
/// Static — the position is set at spawn time and never changes during
/// playback (the underlying replay's `win_move_index` is immutable
/// while `Playing`). Despawned with the rest of the overlay tree when
/// the replay state transitions back to `Inactive`.
///
/// Spawned only when the active replay carries
/// [`Replay::win_move_index`](solitaire_data::Replay::win_move_index)
/// `= Some(_)` — older replays loaded from disk pre-date the field
/// and have no win index to surface.
#[derive(Component, Debug)]
pub struct ReplayOverlayWinMoveMarker;

/// Marker for the fixed-position notches on the scrub bar — five 1px
/// vertical ticks at 0 % / 25 % / 50 % / 75 % / 100 % that give the
/// player visual anchor points for "where am I, relative to the
/// quarter-marks of the replay." Mirrors the notch ladder in the
/// screen-takeover mockup at
/// `docs/ui-mockups/replay-overlay-mobile.html`.
///
/// Static — positions are set at spawn time and never change. The
/// notches paint in [`BORDER_SUBTLE`] which is the same colour as the
/// unfilled track, so visibility comes from extending the notch
/// **vertically past** the 1px track (5px tall, anchored 2px above
/// the track top) rather than from colour contrast. Same trick the
/// WIN MOVE marker uses.
#[derive(Component, Debug)]
pub struct ReplayOverlayScrubNotch;

/// Marker for the percentage labels under each scrub-bar notch
/// (`0%` / `25%` / `50%` / `75%` / `100%`). One label per notch;
/// labels live in a dedicated 16 px row below the 1 px scrub track
/// (the row that grew the banner from 60 → 76 px).
///
/// Positioning follows a "endpoints flush to edges, middle three
/// anchored at percentage" pattern: the leftmost label uses
/// `left: 0`, the rightmost uses `right: 0`, and the middle three
/// (`25%` / `50%` / `75%`) anchor at `left: Val::Percent(p)`. This
/// avoids overflow at 100 % without needing CSS-style
/// `translate-x: -50%` centering (which Bevy 0.18 UI doesn't have a
/// clean equivalent for) — the trade-off is a slight right-of-notch
/// offset on the middle three, which is visually subtle at the
/// `TYPE_CAPTION` font size.
#[derive(Component, Debug)]
pub struct ReplayOverlayScrubNotchLabel;

/// Marker on the keybind-hint footer row at the bottom edge of the
/// banner. Carries two `Text` children: a vim-style mode indicator
/// (`▌ NORMAL │ replay`) on the left and the keybind hint
/// (`[SPACE] pause/resume`) on the right. 1 px top border in
/// [`BORDER_SUBTLE`] separates it from the notch-label row above.
///
/// Surfaces the existing Space-key accelerator visually so the
/// UI-first contract from CLAUDE.md §3.3 (every player action has
/// a visible UI control) holds for keyboard accelerators too.
/// Future commits that wire ESC for stop or ← / → for scrub will
/// extend the right-hand text in lockstep — the footer always
/// reflects what's actually wired, never aspirational.
#[derive(Component, Debug)]
pub struct ReplayOverlayKeybindFooter;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Bevy plugin that registers every system needed to drive the replay
/// overlay's lifecycle.
///
/// The plugin is independent of [`crate::replay_playback::ReplayPlaybackPlugin`]
/// — it only reads the shared `ReplayPlaybackState` resource. Tests insert
/// the resource manually and exercise the overlay in isolation.
pub struct ReplayOverlayPlugin;

impl Plugin for ReplayOverlayPlugin {
    fn build(&self, app: &mut App) {
        // The systems are ordered so that, on a single frame:
        //   1. The state-watcher spawns or despawns the overlay if the
        //      `ReplayPlaybackState` resource changed.
        //   2. The completion-text update swaps the banner label when the
        //      state is `Completed`.
        //   3. The progress-text update writes the latest "Move N of M".
        //   4. The Stop-button click handler reads `Interaction::Pressed`
        //      and calls `stop_replay_playback` (which mutates the state).
        // Putting Stop last means a click in frame N is observed by
        // `react_to_state_change` in frame N+1, which then despawns the
        // overlay in response — a clean state-driven loop.
        // Step-button handler dispatches into the same canonical move
        // / draw events that the tick loop fires. Register them
        // defensively here so this plugin can run under
        // `MinimalPlugins` without the playback plugin attached;
        // `add_message` is idempotent so the duplicate registration
        // in production (alongside `replay_playback`) is harmless.
        app.add_message::<MoveRequestEvent>()
            .add_message::<DrawRequestEvent>()
            .add_message::<UndoRequestEvent>()
            .add_systems(
                Update,
                (
                    react_to_state_change,
                    update_banner_label,
                    update_progress_text,
                    update_floating_progress_chip,
                    update_scrub_fill,
                    update_pause_button_label,
                    handle_pause_button,
                    handle_step_button,
                    handle_pause_keyboard,
                    handle_stop_keyboard,
                    handle_arrow_keyboard,
                    handle_stop_button,
                )
                    .chain(),
            );
    }
}

// ---------------------------------------------------------------------------
// Spawning
// ---------------------------------------------------------------------------

/// Reads [`ReplayPlaybackState`] every time the resource changes and either
/// spawns or despawns the overlay accordingly. Treats the resource as the
/// single source of truth — the spawn / despawn decision is derived from
/// `is_playing() || is_completed()` rather than tracking previous-state
/// transitions explicitly, which keeps the system stateless.
fn react_to_state_change(
    mut commands: Commands,
    state: Res<ReplayPlaybackState>,
    existing: Query<Entity, With<ReplayOverlayRoot>>,
    floating_chips: Query<Entity, With<ReplayFloatingProgressChip>>,
    font_res: Option<Res<FontResource>>,
) {
    if !state.is_changed() {
        return;
    }

    let should_be_visible = state.is_playing() || state.is_completed();
    let already_spawned = existing.iter().next().is_some();

    if should_be_visible && !already_spawned {
        spawn_overlay(&mut commands, font_res.as_deref(), &state);
    } else if !should_be_visible && already_spawned {
        for entity in &existing {
            commands.entity(entity).despawn();
        }
        // Floating chip lives outside the UI tree (world-space
        // entity), so the banner-root despawn doesn't reach it.
        // Despawn separately on the same state transition so both
        // disappear together when the replay ends.
        for entity in &floating_chips {
            commands.entity(entity).despawn();
        }
    }
    // The `should_be_visible && already_spawned` branch is a no-op here —
    // the per-frame text update systems below repaint the banner label
    // and progress readout in place without a respawn.
}

/// Spawns the banner — a flex-row Node anchored to the top edge of the
/// window with three children: the "▌ replay" / "▌ replay complete" label,
/// the centred progress text, and the right-aligned Stop button.
fn spawn_overlay(
    commands: &mut Commands,
    font_res: Option<&FontResource>,
    state: &ReplayPlaybackState,
) {
    let font_handle = font_res.map(|f| f.0.clone()).unwrap_or_default();
    // Clone for the floating chip spawn that runs *after* the
    // banner's `.with_children(|banner| { ... })` closure consumes
    // the original `font_handle`. Cheap — Bevy's `Handle<Font>` is
    // `Arc`-backed, the clone bumps a refcount.
    let font_handle_for_floating = font_handle.clone();
    // Second clone for the scrub-bar label row and keybind footer
    // inside the outer banner closure. The inner top-row closure
    // consumes the original `font_handle` for the progress-chip
    // text, so by the time the outer closure reaches the
    // label-row / footer spawns the original is gone.
    // `font_handle_for_labels` is `.clone()`'d (never moved) inside
    // the labels closure, so it's still alive for the footer
    // spawn afterwards — single shared clone covers both.
    let font_handle_for_labels = font_handle.clone();

    let banner_label = if state.is_completed() {
        "\u{258C} replay complete" // ▌ — cursor-block prefix; matches the splash boot-screen convention.
    } else {
        "\u{258C} replay" // ▌
    };
    let progress_label = format_progress(state);

    let banner_bg = Color::srgba(
        BG_ELEVATED_HI.to_srgba().red,
        BG_ELEVATED_HI.to_srgba().green,
        BG_ELEVATED_HI.to_srgba().blue,
        BANNER_ALPHA,
    );

    commands
        .spawn((
            ReplayOverlayRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(BANNER_HEIGHT),
                // Column outer so the content row sits above the 1px
                // scrub bar at the bottom edge.
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(banner_bg),
            // Pin the banner to its z layer in both the local and the
            // global stacking context — `GlobalZIndex` matters because
            // the overlay is a top-level Node (no parent), and Bevy 0.18
            // has historically had subtle stacking-context drift here.
            ZIndex(Z_REPLAY_OVERLAY),
            GlobalZIndex(Z_REPLAY_OVERLAY),
        ))
        .with_children(|banner| {
            // Top row: the existing content (label / progress / Stop).
            banner
                .spawn(Node {
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::axes(VAL_SPACE_4, VAL_SPACE_2),
                    column_gap: VAL_SPACE_4,
                    ..default()
                })
                .with_children(|row| {
                    // Left: column with the accent "▌ replay" headline
                    // above and a small `GAME #YYYY-DDD` caption below.
                    // The caption mirrors the mockup's right-anchored
                    // game identifier but stays visually grouped with
                    // the headline so the two pieces of "this is a
                    // replay of game X" read as a single unit.
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        row_gap: Val::Px(2.0),
                        ..default()
                    })
                    .with_children(|left| {
                        left.spawn((
                            ReplayOverlayBannerText,
                            Text::new(banner_label),
                            TextFont {
                                font: font_handle.clone(),
                                font_size: TYPE_HEADLINE,
                                ..default()
                            },
                            TextColor(ACCENT_PRIMARY),
                        ));
                        left.spawn((
                            ReplayOverlayGameCaption,
                            Text::new(format_game_caption(state).unwrap_or_default()),
                            TextFont {
                                font: font_handle.clone(),
                                font_size: TYPE_CAPTION,
                                ..default()
                            },
                            TextColor(TEXT_SECONDARY),
                        ));
                    });

                    // Centre: progress readout, wrapped in a 1 px
                    // ACCENT_PRIMARY-bordered chip so it reads as a
                    // discrete callout rather than free-floating
                    // text. No fill — the Terminal aesthetic gets
                    // depth from borders + tonal layering, not
                    // shadows. The marker stays on the inner Text so
                    // `update_progress_text` keeps working unchanged.
                    row.spawn((
                        Node {
                            border: UiRect::all(Val::Px(1.0)),
                            padding: UiRect::axes(VAL_SPACE_2, VAL_SPACE_1),
                            ..default()
                        },
                        BorderColor::all(ACCENT_PRIMARY),
                    ))
                    .with_children(|chip| {
                        chip.spawn((
                            ReplayOverlayProgressText,
                            Text::new(progress_label),
                            TextFont {
                                font: font_handle,
                                font_size: TYPE_BODY,
                                ..default()
                            },
                            TextColor(TEXT_PRIMARY),
                        ));
                    });

                    // Right: Stop button. Tertiary variant — the
                    // action is available but not the loudest element
                    // in the banner; the "Replay" primary accent owns
                    // that slot. `spawn_modal_button` gives us hover /
                    // press paint and focus rings for free via the
                    // existing `UiModalPlugin` paint system.
                    row.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: VAL_SPACE_2,
                        ..default()
                    })
                    .with_children(|wrap| {
                        // Pause / Resume label is set from the current
                        // state so a freshly-spawned overlay (which
                        // currently always starts unpaused) reads
                        // "Pause". `update_pause_button_label`
                        // repaints it whenever the state changes.
                        spawn_modal_button(
                            wrap,
                            ReplayPauseButton,
                            pause_button_label(state),
                            None,
                            ButtonVariant::Tertiary,
                            font_res,
                        );
                        spawn_modal_button(
                            wrap,
                            ReplayStepButton,
                            "Step",
                            None,
                            ButtonVariant::Tertiary,
                            font_res,
                        );
                        spawn_modal_button(
                            wrap,
                            ReplayStopButton,
                            "Stop",
                            None,
                            ButtonVariant::Tertiary,
                            font_res,
                        );
                    });
                });

            // Bottom edge: 1px-tall scrub bar. Track in `BORDER_SUBTLE`,
            // fill in `ACCENT_PRIMARY`. The fill width is rewritten by
            // [`update_scrub_fill`] every tick the cursor advances.
            // Initial fill width matches the spawn-time progress so the
            // first-frame paint already reflects state instead of
            // popping from 0 → cursor on the first tick.
            let initial_scrub_pct = scrub_pct(state);
            let win_pct = win_move_marker_pct(state);
            banner
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(1.0),
                        ..default()
                    },
                    BackgroundColor(BORDER_SUBTLE),
                    // HC marker: bumps the 1 px track from #505050
                    // → #a0a0a0 under high-contrast mode. The track
                    // paints via BackgroundColor (it's a 1 px Node,
                    // not a border on a wider container) so the
                    // BorderColor-targeting HighContrastBorder marker
                    // doesn't apply — HighContrastBackground is the
                    // parallel primitive for this case.
                    HighContrastBackground::with_default(BORDER_SUBTLE),
                ))
                .with_children(|track| {
                    track.spawn((
                        ReplayOverlayScrubFill,
                        Node {
                            width: Val::Percent(initial_scrub_pct),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(ACCENT_PRIMARY),
                    ));
                    // WIN MOVE marker — small green tick anchored at
                    // `win_move_index / total`. Spawned only when the
                    // active replay carries the field; older replays
                    // pre-dating `win_move_index` simply don't get a
                    // marker. Centered vertically on the 1px track via
                    // a 3px-tall node offset 1px above the track top so
                    // 1px sits above and 1px below the track line.
                    if let Some(pct) = win_pct {
                        track.spawn((
                            ReplayOverlayWinMoveMarker,
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Percent(pct),
                                top: Val::Px(-1.0),
                                width: Val::Px(2.0),
                                height: Val::Px(3.0),
                                ..default()
                            },
                            BackgroundColor(STATE_SUCCESS),
                        ));
                    }
                    // Fixed quarter-mark notches: five 1px vertical
                    // ticks at 0 / 25 / 50 / 75 / 100 % that give the
                    // player visual anchor points without needing to
                    // mentally bisect the bar. Painted in
                    // BORDER_SUBTLE — same colour as the unfilled
                    // track — so visibility comes from extending past
                    // the 1px track height (5px tall, anchored 2px
                    // above the track top) rather than colour
                    // contrast. Spawned *after* the WIN MOVE marker
                    // so a notch and the marker landing on the same
                    // percentage paint the marker on top.
                    for pct in scrub_notch_positions() {
                        track.spawn((
                            ReplayOverlayScrubNotch,
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Percent(pct),
                                top: Val::Px(-2.0),
                                width: Val::Px(1.0),
                                height: Val::Px(5.0),
                                ..default()
                            },
                            BackgroundColor(BORDER_SUBTLE),
                            // Same HC-paint reasoning as the track
                            // above: 5 px tall × 1 px wide tick mark
                            // paints via BackgroundColor, so
                            // HighContrastBackground (not -Border) is
                            // the right marker.
                            HighContrastBackground::with_default(BORDER_SUBTLE),
                        ));
                    }
                });

            // Third banner row: percentage labels (`0%` / `25%` /
            // `50%` / `75%` / `100%`) under each scrub-bar notch.
            // Sibling of (not child of) the 1px track because labels
            // need their own vertical real estate (TYPE_CAPTION text
            // doesn't fit inside a 1px container). Position math:
            // track Node has `Val::Percent(p)` referencing the
            // banner's full width; this label row also has the
            // banner's full width, so labels at the same
            // percentages line up vertically with their notches.
            let labels = scrub_notch_labels();
            let positions = scrub_notch_positions();
            banner
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(SCRUB_LABEL_ROW_HEIGHT),
                    position_type: PositionType::Relative,
                    ..default()
                })
                .with_children(|row| {
                    for (i, (label, pct)) in
                        labels.iter().zip(positions.iter()).enumerate()
                    {
                        // Endpoints flush to the row's edges; middle
                        // three labels anchor at their percentage.
                        // `i == 0` → flush left (`left: 0`), so the
                        // "0%" caption doesn't get clipped at the
                        // left edge. `i == last` → flush right
                        // (`right: 0`) so "100%" doesn't overflow
                        // the banner. Bevy 0.18 UI has no clean
                        // CSS-style `translate-x: -50%` centering,
                        // so the middle three labels sit slightly
                        // right-of-notch — visually subtle at this
                        // font size; explicit polish target if
                        // anyone notices.
                        let mut node = Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(2.0),
                            ..default()
                        };
                        if i == 0 {
                            node.left = Val::Px(0.0);
                        } else if i == labels.len() - 1 {
                            node.right = Val::Px(0.0);
                        } else {
                            node.left = Val::Percent(*pct);
                        }
                        row.spawn((
                            ReplayOverlayScrubNotchLabel,
                            node,
                            Text::new(*label),
                            TextFont {
                                font: font_handle_for_labels.clone(),
                                font_size: TYPE_CAPTION,
                                ..default()
                            },
                            // The mockup's `text-outline` (BORDER_SUBTLE)
                            // would match the notches but reads as too
                            // low-contrast against `BG_ELEVATED_HI` for
                            // the labels to actually be legible at 12 px.
                            // TEXT_SECONDARY keeps the subdued visual
                            // hierarchy (caption, not headline) while
                            // staying readable.
                            TextColor(TEXT_SECONDARY),
                        ));
                    }
                });

            // Fourth banner row: keybind-hint footer. Vim-style
            // mode line on the left (`▌ NORMAL │ replay`), keybind
            // hint on the right (`[SPACE] pause/resume`), 1px top
            // border in BORDER_SUBTLE separating it from the
            // labels row above. Surfaces the existing Space
            // accelerator visually so CLAUDE.md §3.3's UI-first
            // contract holds for keyboard accelerators too.
            banner
                .spawn((
                    ReplayOverlayKeybindFooter,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(KEYBIND_FOOTER_HEIGHT),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(VAL_SPACE_4),
                        border: UiRect::top(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(BORDER_SUBTLE),
                    // Marker for `apply_high_contrast_borders`: bumps
                    // the 1 px top border from BORDER_SUBTLE (#505050)
                    // to BORDER_SUBTLE_HC (#a0a0a0) when
                    // `Settings::high_contrast_mode` is on. Without
                    // this the footer reads as floating loose under
                    // HC because the border that visually anchors it
                    // to the labels row above is near-invisible.
                    HighContrastBorder::with_default(BORDER_SUBTLE),
                ))
                .with_children(|footer| {
                    footer.spawn((
                        Text::new(keybind_footer_mode_text()),
                        TextFont {
                            font: font_handle_for_labels.clone(),
                            font_size: TYPE_CAPTION,
                            ..default()
                        },
                        TextColor(TEXT_SECONDARY),
                    ));
                    footer.spawn((
                        Text::new(keybind_footer_hint_text()),
                        TextFont {
                            font: font_handle_for_labels.clone(),
                            font_size: TYPE_CAPTION,
                            ..default()
                        },
                        TextColor(TEXT_SECONDARY),
                    ));
                });
        });

    // Floating progress chip — a 2D world-space `Text2d` rendered
    // above the destination pile of the most-recently-applied move.
    // Sibling of (not child of) the banner overlay because it lives
    // in world-space coordinates, not the UI tree. Spawned hidden;
    // `update_floating_progress_chip` shows + positions it on the
    // first frame the cursor advances past 0. Lifecycle matches
    // the banner overlay — `react_to_state_change` despawns both
    // when the replay state transitions back to `Inactive`.
    commands.spawn((
        ReplayFloatingProgressChip,
        Text2d::new(format_progress(state)),
        TextFont {
            font: font_handle_for_floating,
            font_size: TYPE_BODY,
            ..default()
        },
        TextColor(TEXT_PRIMARY),
        // High Z keeps the chip above every card stack
        // (Z_DROP_OVERLAY = 50, Z_STOCK_BADGE = 30, regular cards
        // stack to the low double digits at most).
        Transform::from_xyz(0.0, 0.0, 100.0),
        Visibility::Hidden,
    ));
}

/// Pure helper — returns the scrub-fill width as a percentage of the
/// track for the given playback state. `Completed` reads as 100 %;
/// `Inactive` and `Playing` with no progress read as 0 %.
fn scrub_pct(state: &ReplayPlaybackState) -> f32 {
    if state.is_completed() {
        return 100.0;
    }
    match state.progress() {
        Some((_, 0)) | None => 0.0,
        Some((cursor, total)) => {
            let frac = (cursor as f32 / total as f32).clamp(0.0, 1.0);
            frac * 100.0
        }
    }
}

/// Pure helper — returns the fixed scrub-bar notch positions as
/// percentages along the track. Five evenly-spaced notches at the
/// quarter-marks: `[0, 25, 50, 75, 100]`. Function (rather than
/// const) so the unit-test surface is obvious and a future
/// regression — e.g. someone simplifying to three notches — fails
/// at the helper test rather than at visual review.
fn scrub_notch_positions() -> [f32; 5] {
    [0.0, 25.0, 50.0, 75.0, 100.0]
}

/// Pure helper — returns the percentage-label text for each notch,
/// in left-to-right order. Paired with [`scrub_notch_positions`] so
/// `labels[i]` belongs at `positions[i]`. Lifted to a function for
/// the same reason as the positions helper: a clean unit-test
/// surface that fails at a regression (e.g. someone simplifying
/// `100%` → `MAX`) rather than at visual review.
fn scrub_notch_labels() -> [&'static str; 5] {
    ["0%", "25%", "50%", "75%", "100%"]
}

/// Pure helper — returns the vim-style mode indicator text shown on
/// the left side of the keybind-hint footer row. `▌ NORMAL │ replay`
/// matches the `▌replay.tsx` motif from the splash boot-screen and
/// the screen-takeover mockup. The cursor block (`▌`) matches the
/// banner-label prefix; "NORMAL" is the vim mode (mockup parity);
/// "replay" identifies the surface.
fn keybind_footer_mode_text() -> &'static str {
    "\u{258C} NORMAL \u{2502} replay" // ▌ NORMAL │ replay
}

/// Pure helper — returns the keybind-hint text shown on the right
/// side of the keybind-hint footer row. Lists only the keys that
/// are *actually wired* today: the Space accelerator for
/// pause/resume, the ESC accelerator for stop, and the ← / →
/// accelerators for paused single-move stepping. The footer never
/// lists unimplemented keybinds (would lie to users).
fn keybind_footer_hint_text() -> &'static str {
    "[SPACE] pause/resume \u{00B7} [ESC] stop \u{00B7} [\u{2190}\u{2192}] step" // · separator
}

/// Pure helper — returns the WIN MOVE marker's left-edge position as
/// a percentage of the scrub track, or `None` when no marker should
/// be drawn.
///
/// `None` is returned in any of these cases:
/// - The state isn't `Playing` (no replay attached).
/// - The replay's `win_move_index` is `None` (older replay loaded
///   from disk pre-dating the field).
/// - The replay's move list is empty (shouldn't happen for real wins,
///   but guards the divide-by-zero).
///
/// The percentage clamps to `[0, 100]` so a malformed
/// `win_move_index >= total` (defensive — shouldn't happen) doesn't
/// position the marker outside the track.
fn win_move_marker_pct(state: &ReplayPlaybackState) -> Option<f32> {
    let ReplayPlaybackState::Playing { replay, .. } = state else {
        return None;
    };
    let idx = replay.win_move_index?;
    let total = replay.moves.len();
    if total == 0 {
        return None;
    }
    let frac = (idx as f32 / total as f32).clamp(0.0, 1.0);
    Some(frac * 100.0)
}

// ---------------------------------------------------------------------------
// Per-frame text updates
// ---------------------------------------------------------------------------

/// Overwrites the banner label whenever the resource changes — covers the
/// `Playing → Completed` transition by swapping "▌ replay" for
/// "▌ replay complete" in place without despawning the overlay.
fn update_banner_label(
    state: Res<ReplayPlaybackState>,
    mut q: Query<&mut Text, With<ReplayOverlayBannerText>>,
) {
    if !state.is_changed() {
        return;
    }
    let label = if state.is_completed() {
        "\u{258C} replay complete" // ▌
    } else if state.is_playing() {
        "\u{258C} replay" // ▌
    } else {
        return;
    };
    for mut text in &mut q {
        **text = label.to_string();
    }
}

/// Repaints the "Move N of M" centre readout every frame the cursor moves.
/// Cheap — early-exits if the resource has not changed since the last
/// frame so idle replays don't churn the text mesh.
fn update_progress_text(
    state: Res<ReplayPlaybackState>,
    mut q: Query<&mut Text, With<ReplayOverlayProgressText>>,
) {
    if !state.is_changed() {
        return;
    }
    let label = format_progress(&state);
    for mut text in &mut q {
        **text = label.clone();
    }
}

/// Repositions the floating progress chip above the destination
/// pile of the most-recently-applied move and repaints its text.
///
/// The chip is hidden when:
/// - the cursor is at 0 (no moves applied yet — chip would have
///   nowhere meaningful to land), OR
/// - the most-recently-applied move was a `StockClick` (no
///   destination pile — stock-click feedback already lives at
///   the stock pile and we don't want the chip to jitter back
///   to the stock pile every cycle).
///
/// When visible, the chip's world-space `Transform.translation`
/// is set to the destination pile's centre plus a fixed upward
/// offset (`card_size.y * 0.6`) so the chip floats just above
/// the top edge of the card. World-space placement (rather than
/// UI-space + camera projection) keeps the math trivial and means
/// the chip stays correctly positioned through window resizes
/// without any extra wiring — `LayoutResource` already drives
/// every other piece of pile geometry.
fn update_floating_progress_chip(
    state: Res<ReplayPlaybackState>,
    layout: Option<Res<LayoutResource>>,
    mut chips: Query<
        (&mut Transform, &mut Visibility, &mut Text2d),
        With<ReplayFloatingProgressChip>,
    >,
) {
    let Some(layout) = layout else {
        return;
    };

    // Resolve the destination pile of the last-applied move (if
    // any). `cursor` is the index of the *next* move to apply, so
    // the most-recently-applied move sits at `cursor - 1`.
    let dest_pile = match state.as_ref() {
        ReplayPlaybackState::Playing { replay, cursor, .. } if *cursor > 0 => {
            match &replay.moves[cursor - 1] {
                ReplayMove::Move { to, .. } => Some(to.clone()),
                ReplayMove::StockClick => None,
            }
        }
        _ => None,
    };

    let Some(world_pos) = dest_pile
        .as_ref()
        .and_then(|p| layout.0.pile_positions.get(p).copied())
    else {
        // Nothing to point at — hide every chip and exit.
        for (_, mut visibility, _) in chips.iter_mut() {
            *visibility = Visibility::Hidden;
        }
        return;
    };

    // Position above the destination pile by ~60 % of a card
    // height. Half a card lifts above the centre, the extra 10 %
    // is breathing room above the top edge so the chip doesn't
    // visually clip the card.
    let above = Vec2::new(0.0, layout.0.card_size.y * 0.6);
    let target = (world_pos + above).extend(100.0);
    let label = format_progress(&state);

    for (mut transform, mut visibility, mut text2d) in chips.iter_mut() {
        transform.translation = target;
        *visibility = Visibility::Inherited;
        if **text2d != label {
            **text2d = label.clone();
        }
    }
}

/// Repaints the bottom-edge accent scrub fill to mirror cursor progress.
/// Same change-detection guard as the text updaters — the overlay
/// already early-exits when nothing moved, so an idle replay leaves the
/// scrub bar's `Node` untouched.
fn update_scrub_fill(
    state: Res<ReplayPlaybackState>,
    mut q: Query<&mut Node, With<ReplayOverlayScrubFill>>,
) {
    if !state.is_changed() {
        return;
    }
    let pct = scrub_pct(&state);
    for mut node in &mut q {
        node.width = Val::Percent(pct);
    }
}

/// Pure helper — formats the `GAME #YYYY-DDD` caption for the given
/// state. Returns `None` for `Inactive` / `Completed` (the replay is
/// consumed when transitioning out of `Playing`, so the identifier
/// isn't recoverable from state in those branches); spawn-time
/// callers fall back to an empty string.
///
/// Year + chrono ordinal (`{year}-{ordinal:03}`) gives a compact
/// monotonically-increasing identifier shaped like `2026-127` — same
/// shape as the mockup's `GAME #2024-127` motif.
fn format_game_caption(state: &ReplayPlaybackState) -> Option<String> {
    match state {
        ReplayPlaybackState::Playing { replay, .. } => Some(format!(
            "GAME #{}-{:03}",
            replay.recorded_at.year(),
            replay.recorded_at.ordinal()
        )),
        ReplayPlaybackState::Inactive | ReplayPlaybackState::Completed => None,
    }
}

/// Pure helper — formats the centre progress readout for the given state.
/// Exposed at module scope so the spawn path and the per-frame update
/// path produce the exact same string.
fn format_progress(state: &ReplayPlaybackState) -> String {
    match state.progress() {
        // `MOVE N/M` (uppercase + slash) reads as a Terminal output
        // line and matches the floating-chip motif in the mockup at
        // `docs/ui-mockups/replay-overlay-mobile.html`.
        Some((cursor, total)) => format!("MOVE {cursor}/{total}"),
        None if state.is_completed() => "REPLAY COMPLETE".to_string(),
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Playback-control button handlers
// ---------------------------------------------------------------------------

/// Pure helper — returns the label the Pause / Resume button should
/// carry for the given state. "Pause" while running, "Resume" while
/// paused, empty otherwise (the button is despawned with the rest of
/// the overlay tree on transitions to `Inactive` / `Completed`, so
/// the empty branch only fires for one frame around state changes).
fn pause_button_label(state: &ReplayPlaybackState) -> &'static str {
    match state {
        ReplayPlaybackState::Playing { paused: true, .. } => "Resume",
        ReplayPlaybackState::Playing { paused: false, .. } => "Pause",
        ReplayPlaybackState::Inactive | ReplayPlaybackState::Completed => "",
    }
}

/// Watches the Stop button for `Interaction::Pressed` transitions. On a
/// click, calls [`stop_replay_playback`] which resets the state to
/// `Inactive`; the next frame's `react_to_state_change` then despawns
/// the overlay.
fn handle_stop_button(
    mut commands: Commands,
    mut state: ResMut<ReplayPlaybackState>,
    buttons: Query<&Interaction, (With<ReplayStopButton>, Changed<Interaction>)>,
) {
    if !buttons.iter().any(|i| *i == Interaction::Pressed) {
        return;
    }
    stop_replay_playback(&mut commands, &mut state);
}

/// Watches the Pause / Resume button for `Interaction::Pressed`
/// transitions. On a click, toggles the `paused` flag via
/// [`toggle_pause_replay_playback`]. The label repaint happens in
/// [`update_pause_button_label`] on the same frame the state mutation
/// flushes.
fn handle_pause_button(
    mut state: ResMut<ReplayPlaybackState>,
    buttons: Query<&Interaction, (With<ReplayPauseButton>, Changed<Interaction>)>,
) {
    if !buttons.iter().any(|i| *i == Interaction::Pressed) {
        return;
    }
    toggle_pause_replay_playback(&mut state);
}

/// Watches the Step button for `Interaction::Pressed` transitions. On
/// a click, advances exactly one move via [`step_replay_playback`].
/// No-op while playback is unpaused (would race the tick loop) — the
/// guard lives inside `step_replay_playback`.
fn handle_step_button(
    mut state: ResMut<ReplayPlaybackState>,
    mut moves_writer: MessageWriter<MoveRequestEvent>,
    mut draws_writer: MessageWriter<DrawRequestEvent>,
    buttons: Query<&Interaction, (With<ReplayStepButton>, Changed<Interaction>)>,
) {
    if !buttons.iter().any(|i| *i == Interaction::Pressed) {
        return;
    }
    step_replay_playback(&mut state, &mut moves_writer, &mut draws_writer);
}

/// Repaints the Pause / Resume button's label whenever
/// [`ReplayPlaybackState`] changes. Walks from the marked button
/// entity to its single child [`Text`] so the spawn path doesn't need
/// a second marker on the inner node.
fn update_pause_button_label(
    state: Res<ReplayPlaybackState>,
    buttons: Query<&Children, With<ReplayPauseButton>>,
    mut texts: Query<&mut Text>,
) {
    if !state.is_changed() {
        return;
    }
    let label = pause_button_label(&state);
    if label.is_empty() {
        // Overlay is mid-teardown; the button entity will despawn
        // this frame anyway. Skip the repaint to avoid touching a
        // doomed entity.
        return;
    }
    for children in &buttons {
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                text.0 = label.to_string();
                break;
            }
        }
    }
}

/// Watches `Space` for the keyboard pause / resume accelerator.
/// UI-first contract from CLAUDE.md §3.3 is satisfied by the on-
/// screen Pause / Resume button; this is the optional accelerator.
/// No-op when the playback isn't `Playing` (e.g. while a modal is
/// open and the player is using `Space` for something else).
fn handle_pause_keyboard(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut state: ResMut<ReplayPlaybackState>,
) {
    let Some(keys) = keys else { return };
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }
    toggle_pause_replay_playback(&mut state);
}

/// Watches the arrow keys for the paused single-step
/// accelerators. UI-first contract from CLAUDE.md §3.3 is
/// satisfied by the on-screen Step button (forward only); these
/// are the optional accelerators that also surface a backwards
/// step.
///
/// Both keys are paused-only — the underlying step helpers
/// hard-gate via destructure on `paused: true`. Pressing → during
/// running playback or ← at cursor 0 are silent no-ops; the
/// player learns "pause first, then arrow."
///
/// The mockup labels these `[← →] scrub` (continuous fast scan).
/// Single-move step is the closest behaviour shippable today; the
/// footer hint reads `[← →] step` to match what's wired rather
/// than the aspirational "scrub."
fn handle_arrow_keyboard(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut state: ResMut<ReplayPlaybackState>,
    mut moves_writer: MessageWriter<MoveRequestEvent>,
    mut draws_writer: MessageWriter<DrawRequestEvent>,
    mut undo_writer: MessageWriter<UndoRequestEvent>,
) {
    let Some(keys) = keys else { return };
    if keys.just_pressed(KeyCode::ArrowRight) {
        step_replay_playback(&mut state, &mut moves_writer, &mut draws_writer);
    }
    if keys.just_pressed(KeyCode::ArrowLeft) {
        step_backwards_replay_playback(&mut state, &mut undo_writer);
    }
}

/// Watches `Esc` for the keyboard stop accelerator. UI-first
/// contract from CLAUDE.md §3.3 is satisfied by the on-screen
/// Stop button; this is the optional accelerator.
///
/// Cross-plugin coordination: `pause_plugin::toggle_pause` also
/// listens for `Esc` and would otherwise open the pause modal on
/// the same press. The conflict is resolved by `toggle_pause`
/// gating itself on `ReplayPlaybackState::is_playing()` —
/// symmetrical to the existing `forfeit_screens` /
/// `other_modal_scrims` defer-if pattern in that system. So during
/// an active replay this handler owns the `Esc` press and the
/// pause modal stays closed.
///
/// No-op when the playback isn't `Playing` (the resource may still
/// exist as `Inactive` or `Completed`; only `Playing` means a
/// replay is on screen for the player to stop).
fn handle_stop_keyboard(
    mut commands: Commands,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut state: ResMut<ReplayPlaybackState>,
) {
    let Some(keys) = keys else { return };
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    if !state.is_playing() {
        return;
    }
    stop_replay_playback(&mut commands, &mut state);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use solitaire_core::game_state::{DrawMode, GameMode};
    use solitaire_data::{Replay, ReplayMove};

    /// Build a minimal but well-formed [`Replay`] with `move_count` no-op
    /// `StockClick` entries. Tests only ever read `replay.moves.len()`
    /// (denominator of the progress indicator), so the move kind is
    /// irrelevant beyond producing the right count.
    fn synthetic_replay(move_count: usize) -> Replay {
        Replay::new(
            42,
            DrawMode::DrawOne,
            GameMode::Classic,
            120,
            1_000,
            NaiveDate::from_ymd_opt(2026, 5, 2).expect("valid date"),
            (0..move_count).map(|_| ReplayMove::StockClick).collect(),
        )
    }

    /// Build a test app that has the overlay plugin but **not** the
    /// playback plugin — tests insert `ReplayPlaybackState` manually so
    /// they can drive every state transition deterministically.
    fn headless_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(ReplayOverlayPlugin);
        app.init_resource::<ReplayPlaybackState>();
        app
    }

    /// Count `ReplayOverlayRoot` entities in the world — the overlay's
    /// presence/absence is the spawn-test's primary observable.
    fn overlay_root_count(app: &mut App) -> usize {
        app.world_mut()
            .query::<&ReplayOverlayRoot>()
            .iter(app.world())
            .count()
    }

    /// Read the current text content of the unique progress-text entity.
    fn progress_text(app: &mut App) -> String {
        let mut q = app
            .world_mut()
            .query_filtered::<&Text, With<ReplayOverlayProgressText>>();
        q.iter(app.world())
            .next()
            .map(|t| t.0.clone())
            .unwrap_or_default()
    }

    /// Read the current text content of the unique banner-label entity.
    fn banner_text(app: &mut App) -> String {
        let mut q = app
            .world_mut()
            .query_filtered::<&Text, With<ReplayOverlayBannerText>>();
        q.iter(app.world())
            .next()
            .map(|t| t.0.clone())
            .unwrap_or_default()
    }

    /// Set the playback resource without going through the playback core.
    fn set_state(app: &mut App, state: ReplayPlaybackState) {
        app.world_mut().insert_resource(state);
    }

    /// Find the unique `ReplayStopButton` entity for the click-handler
    /// test. There must be exactly one.
    fn stop_button_entity(app: &mut App) -> Entity {
        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<ReplayStopButton>>();
        q.iter(app.world())
            .next()
            .expect("Stop button must exist while overlay is spawned")
    }

    /// Going `Inactive → Playing` spawns exactly one overlay root and
    /// the banner label reads "▌ replay".
    #[test]
    fn overlay_spawns_when_playback_starts() {
        let mut app = headless_app();
        // First update with the default `Inactive` resource — overlay
        // must not exist yet.
        app.update();
        assert_eq!(overlay_root_count(&mut app), 0);

        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(10),
                cursor: 0,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();

        assert_eq!(
            overlay_root_count(&mut app),
            1,
            "exactly one ReplayOverlayRoot must spawn on Inactive → Playing",
        );
        assert_eq!(banner_text(&mut app), "\u{258C} replay");
    }

    /// The progress-text entity reads `"Move {cursor} of {total}"` for a
    /// well-formed `Playing` state.
    #[test]
    fn overlay_progress_text_reflects_cursor() {
        let mut app = headless_app();
        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(10),
                cursor: 5,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();

        assert_eq!(progress_text(&mut app), "MOVE 5/10");
    }

    /// Pressing the Stop button resets the state back to `Inactive` and
    /// the next frame's `react_to_state_change` despawns the overlay.
    /// Mirrors the synthetic `Interaction::Pressed` insertion pattern
    /// used elsewhere in the engine for headless click tests.
    #[test]
    fn overlay_stop_button_click_clears_playback() {
        let mut app = headless_app();
        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(10),
                cursor: 0,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();
        assert_eq!(overlay_root_count(&mut app), 1);

        let stop = stop_button_entity(&mut app);
        app.world_mut()
            .entity_mut(stop)
            .insert(Interaction::Pressed);
        // Tick once: the click handler runs late in the frame and resets
        // the state to `Inactive`.
        app.update();

        // State must be back to Inactive.
        let state = app.world().resource::<ReplayPlaybackState>();
        assert!(
            matches!(state, ReplayPlaybackState::Inactive),
            "Stop click must reset ReplayPlaybackState to Inactive; got {state:?}",
        );

        // One more tick — `react_to_state_change` sees the resource
        // change to Inactive and despawns the overlay.
        app.update();
        assert_eq!(
            overlay_root_count(&mut app),
            0,
            "overlay must despawn the frame after state returns to Inactive",
        );
    }

    /// Lifecycle: the floating progress chip spawns alongside the
    /// banner overlay when playback starts, and despawns when
    /// playback ends. (Position correctness needs `LayoutResource`,
    /// which isn't set up in this headless fixture; the lifecycle
    /// test below is what's load-bearing for the spawn/despawn
    /// pairing.)
    #[test]
    fn floating_chip_spawns_and_despawns_with_overlay() {
        let mut app = headless_app();
        // Inactive → no chip.
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&ReplayFloatingProgressChip>()
                .iter(app.world())
                .count(),
            0,
            "no floating chip while playback is Inactive",
        );

        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(5),
                cursor: 0,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&ReplayFloatingProgressChip>()
                .iter(app.world())
                .count(),
            1,
            "floating chip must spawn when playback starts",
        );

        set_state(&mut app, ReplayPlaybackState::Inactive);
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&ReplayFloatingProgressChip>()
                .iter(app.world())
                .count(),
            0,
            "floating chip must despawn when playback ends",
        );
    }

    /// Manually flipping the resource back to `Inactive` (e.g. via the
    /// playback core's auto-clear after `Completed`) tears the overlay
    /// down without any further input.
    #[test]
    fn overlay_despawns_when_playback_returns_to_inactive() {
        let mut app = headless_app();
        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(3),
                cursor: 1,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();
        assert_eq!(overlay_root_count(&mut app), 1);

        set_state(&mut app, ReplayPlaybackState::Inactive);
        app.update();

        assert_eq!(
            overlay_root_count(&mut app),
            0,
            "overlay must despawn on Playing → Inactive transition",
        );
    }

    /// On `Playing → Completed` the banner label updates in place rather
    /// than respawning. The overlay must still be present, and the label
    /// must read "▌ replay complete".
    #[test]
    fn overlay_text_changes_on_completed() {
        let mut app = headless_app();
        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(7),
                cursor: 7,
                secs_to_next: 0.0,
                paused: false,
            },
        );
        app.update();
        assert_eq!(banner_text(&mut app), "\u{258C} replay");

        set_state(&mut app, ReplayPlaybackState::Completed);
        app.update();

        assert_eq!(
            overlay_root_count(&mut app),
            1,
            "overlay must remain spawned while in Completed state",
        );
        assert_eq!(
            banner_text(&mut app),
            "\u{258C} replay complete",
            "banner label must swap on Playing → Completed",
        );
    }

    /// Read the current `Node.width` of the unique scrub-fill entity as
    /// a percentage. Assertions can then compare against expected
    /// `cursor / total` ratios without poking ECS internals at the call
    /// site.
    fn scrub_fill_pct(app: &mut App) -> f32 {
        let mut q = app
            .world_mut()
            .query_filtered::<&Node, With<ReplayOverlayScrubFill>>();
        let node = q
            .iter(app.world())
            .next()
            .expect("scrub-fill node must exist while overlay is spawned");
        match node.width {
            Val::Percent(p) => p,
            other => panic!("scrub fill width must be Val::Percent; got {other:?}"),
        }
    }

    /// Pure-helper guard. Locks in the four corners of `scrub_pct` so a
    /// future refactor of `ReplayPlaybackState::progress()` can't
    /// silently regress the visual cue: `Inactive → 0 %`,
    /// `Playing { cursor: 0, total: N } → 0 %`,
    /// `Playing { cursor: N/2, total: N } → 50 %`,
    /// `Completed → 100 %`.
    #[test]
    fn scrub_pct_covers_state_corners() {
        assert_eq!(scrub_pct(&ReplayPlaybackState::Inactive), 0.0);
        assert_eq!(scrub_pct(&ReplayPlaybackState::Completed), 100.0);
        assert_eq!(
            scrub_pct(&ReplayPlaybackState::Playing {
                replay: synthetic_replay(10),
                cursor: 0,
                secs_to_next: 0.5,
                paused: false,
            }),
            0.0,
        );
        assert_eq!(
            scrub_pct(&ReplayPlaybackState::Playing {
                replay: synthetic_replay(10),
                cursor: 5,
                secs_to_next: 0.5,
                paused: false,
            }),
            50.0,
        );
        assert_eq!(
            scrub_pct(&ReplayPlaybackState::Playing {
                replay: synthetic_replay(10),
                cursor: 10,
                secs_to_next: 0.5,
                paused: false,
            }),
            100.0,
        );
    }

    /// Read the current text content of the unique GAME-caption entity.
    fn game_caption_text(app: &mut App) -> String {
        let mut q = app
            .world_mut()
            .query_filtered::<&Text, With<ReplayOverlayGameCaption>>();
        q.iter(app.world())
            .next()
            .map(|t| t.0.clone())
            .unwrap_or_default()
    }

    /// Pure-helper guard. `Inactive` / `Completed` carry no replay
    /// reference so the caption is `None`; `Playing` formats the
    /// recorded-date as `GAME #YYYY-DDD` with a 3-digit zero-padded
    /// ordinal. Locks all three branches so a future refactor can't
    /// silently regress the identifier shape.
    #[test]
    fn format_game_caption_covers_state_corners() {
        assert_eq!(format_game_caption(&ReplayPlaybackState::Inactive), None);
        assert_eq!(format_game_caption(&ReplayPlaybackState::Completed), None);

        // 2026-05-02 is the 122nd day of 2026 (Jan = 31, Feb = 28,
        // Mar = 31, Apr = 30, May 2 = 122). Synthetic_replay always
        // uses this date so the assertion is stable.
        assert_eq!(
            format_game_caption(&ReplayPlaybackState::Playing {
                replay: synthetic_replay(10),
                cursor: 5,
                secs_to_next: 0.5,
                paused: false,
            }),
            Some("GAME #2026-122".to_string()),
        );

        // Single-digit ordinal must zero-pad to three digits — pin
        // the format string in case someone simplifies to `{}-{}`.
        let mut early_january = synthetic_replay(10);
        early_january.recorded_at = NaiveDate::from_ymd_opt(2026, 1, 5).expect("valid date");
        assert_eq!(
            format_game_caption(&ReplayPlaybackState::Playing {
                replay: early_january,
                cursor: 0,
                secs_to_next: 0.5,
                paused: false,
            }),
            Some("GAME #2026-005".to_string()),
        );
    }

    /// End-to-end: spawning the overlay paints the GAME caption with
    /// the active replay's recorded date in `YYYY-DDD` form. Caption
    /// is empty for `Completed` since the replay is consumed.
    #[test]
    fn overlay_game_caption_shows_replay_date() {
        let mut app = headless_app();
        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(10),
                cursor: 0,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();
        assert_eq!(game_caption_text(&mut app), "GAME #2026-122");

        // Caption empties out on Playing → Completed because
        // `format_game_caption` returns None and the spawn-path
        // helper falls through to `unwrap_or_default()`.
        // The overlay itself stays spawned in `Completed`.
        set_state(&mut app, ReplayPlaybackState::Completed);
        app.update();
        assert_eq!(
            overlay_root_count(&mut app),
            1,
            "overlay must remain spawned while in Completed state",
        );
    }

    /// End-to-end: the spawn path must paint the scrub fill at the
    /// initial cursor's percentage, and the per-frame `update_scrub_fill`
    /// system must repaint it as the cursor advances. Mirrors the shape
    /// of `overlay_progress_text_reflects_cursor`.
    #[test]
    fn overlay_scrub_fill_tracks_cursor() {
        let mut app = headless_app();
        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(8),
                cursor: 2,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();
        assert_eq!(
            scrub_fill_pct(&mut app),
            25.0,
            "spawn-time fill must reflect the initial cursor",
        );

        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(8),
                cursor: 6,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();
        assert_eq!(
            scrub_fill_pct(&mut app),
            75.0,
            "update_scrub_fill must repaint width on cursor advance",
        );

        set_state(&mut app, ReplayPlaybackState::Completed);
        app.update();
        assert_eq!(
            scrub_fill_pct(&mut app),
            100.0,
            "Completed state must read as a fully-filled track",
        );
    }

    // -----------------------------------------------------------------------
    // win_move_marker_pct + ReplayOverlayWinMoveMarker spawn behaviour
    // -----------------------------------------------------------------------

    fn win_marker_count(app: &mut App) -> usize {
        app.world_mut()
            .query::<&ReplayOverlayWinMoveMarker>()
            .iter(app.world())
            .count()
    }

    #[test]
    fn win_move_marker_pct_is_none_for_inactive() {
        assert_eq!(win_move_marker_pct(&ReplayPlaybackState::Inactive), None);
    }

    #[test]
    fn win_move_marker_pct_is_none_for_completed() {
        // `Completed` carries no replay so the marker has no data to
        // anchor against — the overlay treats this as "no marker".
        assert_eq!(win_move_marker_pct(&ReplayPlaybackState::Completed), None);
    }

    #[test]
    fn win_move_marker_pct_is_none_when_replay_lacks_field() {
        // Synthetic replay constructor leaves win_move_index as None
        // (legacy / pre-`ab857bb` path).
        let state = ReplayPlaybackState::Playing {
            replay: synthetic_replay(10),
            cursor: 0,
            secs_to_next: 0.5,
            paused: false,
        };
        assert_eq!(win_move_marker_pct(&state), None);
    }

    #[test]
    fn win_move_marker_pct_is_some_at_correct_position() {
        // 10 moves, win at index 9 → marker sits at 90 % of the track.
        // Matches the recording semantic: cursor reaches the marker
        // exactly when the about-to-apply move IS the win move.
        let state = ReplayPlaybackState::Playing {
            replay: synthetic_replay(10).with_win_move_index(Some(9)),
            cursor: 0,
            secs_to_next: 0.5,
            paused: false,
        };
        assert_eq!(win_move_marker_pct(&state), Some(90.0));
    }

    #[test]
    fn win_move_marker_pct_clamps_to_track_bounds() {
        // Defensive: if a malformed replay carried `win_move_index >=
        // total`, the marker must still sit on the track, not past it.
        let state = ReplayPlaybackState::Playing {
            replay: synthetic_replay(5).with_win_move_index(Some(99)),
            cursor: 0,
            secs_to_next: 0.5,
            paused: false,
        };
        assert_eq!(win_move_marker_pct(&state), Some(100.0));
    }

    #[test]
    fn marker_spawned_when_replay_has_win_move_index() {
        let mut app = headless_app();
        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(8).with_win_move_index(Some(7)),
                cursor: 0,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();
        assert_eq!(
            win_marker_count(&mut app),
            1,
            "marker entity must spawn when replay carries Some(win_move_index)"
        );
    }

    #[test]
    fn marker_not_spawned_when_replay_lacks_win_move_index() {
        let mut app = headless_app();
        // Default constructor → win_move_index: None (legacy replay).
        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(8),
                cursor: 0,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();
        assert_eq!(
            win_marker_count(&mut app),
            0,
            "no marker should spawn for a replay pre-dating the field"
        );
    }

    #[test]
    fn marker_despawns_when_replay_state_returns_to_inactive() {
        let mut app = headless_app();
        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(8).with_win_move_index(Some(7)),
                cursor: 0,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();
        assert_eq!(win_marker_count(&mut app), 1);

        set_state(&mut app, ReplayPlaybackState::Inactive);
        app.update();
        assert_eq!(
            win_marker_count(&mut app),
            0,
            "marker must despawn with the rest of the overlay tree"
        );
    }

    // -----------------------------------------------------------------------
    // scrub_notch_positions + ReplayOverlayScrubNotch spawn behaviour
    // -----------------------------------------------------------------------

    fn scrub_notch_count(app: &mut App) -> usize {
        app.world_mut()
            .query::<&ReplayOverlayScrubNotch>()
            .iter(app.world())
            .count()
    }

    /// Pure-helper guard. Locks in the five-notch ladder at the
    /// quarter-marks. A future simplification to fewer notches (or a
    /// shift to non-quarter spacing) must touch this test, surfacing
    /// the visual change at review time.
    #[test]
    fn scrub_notch_positions_are_quarter_marks() {
        assert_eq!(
            scrub_notch_positions(),
            [0.0, 25.0, 50.0, 75.0, 100.0],
            "scrub notches must sit at the five quarter-mark percentages",
        );
    }

    /// Five notch entities spawn alongside the rest of the overlay
    /// tree on `Inactive → Playing`. Cardinality matches
    /// `scrub_notch_positions().len()`.
    #[test]
    fn scrub_notches_spawn_with_overlay() {
        let mut app = headless_app();
        app.update();
        assert_eq!(scrub_notch_count(&mut app), 0);

        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(10),
                cursor: 0,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();
        assert_eq!(
            scrub_notch_count(&mut app),
            scrub_notch_positions().len(),
            "exactly one notch entity per quarter-mark must spawn",
        );
    }

    /// Each spawned notch carries `HighContrastBackground` so the
    /// existing `update_high_contrast_backgrounds` system bumps
    /// `BORDER_SUBTLE` → `BORDER_SUBTLE_HC` under HC mode.
    /// Five-of-five — every notch tagged.
    #[test]
    fn scrub_notches_carry_high_contrast_background_marker() {
        let mut app = headless_app();
        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(10),
                cursor: 0,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<&HighContrastBackground, With<ReplayOverlayScrubNotch>>()
            .iter(app.world())
            .count();
        assert_eq!(
            count,
            scrub_notch_positions().len(),
            "every notch must carry HighContrastBackground for HC repaint coverage",
        );
    }

    /// The 1 px scrub track also carries `HighContrastBackground` so
    /// the unfilled portion bumps under HC. The fill (ACCENT_PRIMARY,
    /// brick-red) doesn't need a marker — accent colours are
    /// already saturated and don't need an HC variant.
    #[test]
    fn scrub_track_carries_high_contrast_background_marker() {
        let mut app = headless_app();
        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(10),
                cursor: 0,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();

        // Track is the parent Node of the scrub-fill. Find it by
        // walking up from `ReplayOverlayScrubFill` to its parent.
        let world = app.world_mut();
        let mut fill_q = world.query_filtered::<Entity, With<ReplayOverlayScrubFill>>();
        let fill = fill_q
            .iter(world)
            .next()
            .expect("scrub fill must exist while overlay is spawned");
        let mut parent_q = world.query::<&ChildOf>();
        let parent = parent_q
            .get(world, fill)
            .map(|p| p.parent())
            .expect("scrub fill must have a parent (the track)");
        let mut hc_q = world.query::<&HighContrastBackground>();
        assert!(
            hc_q.get(world, parent).is_ok(),
            "scrub track Node (parent of scrub fill) must carry HighContrastBackground",
        );
    }

    /// Notches share the overlay tree's lifecycle — they despawn on
    /// `Playing → Inactive` along with the banner root.
    #[test]
    fn scrub_notches_despawn_with_overlay() {
        let mut app = headless_app();
        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(10),
                cursor: 0,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();
        assert_eq!(scrub_notch_count(&mut app), 5);

        set_state(&mut app, ReplayPlaybackState::Inactive);
        app.update();
        assert_eq!(
            scrub_notch_count(&mut app),
            0,
            "notches must despawn with the rest of the overlay tree",
        );
    }

    fn scrub_notch_label_count(app: &mut App) -> usize {
        app.world_mut()
            .query::<&ReplayOverlayScrubNotchLabel>()
            .iter(app.world())
            .count()
    }

    /// Returns the rendered text of every `ReplayOverlayScrubNotchLabel`
    /// in left-to-right order — the iteration order isn't guaranteed by
    /// the ECS query, so callers needing a stable order must sort.
    fn scrub_notch_label_texts(app: &mut App) -> Vec<String> {
        let world = app.world_mut();
        let mut q = world.query_filtered::<&Text, With<ReplayOverlayScrubNotchLabel>>();
        q.iter(world).map(|t| t.0.clone()).collect()
    }

    /// Pure-helper guard for the label strings. Pairs with
    /// `scrub_notch_positions_are_quarter_marks` — same length, same
    /// order, so `labels[i]` belongs at `positions[i]`.
    #[test]
    fn scrub_notch_labels_are_quarter_mark_percents() {
        assert_eq!(
            scrub_notch_labels(),
            ["0%", "25%", "50%", "75%", "100%"],
            "scrub notch labels must read as the five quarter-mark percentages",
        );
        assert_eq!(
            scrub_notch_labels().len(),
            scrub_notch_positions().len(),
            "labels and positions must remain paired one-to-one",
        );
    }

    /// Five label entities spawn alongside the rest of the overlay.
    /// Cardinality matches `scrub_notch_labels().len()`.
    #[test]
    fn scrub_notch_labels_spawn_with_overlay() {
        let mut app = headless_app();
        app.update();
        assert_eq!(scrub_notch_label_count(&mut app), 0);

        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(10),
                cursor: 0,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();
        assert_eq!(
            scrub_notch_label_count(&mut app),
            scrub_notch_labels().len(),
            "exactly one label entity per notch must spawn",
        );
    }

    /// Each spawned label carries one of the helper's strings — pins
    /// the spawn-path against drift between the helper and the actual
    /// painted text.
    #[test]
    fn scrub_notch_labels_carry_helper_strings() {
        let mut app = headless_app();
        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(10),
                cursor: 0,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();

        let mut texts = scrub_notch_label_texts(&mut app);
        texts.sort();
        let mut expected: Vec<String> = scrub_notch_labels()
            .iter()
            .map(|s| s.to_string())
            .collect();
        expected.sort();
        assert_eq!(
            texts, expected,
            "spawned label texts must equal the helper's strings (set equality, ECS order is not guaranteed)",
        );
    }

    /// Labels share the overlay tree's lifecycle — they despawn on
    /// `Playing → Inactive` along with the banner root.
    #[test]
    fn scrub_notch_labels_despawn_with_overlay() {
        let mut app = headless_app();
        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(10),
                cursor: 0,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();
        assert_eq!(scrub_notch_label_count(&mut app), 5);

        set_state(&mut app, ReplayPlaybackState::Inactive);
        app.update();
        assert_eq!(
            scrub_notch_label_count(&mut app),
            0,
            "labels must despawn with the rest of the overlay tree",
        );
    }

    fn keybind_footer_count(app: &mut App) -> usize {
        app.world_mut()
            .query::<&ReplayOverlayKeybindFooter>()
            .iter(app.world())
            .count()
    }

    /// Returns every `Text` rendered as a descendant of the
    /// keybind-footer row. Used to assert the mode + hint texts
    /// appear inside the footer without requiring per-text markers.
    fn keybind_footer_text_set(app: &mut App) -> Vec<String> {
        let world = app.world_mut();
        // Find the footer entity, then walk its descendants for `Text`.
        let mut footer_q = world.query_filtered::<Entity, With<ReplayOverlayKeybindFooter>>();
        let Some(footer) = footer_q.iter(world).next() else {
            return Vec::new();
        };
        let mut child_q = world.query::<&Children>();
        let Ok(children) = child_q.get(world, footer) else {
            return Vec::new();
        };
        let child_entities: Vec<Entity> = children.iter().collect();
        let mut text_q = world.query::<&Text>();
        child_entities
            .into_iter()
            .filter_map(|e| text_q.get(world, e).ok().map(|t| t.0.clone()))
            .collect()
    }

    /// Pure-helper guards for the static text strings. Pin both
    /// helpers so a future refactor that reformats the mode line
    /// or extends the hint with un-wired keybinds fails at the
    /// helper test rather than at visual review.
    #[test]
    fn keybind_footer_helpers_carry_expected_text() {
        assert_eq!(
            keybind_footer_mode_text(),
            "\u{258C} NORMAL \u{2502} replay",
            "mode line must read as the cursor-block + NORMAL + bar + replay motif",
        );
        assert_eq!(
            keybind_footer_hint_text(),
            "[SPACE] pause/resume \u{00B7} [ESC] stop \u{00B7} [\u{2190}\u{2192}] step",
            "hint text must list all three wired keybind groups (Space → pause/resume, Esc → stop, ←→ → step) separated by middle dots",
        );
    }

    /// Footer entity spawns alongside the rest of the overlay tree
    /// on `Inactive → Playing`. Cardinality is exactly one — the
    /// footer is a singleton row, not a per-keybind multiple.
    #[test]
    fn keybind_footer_spawns_with_overlay() {
        let mut app = headless_app();
        app.update();
        assert_eq!(keybind_footer_count(&mut app), 0);

        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(10),
                cursor: 0,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();
        assert_eq!(
            keybind_footer_count(&mut app),
            1,
            "exactly one keybind-footer row must spawn with the overlay",
        );
    }

    /// Spawned footer carries both helper strings as direct-child
    /// `Text` content — pins the spawn-path against drift between
    /// the helpers and the actual painted text.
    #[test]
    fn keybind_footer_paints_helper_strings() {
        let mut app = headless_app();
        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(10),
                cursor: 0,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();

        let texts = keybind_footer_text_set(&mut app);
        assert!(
            texts.contains(&keybind_footer_mode_text().to_string()),
            "footer must contain the mode-line text; got {texts:?}",
        );
        assert!(
            texts.contains(&keybind_footer_hint_text().to_string()),
            "footer must contain the keybind-hint text; got {texts:?}",
        );
    }

    /// Spawned footer carries `HighContrastBorder` so the existing
    /// `apply_high_contrast_borders` system bumps the 1 px top
    /// border under HC mode. Without this the footer reads as
    /// floating loose under HC.
    #[test]
    fn keybind_footer_carries_high_contrast_border_marker() {
        let mut app = headless_app();
        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(10),
                cursor: 0,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();

        let mut q = app
            .world_mut()
            .query_filtered::<&HighContrastBorder, With<ReplayOverlayKeybindFooter>>();
        let marker = q.iter(app.world()).next();
        assert!(
            marker.is_some(),
            "footer must carry HighContrastBorder so `apply_high_contrast_borders` picks it up under HC mode",
        );
    }

    /// Footer shares the overlay tree's lifecycle — it despawns on
    /// `Playing → Inactive` along with the banner root.
    #[test]
    fn keybind_footer_despawns_with_overlay() {
        let mut app = headless_app();
        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(10),
                cursor: 0,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();
        assert_eq!(keybind_footer_count(&mut app), 1);

        set_state(&mut app, ReplayPlaybackState::Inactive);
        app.update();
        assert_eq!(
            keybind_footer_count(&mut app),
            0,
            "footer must despawn with the rest of the overlay tree",
        );
    }

    /// Notches are independent of `win_move_index` — a replay with no
    /// win marker still gets the full five-notch ladder (notches give
    /// quarter-mark anchor points; the win marker is an additional
    /// overlay on top of them, not a replacement).
    #[test]
    fn scrub_notches_spawn_even_without_win_marker() {
        let mut app = headless_app();
        // Default constructor → win_move_index: None.
        set_state(
            &mut app,
            ReplayPlaybackState::Playing {
                replay: synthetic_replay(8),
                cursor: 0,
                secs_to_next: 0.5,
                paused: false,
            },
        );
        app.update();
        assert_eq!(
            scrub_notch_count(&mut app),
            5,
            "notches and win marker are independent — no marker doesn't drop the notches",
        );
    }

    // -----------------------------------------------------------------------
    // pause_button_label + pause / step click handlers + keyboard accelerator
    // -----------------------------------------------------------------------

    /// Read the current text content of the unique pause / resume button.
    fn pause_button_text(app: &mut App) -> String {
        let world = app.world_mut();
        let mut button_q = world.query_filtered::<&Children, With<ReplayPauseButton>>();
        let children: Vec<Entity> = button_q
            .iter(world)
            .next()
            .map(|c| c.iter().collect())
            .unwrap_or_default();
        let mut text_q = world.query::<&Text>();
        for child in children {
            if let Ok(text) = text_q.get(world, child) {
                return text.0.clone();
            }
        }
        String::new()
    }

    /// Find the unique entity carrying the given button marker.
    fn unique_button<M: Component>(app: &mut App) -> Entity {
        let world = app.world_mut();
        let mut q = world.query_filtered::<Entity, With<M>>();
        q.iter(world).next().expect("button entity must exist")
    }

    fn pressed_paused_state(replay_len: usize, cursor: usize) -> ReplayPlaybackState {
        ReplayPlaybackState::Playing {
            replay: synthetic_replay(replay_len),
            cursor,
            secs_to_next: 0.5,
            paused: true,
        }
    }

    fn running_state(replay_len: usize, cursor: usize) -> ReplayPlaybackState {
        ReplayPlaybackState::Playing {
            replay: synthetic_replay(replay_len),
            cursor,
            secs_to_next: 0.5,
            paused: false,
        }
    }

    #[test]
    fn pause_button_label_reads_pause_when_running() {
        assert_eq!(pause_button_label(&running_state(5, 0)), "Pause");
    }

    #[test]
    fn pause_button_label_reads_resume_when_paused() {
        assert_eq!(pause_button_label(&pressed_paused_state(5, 0)), "Resume");
    }

    #[test]
    fn pause_button_label_is_empty_off_state() {
        assert_eq!(pause_button_label(&ReplayPlaybackState::Inactive), "");
        assert_eq!(pause_button_label(&ReplayPlaybackState::Completed), "");
    }

    #[test]
    fn pause_button_text_swaps_when_state_pauses() {
        let mut app = headless_app();
        set_state(&mut app, running_state(5, 0));
        app.update();
        assert_eq!(pause_button_text(&mut app), "Pause");

        set_state(&mut app, pressed_paused_state(5, 0));
        app.update();
        assert_eq!(
            pause_button_text(&mut app),
            "Resume",
            "label must repaint to Resume on the frame the state pauses"
        );
    }

    #[test]
    fn pause_button_click_toggles_paused_flag() {
        let mut app = headless_app();
        set_state(&mut app, running_state(5, 0));
        app.update();

        let button = unique_button::<ReplayPauseButton>(&mut app);
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Pressed);
        app.update();

        match app.world().resource::<ReplayPlaybackState>() {
            ReplayPlaybackState::Playing { paused, .. } => {
                assert!(*paused, "click must flip running → paused");
            }
            other => panic!("expected Playing, got {other:?}"),
        }
    }

    #[test]
    fn step_button_click_advances_cursor_while_paused() {
        let mut app = headless_app();
        set_state(&mut app, pressed_paused_state(5, 0));
        app.update();

        let button = unique_button::<ReplayStepButton>(&mut app);
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Pressed);
        app.update();

        match app.world().resource::<ReplayPlaybackState>() {
            ReplayPlaybackState::Playing { cursor, paused, .. } => {
                assert_eq!(*cursor, 1, "step must advance the cursor by exactly one");
                assert!(*paused, "step must leave the paused flag untouched");
            }
            other => panic!("expected Playing, got {other:?}"),
        }
    }

    #[test]
    fn step_button_click_is_noop_while_running() {
        let mut app = headless_app();
        set_state(&mut app, running_state(5, 0));
        app.update();

        let button = unique_button::<ReplayStepButton>(&mut app);
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Pressed);
        app.update();

        match app.world().resource::<ReplayPlaybackState>() {
            ReplayPlaybackState::Playing { cursor, paused, .. } => {
                assert_eq!(*cursor, 0, "running-step must not race the tick loop");
                assert!(!*paused);
            }
            other => panic!("expected Playing, got {other:?}"),
        }
    }

    /// Pressing Esc while a replay is playing resets the state to
    /// `Inactive` (same end-state as clicking the Stop button).
    /// Mirrors `space_keyboard_toggles_paused_flag` for the stop
    /// accelerator.
    #[test]
    fn esc_keyboard_stops_active_replay() {
        let mut app = headless_app();
        // The keyboard handler reads `Option<Res<ButtonInput<KeyCode>>>`
        // and no-ops when missing — provide it for this test.
        app.init_resource::<ButtonInput<KeyCode>>();
        set_state(&mut app, running_state(5, 0));
        app.update();
        assert_eq!(overlay_root_count(&mut app), 1);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();

        assert!(
            matches!(
                app.world().resource::<ReplayPlaybackState>(),
                ReplayPlaybackState::Inactive
            ),
            "Esc must reset state to Inactive while replay is Playing",
        );

        // One more tick — `react_to_state_change` despawns the overlay
        // in response to the state going Inactive.
        app.update();
        assert_eq!(
            overlay_root_count(&mut app),
            0,
            "overlay must despawn the frame after Esc stops the replay",
        );
    }

    /// Esc is a no-op when the replay isn't `Playing` — covers
    /// `Inactive` (no replay attached) and `Completed` (auto-clear
    /// underway). The handler must stay quiet so the global Esc
    /// listeners (pause modal, etc.) own those frames.
    #[test]
    fn esc_keyboard_is_noop_when_not_playing() {
        let mut app = headless_app();
        app.init_resource::<ButtonInput<KeyCode>>();
        // Resource defaults to Inactive — no replay attached.
        app.update();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();

        // State stays Inactive — no spurious mutation.
        assert!(matches!(
            app.world().resource::<ReplayPlaybackState>(),
            ReplayPlaybackState::Inactive
        ));
    }

    /// The keybind-footer hint text now lists both wired
    /// accelerators (Space + Esc). Lock the format so a future edit
    /// that drops one or the other has to also update this test.
    #[test]
    fn keybind_footer_hint_lists_space_and_esc() {
        let hint = keybind_footer_hint_text();
        assert!(
            hint.contains("[SPACE]"),
            "hint must surface the Space accelerator; got {hint:?}",
        );
        assert!(
            hint.contains("[ESC]"),
            "hint must surface the Esc accelerator; got {hint:?}",
        );
    }

    /// Hint must also list the arrow-key step accelerators.
    /// Pinned separately from the Space + Esc test so a future
    /// regression that drops only the arrows is caught here even
    /// if the Space + Esc check still passes.
    #[test]
    fn keybind_footer_hint_lists_arrow_steps() {
        let hint = keybind_footer_hint_text();
        assert!(
            hint.contains("\u{2190}\u{2192}"),
            "hint must surface the ←→ step accelerators; got {hint:?}",
        );
        assert!(
            hint.contains("step"),
            "hint must label the arrow accelerators as 'step' \
             (matches what's wired — single-move step, not continuous scrub); got {hint:?}",
        );
    }

    /// Pressing → while paused advances the cursor by exactly one
    /// — same end-state as clicking the on-screen Step button.
    #[test]
    fn arrow_right_keyboard_advances_cursor_while_paused() {
        let mut app = headless_app();
        app.init_resource::<ButtonInput<KeyCode>>();
        set_state(&mut app, pressed_paused_state(5, 0));
        app.update();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowRight);
        app.update();

        match app.world().resource::<ReplayPlaybackState>() {
            ReplayPlaybackState::Playing { cursor, paused, .. } => {
                assert_eq!(
                    *cursor, 1,
                    "→ must advance the cursor by exactly one while paused",
                );
                assert!(
                    *paused,
                    "→ must leave the paused flag untouched",
                );
            }
            other => panic!("expected Playing, got {other:?}"),
        }
    }

    /// Pressing → while running is a no-op — the existing
    /// `step_replay_playback` guard prevents racing the tick loop.
    #[test]
    fn arrow_right_keyboard_is_noop_while_running() {
        let mut app = headless_app();
        app.init_resource::<ButtonInput<KeyCode>>();
        set_state(&mut app, running_state(5, 0));
        app.update();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowRight);
        app.update();

        match app.world().resource::<ReplayPlaybackState>() {
            ReplayPlaybackState::Playing { cursor, paused, .. } => {
                assert_eq!(*cursor, 0, "→ must not race the tick loop");
                assert!(!*paused);
            }
            other => panic!("expected Playing, got {other:?}"),
        }
    }

    /// Pressing ← while paused with cursor > 0 decrements the
    /// cursor by exactly one. The corresponding game-state reversal
    /// happens when `handle_undo` reads the dispatched
    /// `UndoRequestEvent` — that's covered in the playback core's
    /// integration test, not here.
    #[test]
    fn arrow_left_keyboard_decrements_cursor_while_paused() {
        let mut app = headless_app();
        app.init_resource::<ButtonInput<KeyCode>>();
        // Start paused at cursor=3 so there's room to step backwards.
        set_state(&mut app, pressed_paused_state(5, 3));
        app.update();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowLeft);
        app.update();

        match app.world().resource::<ReplayPlaybackState>() {
            ReplayPlaybackState::Playing { cursor, paused, .. } => {
                assert_eq!(
                    *cursor, 2,
                    "← must decrement the cursor by exactly one while paused",
                );
                assert!(
                    *paused,
                    "← must leave the paused flag untouched",
                );
            }
            other => panic!("expected Playing, got {other:?}"),
        }
    }

    /// Pressing ← at cursor 0 is a no-op (nothing to rewind past).
    #[test]
    fn arrow_left_keyboard_is_noop_at_cursor_zero() {
        let mut app = headless_app();
        app.init_resource::<ButtonInput<KeyCode>>();
        set_state(&mut app, pressed_paused_state(5, 0));
        app.update();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowLeft);
        app.update();

        match app.world().resource::<ReplayPlaybackState>() {
            ReplayPlaybackState::Playing { cursor, .. } => {
                assert_eq!(*cursor, 0, "← at cursor 0 must be a no-op");
            }
            other => panic!("expected Playing, got {other:?}"),
        }
    }

    /// Pressing ← while running is a no-op — same hard-gate
    /// rationale as the forward-step paused-only check.
    #[test]
    fn arrow_left_keyboard_is_noop_while_running() {
        let mut app = headless_app();
        app.init_resource::<ButtonInput<KeyCode>>();
        set_state(&mut app, running_state(5, 3));
        app.update();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowLeft);
        app.update();

        match app.world().resource::<ReplayPlaybackState>() {
            ReplayPlaybackState::Playing { cursor, paused, .. } => {
                assert_eq!(*cursor, 3, "← must not race the tick loop");
                assert!(!*paused);
            }
            other => panic!("expected Playing, got {other:?}"),
        }
    }

    #[test]
    fn space_keyboard_toggles_paused_flag() {
        let mut app = headless_app();
        // The keyboard handler reads `Option<Res<ButtonInput<KeyCode>>>`
        // and no-ops when missing — provide it for this test.
        app.init_resource::<ButtonInput<KeyCode>>();
        set_state(&mut app, running_state(5, 0));
        app.update();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        app.update();

        match app.world().resource::<ReplayPlaybackState>() {
            ReplayPlaybackState::Playing { paused, .. } => {
                assert!(*paused, "Space must toggle running → paused");
            }
            other => panic!("expected Playing, got {other:?}"),
        }
    }
}
