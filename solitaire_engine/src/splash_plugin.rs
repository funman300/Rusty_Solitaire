//! Launch splash overlay.
//!
//! On app start the engine spawns a fullscreen, high-Z overlay that
//! reads the Terminal-style "boot screen" — a cyan cursor block, the
//! "Solitaire Quest" wordmark, a short fixture boot log, a progress
//! bar, and a footer with the design-system palette swatches and the
//! build version. The overlay fades in over 300 ms, holds for ~1 s,
//! then fades out for 300 ms before despawning. The deal animation
//! plays *behind* the splash during the hold, so the player sees the
//! dealt board appear as the splash dissolves.
//!
//! ## Why an overlay instead of an `AppState`
//!
//! Every existing plugin in this engine runs unconditionally on
//! `Startup`/`Update`; gating them with `run_if(in_state(...))` would
//! be a sweeping refactor for a one-off brand beat. The splash
//! instead sits on top of `Z_SPLASH` (above tooltips, focus ring,
//! and toasts) while the rest of the game runs normally beneath it.
//! The handoff is intentional: the user finishes the splash and the
//! dealt board is already there.
//!
//! ## Dismissal
//!
//! Any keypress, mouse click, or touch begin shortcuts the splash to
//! its fade-out window — never to an instant despawn, so the dissolve
//! still plays for visual continuity. The dismiss input is **not**
//! consumed, so a player who instinctively taps Space to "skip the
//! intro" still gets their stock draw the moment the splash clears
//! (Space and most other gameplay keys read `just_pressed`, which by
//! the next tick is already false — splash dismissal happens on the
//! same tick as the press, so downstream gameplay handlers see
//! exactly the keystroke they would have seen with no splash).
//!
//! ## Fade scaffold
//!
//! Every visible element on the splash carries a [`SplashFadable`]
//! (text colour) or [`SplashFadableBg`] (background colour) marker
//! that records its full-alpha base colour. [`advance_splash`] reads
//! `SplashAge` once per frame, computes the current alpha, and writes
//! `base_color` × current-alpha into every fadable. Replaces the
//! prior per-marker queries (`SplashTitle` / `SplashSubtitle` /
//! `SplashCursor`) which didn't scale past three children — the
//! Terminal splash has ~15 fadable elements (cursor, title, divider,
//! subtitle, four boot-log rows, progress-bar track + fill,
//! progress-bar caption, palette label, eight palette swatches,
//! version line).
//!
//! ## Headless tests
//!
//! Under `MinimalPlugins + SplashPlugin`, the `Time<Virtual>` clock
//! clamps each tick to `max_delta` (default 250 ms) regardless of
//! the `TimeUpdateStrategy::ManualDuration` value, so tests advance
//! time in 200 ms ticks and call `app.update()` enough times to
//! cross the desired threshold (same approach used by
//! `ui_tooltip::tests`).

use std::time::Duration;

use bevy::input::touch::Touches;
use bevy::prelude::*;

use crate::font_plugin::FontResource;
use crate::settings_plugin::SettingsResource;
use crate::ui_theme::{
    ACCENT_PRIMARY, ACCENT_SECONDARY, BG_BASE, BORDER_SUBTLE, MOTION_SPLASH_FADE_SECS,
    MOTION_SPLASH_TOTAL_SECS, STATE_DANGER, STATE_INFO, STATE_SUCCESS, STATE_WARNING,
    TEXT_DISABLED, TEXT_PRIMARY, TYPE_CAPTION, TYPE_DISPLAY, VAL_SPACE_1, VAL_SPACE_2,
    VAL_SPACE_3, VAL_SPACE_5, VAL_SPACE_6, VAL_SPACE_7, Z_SPLASH,
};

// ---------------------------------------------------------------------------
// Public plugin
// ---------------------------------------------------------------------------

/// Drives the launch splash overlay. Add this plugin once at app
/// start; the splash spawns during `Startup`, fades in/out over
/// [`MOTION_SPLASH_TOTAL_SECS`], and despawns itself.
///
/// The overlay is a sibling of every other UI surface — it never
/// becomes a parent of game systems, and the deal animation runs
/// underneath it during the hold window. Dismissal on any keypress
/// / click / touch shortcuts the timeline into the fade-out phase
/// rather than despawning instantly, so the dissolve always plays.
pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_splash).add_systems(
            Update,
            (dismiss_splash_on_input, advance_splash).chain(),
        );
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker on the splash overlay scrim (root entity for the launch beat).
/// Despawned with descendants once [`MOTION_SPLASH_TOTAL_SECS`] elapses
/// or once a user-input dismissal advances the timeline past the hold.
#[derive(Component, Debug)]
pub struct SplashRoot;

/// Tracks the splash's elapsed visible duration. Stored as a component
/// on the splash root rather than a global resource so despawning the
/// splash root removes its state along with it — there's no second-run
/// concern (the splash is one-shot at app start) and a component keeps
/// the splash data co-located with its entity.
#[derive(Component, Debug, Default)]
pub struct SplashAge(pub Duration);

/// Marks a `Text` entity whose `TextColor` should fade with the splash
/// timeline. `base_color` is the full-alpha target colour written by
/// [`advance_splash`]; the system multiplies its alpha by the current
/// fade factor each tick.
#[derive(Component, Debug, Clone, Copy)]
struct SplashFadable {
    base_color: Color,
}

/// Marks a `Node` entity whose `BackgroundColor` should fade with the
/// splash timeline. Same contract as [`SplashFadable`] but for nodes
/// whose visible colour lives on the background, not on text — palette
/// swatches, the progress bar track, and the progress bar fill.
#[derive(Component, Debug, Clone, Copy)]
struct SplashFadableBg {
    base_color: Color,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the splash overlay at `Startup`. Builds a fullscreen scrim
/// at alpha 0 (so the first paint is invisible — the first
/// `advance_splash` tick lifts every fadable's alpha), composes the
/// header / boot-log / progress / footer hierarchy, and tags every
/// visible child with [`SplashFadable`] or [`SplashFadableBg`] so the
/// per-frame fade has a uniform target list.
///
/// **Skipped on subsequent launches.** If `SettingsResource` reports
/// `first_run_complete == true`, the player has already seen the
/// brand beat at least once and we go straight to gameplay — having
/// to wait 1.6 s on every launch wears thin fast. The splash still
/// shows on first run, after a save reset (settings.json deleted),
/// and under `MinimalPlugins` (no `SettingsResource` registered) so
/// the test fixture observes the same spawn it always did.
fn spawn_splash(
    mut commands: Commands,
    font_res: Option<Res<FontResource>>,
    settings: Option<Res<SettingsResource>>,
) {
    if let Some(settings) = settings.as_deref()
        && settings.0.first_run_complete
    {
        return;
    }

    let font_handle = font_res.map(|f| f.0.clone()).unwrap_or_default();

    commands
        .spawn((
            SplashRoot,
            SplashAge(Duration::ZERO),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                // SpaceBetween distributes the three top-level groups
                // (header / centre / footer) so the header sits near
                // the top, the centre column floats in the middle of
                // the viewport, and the footer hugs the bottom edge —
                // mirroring the mockup's `justify-between` body.
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(VAL_SPACE_5, VAL_SPACE_7),
                ..default()
            },
            BackgroundColor(scrim_with_alpha(0.0)),
            GlobalZIndex(Z_SPLASH),
        ))
        .with_children(|root| {
            spawn_header_section(root, &font_handle);
            spawn_centre_section(root, &font_handle);
            spawn_footer_section(root, &font_handle);
        });
}

/// Header section: cursor block, wordmark, divider, "TERMINAL EDITION"
/// label. Stacked vertically and centre-aligned. Renders near the top
/// of the viewport thanks to the root's `justify-between`.
fn spawn_header_section(parent: &mut ChildSpawnerCommands, font_handle: &Handle<Font>) {
    let cursor_font = TextFont {
        font: font_handle.clone(),
        // Larger than TYPE_DISPLAY so the cursor block reads as the
        // signature element above the wordmark. Hand-tuned literal —
        // a one-off display character outside the regular text scale.
        font_size: 96.0,
        ..default()
    };
    let title_font = TextFont {
        font: font_handle.clone(),
        font_size: TYPE_DISPLAY,
        ..default()
    };
    let subtitle_font = TextFont {
        font: font_handle.clone(),
        font_size: TYPE_CAPTION,
        ..default()
    };

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: VAL_SPACE_2,
            margin: UiRect::top(VAL_SPACE_6),
            ..default()
        })
        .with_children(|hdr| {
            hdr.spawn((
                SplashFadable { base_color: ACCENT_PRIMARY },
                Text::new("\u{258C}"), // ▌ — the Terminal cursor block.
                cursor_font,
                TextColor(transparent(ACCENT_PRIMARY)),
            ));
            hdr.spawn((
                SplashFadable { base_color: TEXT_PRIMARY },
                Text::new("Solitaire Quest"),
                title_font,
                TextColor(transparent(TEXT_PRIMARY)),
            ));
            // Thin horizontal divider under the wordmark — same hue as
            // every other 1px chrome line in the design system.
            hdr.spawn((
                SplashFadableBg { base_color: BORDER_SUBTLE },
                Node {
                    width: Val::Px(192.0),
                    height: Val::Px(1.0),
                    ..default()
                },
                BackgroundColor(transparent(BORDER_SUBTLE)),
            ));
            hdr.spawn((
                SplashFadable { base_color: TEXT_DISABLED },
                Text::new("TERMINAL EDITION"),
                subtitle_font,
                TextColor(transparent(TEXT_DISABLED)),
            ));
        });
}

/// Centre section: boot log + progress bar. The boot-log column is
/// capped at 480 px on desktop per `docs/ui-mockups/desktop-adaptation.md`
/// (otherwise 70 % of viewport width). The progress bar is capped at
/// 720 px likewise.
fn spawn_centre_section(parent: &mut ChildSpawnerCommands, font_handle: &Handle<Font>) {
    let line_font = TextFont {
        font: font_handle.clone(),
        font_size: TYPE_CAPTION,
        ..default()
    };

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: VAL_SPACE_5,
            ..default()
        })
        .with_children(|centre| {
            spawn_boot_log(centre, &line_font);
            spawn_progress_bar(centre, &line_font);
        });
}

/// Boot-log column: three lime check rows + a "▌ ready_" line. Content
/// is fixture text, not driven from real bootstrap state — the splash
/// is a brand beat, not a real loader. Capped at 480 px width on
/// desktop (the design-system spec calls 70 % of mobile viewport,
/// which would stretch oddly on a wide window).
fn spawn_boot_log(parent: &mut ChildSpawnerCommands, line_font: &TextFont) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Start,
            row_gap: VAL_SPACE_1,
            width: Val::Percent(70.0),
            max_width: Val::Px(480.0),
            ..default()
        })
        .with_children(|log| {
            for label in ["assets loaded", "theme: terminal", "progress restored"] {
                spawn_check_row(log, line_font, label);
            }
            spawn_ready_row(log, line_font);
        });
}

/// One ✓-prefixed boot-log line. The check glyph is lime
/// (`STATE_SUCCESS`) so it reads as "complete"; the description text
/// is `TEXT_DISABLED` (the muted gray rung) so the eye treats the
/// list as background log noise rather than information that needs
/// reading.
fn spawn_check_row(parent: &mut ChildSpawnerCommands, line_font: &TextFont, label: &str) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: VAL_SPACE_2,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                SplashFadable { base_color: STATE_SUCCESS },
                Text::new("\u{2713}"), // ✓
                line_font.clone(),
                TextColor(transparent(STATE_SUCCESS)),
            ));
            row.spawn((
                SplashFadable { base_color: TEXT_DISABLED },
                Text::new(label.to_string()),
                line_font.clone(),
                TextColor(transparent(TEXT_DISABLED)),
            ));
        });
}

/// "▌ ready_" line — visual signature of "boot complete, awaiting
/// input". Static; no pulse animation in this commit (a pulse would
/// fight the global fade timeline). The cursor glyph picks up
/// `TEXT_PRIMARY` rather than `ACCENT_PRIMARY` so it doesn't compete
/// with the big cyan cursor in the header.
fn spawn_ready_row(parent: &mut ChildSpawnerCommands, line_font: &TextFont) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: VAL_SPACE_2,
            margin: UiRect::top(VAL_SPACE_2),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                SplashFadable { base_color: TEXT_PRIMARY },
                Text::new("\u{258C} ready_"), // ▌ ready_
                line_font.clone(),
                TextColor(transparent(TEXT_PRIMARY)),
            ));
        });
}

/// Progress bar — a 1 px tall track in `BORDER_SUBTLE` with a 100 %-
/// width cyan fill, plus a `DONE · 247 ASSETS` caption right-aligned
/// below. The "247" is fixture text; the bar is decorative, not a
/// real progress signal. Capped at 720 px width on desktop.
fn spawn_progress_bar(parent: &mut ChildSpawnerCommands, line_font: &TextFont) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: VAL_SPACE_2,
            width: Val::Percent(80.0),
            max_width: Val::Px(720.0),
            ..default()
        })
        .with_children(|bar| {
            // Track.
            bar.spawn((
                SplashFadableBg { base_color: BORDER_SUBTLE },
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    ..default()
                },
                BackgroundColor(transparent(BORDER_SUBTLE)),
            ))
            .with_children(|track| {
                // Fill — 100 % of the track width = "complete".
                track.spawn((
                    SplashFadableBg { base_color: ACCENT_PRIMARY },
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(transparent(ACCENT_PRIMARY)),
                ));
            });
            // Caption — right-aligned below the bar.
            bar.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            })
            .with_children(|caption| {
                caption.spawn((
                    SplashFadable { base_color: TEXT_DISABLED },
                    Text::new("DONE \u{00B7} 247 ASSETS"), // DONE · 247 ASSETS
                    line_font.clone(),
                    TextColor(transparent(TEXT_DISABLED)),
                ));
            });
        });
}

/// Footer section: "BASE16-EIGHTIES" label, eight palette swatches,
/// version line. The swatches are 12 × 12 px coloured squares, one
/// per named token — visible signature of the design system.
fn spawn_footer_section(parent: &mut ChildSpawnerCommands, font_handle: &Handle<Font>) {
    let footer_font = TextFont {
        font: font_handle.clone(),
        font_size: TYPE_CAPTION,
        ..default()
    };

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: VAL_SPACE_3,
            ..default()
        })
        .with_children(|footer| {
            footer.spawn((
                SplashFadable { base_color: TEXT_DISABLED },
                Text::new("BASE16-EIGHTIES"),
                footer_font.clone(),
                TextColor(transparent(TEXT_DISABLED)),
            ));
            spawn_palette_swatch_row(footer);
            footer.spawn((
                SplashFadable { base_color: TEXT_DISABLED },
                Text::new(format!("v{}", env!("CARGO_PKG_VERSION"))),
                footer_font.clone(),
                TextColor(transparent(TEXT_DISABLED)),
            ));
        });
}

/// Eight 12 × 12 px palette squares — one per named design-system
/// token (suit-red / warning / success / info / primary / celebration
/// / on-surface / outline). The order matches the mockup; the row is
/// the visual signature of the palette behind the rest of the UI.
fn spawn_palette_swatch_row(parent: &mut ChildSpawnerCommands) {
    let swatches = [
        STATE_DANGER,
        STATE_WARNING,
        STATE_SUCCESS,
        STATE_INFO,
        ACCENT_PRIMARY,
        ACCENT_SECONDARY,
        TEXT_PRIMARY,
        // `BORDER_STRONG` (`#505050`) is the eighth slot — `outline`
        // in the design-system token spec, also exposed as
        // `TEXT_DISABLED` since the two share a hue. Re-using the
        // existing `TEXT_DISABLED` import keeps the swatch list a
        // single read.
        TEXT_DISABLED,
    ];
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: VAL_SPACE_1,
            ..default()
        })
        .with_children(|row| {
            for color in swatches {
                row.spawn((
                    SplashFadableBg { base_color: color },
                    Node {
                        width: Val::Px(12.0),
                        height: Val::Px(12.0),
                        ..default()
                    },
                    BackgroundColor(transparent(color)),
                ));
            }
        });
}

/// Returns `BG_BASE` with its alpha multiplied by `factor` (0–1). The
/// fade systems lerp this each tick to drive the scrim's dissolve.
fn scrim_with_alpha(factor: f32) -> Color {
    let mut c = BG_BASE;
    c.set_alpha(factor.clamp(0.0, 1.0));
    c
}

/// Returns `c` with alpha 0. Initial paint colour for every fadable
/// element so the very first frame is fully transparent — the next
/// `advance_splash` tick lifts the alpha based on `SplashAge`.
fn transparent(c: Color) -> Color {
    let mut out = c;
    out.set_alpha(0.0);
    out
}

/// Computes the splash's per-frame alpha from its age. Three phases:
///
/// * `0..fade` — fade-in: `alpha = age / fade`.
/// * `fade..total - fade` — hold: `alpha = 1.0`.
/// * `total - fade..total` — fade-out: `alpha = (total - age) / fade`.
/// * `>= total` — splash is complete; caller despawns the root.
///
/// Returns `None` once the timeline is finished, signalling the splash
/// should be despawned.
fn splash_alpha(age: Duration) -> Option<f32> {
    let age_s = age.as_secs_f32();
    let total = MOTION_SPLASH_TOTAL_SECS;
    let fade = MOTION_SPLASH_FADE_SECS;

    if age_s >= total {
        return None;
    }
    if age_s < fade {
        // Fade-in.
        return Some((age_s / fade).clamp(0.0, 1.0));
    }
    if age_s < total - fade {
        // Hold.
        return Some(1.0);
    }
    // Fade-out.
    Some(((total - age_s) / fade).clamp(0.0, 1.0))
}

/// Advances every splash root's age by `time.delta()` and updates the
/// scrim plus every [`SplashFadable`] / [`SplashFadableBg`] alpha,
/// despawning the splash once the timeline finishes. Despawns with
/// descendants so the entire hierarchy leaves the world together.
///
/// The fadable queries are global (no parent constraint) — the splash
/// is a one-shot at app start and is the only owner of these markers,
/// so there is no contamination risk from other plugins.
#[allow(clippy::type_complexity)]
fn advance_splash(
    mut commands: Commands,
    time: Res<Time>,
    mut roots: Query<(Entity, &mut SplashAge, &mut BackgroundColor), With<SplashRoot>>,
    mut fadable_texts: Query<(&SplashFadable, &mut TextColor)>,
    mut fadable_bgs: Query<(&SplashFadableBg, &mut BackgroundColor), Without<SplashRoot>>,
) {
    for (entity, mut age, mut bg) in &mut roots {
        age.0 = age.0.saturating_add(time.delta());
        let Some(alpha) = splash_alpha(age.0) else {
            commands.entity(entity).despawn();
            continue;
        };

        bg.0 = scrim_with_alpha(alpha);

        for (fadable, mut text_color) in &mut fadable_texts {
            let mut c = fadable.base_color;
            c.set_alpha(alpha);
            text_color.0 = c;
        }
        for (fadable, mut bg_color) in &mut fadable_bgs {
            let mut c = fadable.base_color;
            c.set_alpha(alpha);
            bg_color.0 = c;
        }
    }
}

/// Dismisses the splash on any user input. Accelerates each splash
/// root's age into the fade-out window so the dissolve still plays
/// (despawning instantly would feel abrupt). If the timeline is
/// already inside fade-out, the splash is left to finish on its own.
///
/// **Input is not consumed.** The splash neither calls
/// `clear_just_pressed` nor drains the touch / mouse buffers, so a
/// keystroke that dismissed the splash also reaches downstream
/// systems on the same tick (e.g. Space → `DrawRequestEvent`). This
/// matches what the user expects — the splash is a brand beat, not a
/// modal stop sign.
fn dismiss_splash_on_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Option<Res<Touches>>,
    mut roots: Query<&mut SplashAge, With<SplashRoot>>,
) {
    if roots.is_empty() {
        return;
    }

    let touch_pressed = touches.is_some_and(|t| t.iter_just_pressed().next().is_some());
    let dismissed = keys.get_just_pressed().next().is_some()
        || mouse.get_just_pressed().next().is_some()
        || touch_pressed;

    if !dismissed {
        return;
    }

    // Jump the age forward to the start of the fade-out so the
    // overlay dissolves cleanly. Saturating arithmetic on Duration
    // means an already-past-fade-out splash stays past fade-out.
    let fade_out_start = Duration::from_secs_f32(
        (MOTION_SPLASH_TOTAL_SECS - MOTION_SPLASH_FADE_SECS).max(0.0),
    );
    for mut age in &mut roots {
        if age.0 < fade_out_start {
            age.0 = fade_out_start;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;

    /// Builds a headless `App` with `MinimalPlugins + SplashPlugin` and
    /// runs one tick so `spawn_splash` (Startup) has executed before
    /// the first asserting `update`.
    fn headless_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(SplashPlugin);
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<ButtonInput<MouseButton>>();
        app.update();
        app
    }

    /// Tells `TimePlugin` to advance the virtual clock by `secs` on the
    /// next `app.update()`. Mirrors the helper in `ui_tooltip::tests`.
    fn set_manual_time_step(app: &mut App, secs: f32) {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            Duration::from_secs_f32(secs),
        ));
    }

    /// `Time<Virtual>` clamps per-tick deltas to `max_delta` (default
    /// 250 ms) regardless of the requested manual step, so we drive
    /// 200 ms ticks and call `update` enough times to exceed the
    /// target duration. Returns the splash root's recorded age after
    /// the stepping completes (or `None` if the splash was despawned).
    fn advance_by(app: &mut App, total_secs: f32) -> Option<Duration> {
        set_manual_time_step(app, 0.2);
        let ticks = (total_secs / 0.2).ceil() as usize + 1;
        for _ in 0..ticks {
            app.update();
        }
        let mut q = app
            .world_mut()
            .query_filtered::<&SplashAge, With<SplashRoot>>();
        q.iter(app.world()).next().map(|a| a.0)
    }

    fn count_splash_roots(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<Entity, With<SplashRoot>>()
            .iter(app.world())
            .count()
    }

    fn press_key(app: &mut App, key: KeyCode) {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.release_all();
        input.clear();
        input.press(key);
    }

    fn press_mouse(app: &mut App, button: MouseButton) {
        let mut input = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        input.release_all();
        input.clear();
        input.press(button);
    }

    /// Reads the splash scrim's `BackgroundColor` alpha. Panics if the
    /// splash root is missing — that's a regression in `spawn_splash`.
    fn scrim_alpha(app: &mut App) -> f32 {
        let mut q = app
            .world_mut()
            .query_filtered::<&BackgroundColor, With<SplashRoot>>();
        q.iter(app.world())
            .next()
            .expect("SplashRoot should exist")
            .0
            .alpha()
    }

    #[test]
    fn splash_spawns_on_startup() {
        let mut app = headless_app();
        assert_eq!(
            count_splash_roots(&mut app),
            1,
            "SplashRoot must exist after Startup"
        );
    }

    #[test]
    fn splash_skipped_when_first_run_complete() {
        use solitaire_data::Settings;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(SplashPlugin);
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<ButtonInput<MouseButton>>();
        app.insert_resource(SettingsResource(Settings {
            first_run_complete: true,
            ..Settings::default()
        }));
        app.update();
        assert_eq!(
            count_splash_roots(&mut app),
            0,
            "SplashRoot must NOT spawn on subsequent launches"
        );
    }

    #[test]
    fn splash_still_shows_when_first_run_incomplete() {
        use solitaire_data::Settings;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(SplashPlugin);
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<ButtonInput<MouseButton>>();
        app.insert_resource(SettingsResource(Settings {
            first_run_complete: false,
            ..Settings::default()
        }));
        app.update();
        assert_eq!(
            count_splash_roots(&mut app),
            1,
            "SplashRoot must spawn for first-run players (first_run_complete = false)"
        );
    }

    #[test]
    fn splash_despawns_after_total_duration() {
        let mut app = headless_app();
        let _ = advance_by(&mut app, MOTION_SPLASH_TOTAL_SECS + 0.5);
        assert_eq!(
            count_splash_roots(&mut app),
            0,
            "SplashRoot must be despawned after MOTION_SPLASH_TOTAL_SECS"
        );
    }

    #[test]
    fn splash_alpha_curves_through_fade_hold_fade() {
        // Pure-function test on the curve so we don't need to wrangle
        // the virtual-clock clamp here.
        // Start of fade-in.
        assert!(
            splash_alpha(Duration::ZERO).unwrap() < 0.05,
            "alpha at t=0 must be near 0 (fade-in start)"
        );
        // End of fade-in.
        let after_fade_in = Duration::from_secs_f32(MOTION_SPLASH_FADE_SECS);
        assert!(
            (splash_alpha(after_fade_in).unwrap() - 1.0).abs() < 0.001,
            "alpha at end of fade-in must be ~1.0"
        );
        // Mid-hold.
        let mid_hold = Duration::from_secs_f32(MOTION_SPLASH_TOTAL_SECS / 2.0);
        assert!(
            (splash_alpha(mid_hold).unwrap() - 1.0).abs() < f32::EPSILON,
            "alpha mid-hold must be exactly 1.0"
        );
        // Inside fade-out.
        let mid_fade_out = Duration::from_secs_f32(
            MOTION_SPLASH_TOTAL_SECS - MOTION_SPLASH_FADE_SECS / 2.0,
        );
        let mid_out_alpha = splash_alpha(mid_fade_out).unwrap();
        assert!(
            mid_out_alpha < 0.6 && mid_out_alpha > 0.4,
            "alpha mid-fade-out should be ~0.5, got {mid_out_alpha}"
        );
        // Past total.
        let past_total = Duration::from_secs_f32(MOTION_SPLASH_TOTAL_SECS + 0.1);
        assert!(
            splash_alpha(past_total).is_none(),
            "alpha past total duration must be None (signal: despawn)"
        );
    }

    #[test]
    fn splash_dismisses_immediately_on_keypress() {
        let mut app = headless_app();
        set_manual_time_step(&mut app, 0.05);
        app.update();
        let pre_alpha = scrim_alpha(&mut app);
        assert!(
            pre_alpha < 1.0,
            "precondition: splash should be inside fade-in, not yet at full alpha (got {pre_alpha})"
        );

        press_key(&mut app, KeyCode::Space);
        app.update();

        if count_splash_roots(&mut app) == 0 {
            return;
        }
        let mut q = app
            .world_mut()
            .query_filtered::<&SplashAge, With<SplashRoot>>();
        let age = q
            .iter(app.world())
            .next()
            .expect("splash should exist after one post-dismiss tick")
            .0;
        let fade_out_start = Duration::from_secs_f32(
            MOTION_SPLASH_TOTAL_SECS - MOTION_SPLASH_FADE_SECS,
        );
        assert!(
            age >= fade_out_start,
            "after a keypress dismiss the splash must be in fade-out (age >= {fade_out_start:?}); got {age:?}"
        );
    }

    #[test]
    fn splash_dismisses_on_mouse_click() {
        let mut app = headless_app();
        set_manual_time_step(&mut app, 0.05);
        app.update();
        assert!(scrim_alpha(&mut app) < 1.0);

        press_mouse(&mut app, MouseButton::Left);
        app.update();

        if count_splash_roots(&mut app) == 0 {
            return;
        }
        let mut q = app
            .world_mut()
            .query_filtered::<&SplashAge, With<SplashRoot>>();
        let age = q
            .iter(app.world())
            .next()
            .expect("splash should exist after one post-dismiss tick")
            .0;
        let fade_out_start = Duration::from_secs_f32(
            MOTION_SPLASH_TOTAL_SECS - MOTION_SPLASH_FADE_SECS,
        );
        assert!(
            age >= fade_out_start,
            "after a left-click dismiss the splash must be in fade-out; got {age:?}"
        );
    }

    /// Bonus test: dismissing the splash with a keypress does NOT clear
    /// that key's `just_pressed` flag — downstream systems still see
    /// the keystroke that dismissed the splash.
    #[test]
    fn dismissal_keypress_is_visible_to_other_systems() {
        let mut app = headless_app();
        press_key(&mut app, KeyCode::Space);
        app.update();
        let keys = app.world().resource::<ButtonInput<KeyCode>>();
        assert!(
            keys.just_pressed(KeyCode::Space),
            "Splash dismissal must NOT consume the input — downstream gameplay still needs it"
        );
    }

    /// The Terminal boot-screen content must include the four
    /// signature elements: cursor block, wordmark, "TERMINAL EDITION"
    /// subtitle, and at least one boot-log row. Catches a regression
    /// where the spawn hierarchy gets simplified back to "title +
    /// version" — the splash is intentionally rich now.
    #[test]
    fn splash_renders_terminal_boot_screen_content() {
        let mut app = headless_app();
        let texts: Vec<String> = app
            .world_mut()
            .query::<&Text>()
            .iter(app.world())
            .map(|t| t.0.clone())
            .collect();
        assert!(
            texts.iter().any(|t| t == "\u{258C}"),
            "expected the cursor block (▌) on the splash, got: {texts:?}"
        );
        assert!(
            texts.iter().any(|t| t == "Solitaire Quest"),
            "expected the wordmark on the splash, got: {texts:?}"
        );
        assert!(
            texts.iter().any(|t| t == "TERMINAL EDITION"),
            "expected the TERMINAL EDITION subtitle on the splash, got: {texts:?}"
        );
        assert!(
            texts.iter().any(|t| t == "assets loaded"),
            "expected at least one boot-log row, got: {texts:?}"
        );
        assert!(
            texts.iter().any(|t| t == "BASE16-EIGHTIES"),
            "expected the BASE16-EIGHTIES footer label, got: {texts:?}"
        );
    }

    /// Every fadable element starts at alpha 0 (fade-in begins from
    /// fully transparent) and lifts to ~1.0 by the end of the fade-in
    /// window. Catches a regression where a new fadable's initial
    /// paint is full-alpha — that flashes a frame of fully-visible
    /// content before the first `advance_splash` tick lerps it down.
    #[test]
    fn fadables_start_transparent_and_reach_full_alpha() {
        let mut app = headless_app();
        // Right after Startup, before any time has advanced, every
        // fadable element should still carry alpha 0 (the spawn
        // function paints them transparent and the first tick has
        // already run alpha = 0 / fade ≈ 0). We allow a tiny epsilon
        // for floating-point lift on the very first tick.
        let initial_text_alphas: Vec<f32> = app
            .world_mut()
            .query::<(&SplashFadable, &TextColor)>()
            .iter(app.world())
            .map(|(_, color)| color.0.alpha())
            .collect();
        assert!(
            initial_text_alphas.iter().all(|a| *a <= 0.05),
            "fadable text alphas should start near 0; got {initial_text_alphas:?}"
        );

        // Advance past the fade-in window. Every fadable should now
        // be at full alpha.
        let _ = advance_by(&mut app, MOTION_SPLASH_FADE_SECS + 0.4);
        if count_splash_roots(&mut app) == 0 {
            return; // already past fade-out under the test clock — skip.
        }
        let mid_text_alphas: Vec<f32> = app
            .world_mut()
            .query::<(&SplashFadable, &TextColor)>()
            .iter(app.world())
            .map(|(_, color)| color.0.alpha())
            .collect();
        assert!(
            mid_text_alphas.iter().all(|a| *a >= 0.9),
            "fadable text alphas should be at full alpha during the hold; got {mid_text_alphas:?}"
        );
    }
}
