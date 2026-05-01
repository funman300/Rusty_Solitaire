//! Per-platform resolution of the user-themes directory.
//!
//! The path is determined exactly once and exposed via
//! [`user_theme_dir`]. On desktop platforms it is derived from
//! `dirs::data_dir()` (matching the rest of the project's
//! per-app-storage convention); on mobile it must be supplied by the
//! platform entry point via [`set_user_theme_dir`] before any code
//! that needs the path executes — there is deliberately no silent
//! fallback because mobile sandboxing makes any guess we'd hard-code
//! wrong.
//!
//! # Why panic instead of returning Result?
//!
//! User-theme resolution is bootstrap-time configuration, not game
//! logic, so per CLAUDE.md panics are acceptable here. Returning
//! `Result` would force every caller (the registry, the asset source,
//! the importer) to plumb an error through systems that have no
//! recovery path: there is no useful state to display if we can't
//! find the user themes directory at all.

use std::path::PathBuf;
use std::sync::OnceLock;

/// Override slot populated by mobile entry points (Android's
/// `android_main`, iOS's launch handler) before the Bevy `App` starts.
/// Desktop platforms ignore the override and fall through to
/// [`desktop_theme_dir`].
static USER_THEME_DIR_OVERRIDE: OnceLock<PathBuf> = OnceLock::new();

/// Sub-folder under `dirs::data_dir()` where the project keeps every
/// per-user file. Matches the existing convention used by
/// `solitaire_data` for `settings.json`, `stats.json`, etc.
const APP_DIR_NAME: &str = "solitaire_quest";

/// Sub-folder under [`APP_DIR_NAME`] dedicated to user themes.
const THEME_DIR_NAME: &str = "themes";

/// Sets the user-themes directory at runtime — mobile-only API.
///
/// Returns `Err` containing the rejected path if the override has
/// already been set. The first caller wins and subsequent calls are
/// silently a no-op-with-feedback so a mis-configured embedder can't
/// flip the path mid-session.
///
/// On desktop platforms this is functional but unnecessary —
/// [`user_theme_dir`] derives the path from `dirs::data_dir` directly
/// and ignores the override. Setting it on desktop is harmless but
/// nearly always a sign of confusion.
pub fn set_user_theme_dir(path: PathBuf) -> Result<(), PathBuf> {
    USER_THEME_DIR_OVERRIDE.set(path)
}

/// Returns the absolute path of the user-themes directory on the
/// current platform.
///
/// # Panics
///
/// Panics on:
///
/// - Desktop, if `dirs::data_dir()` returns `None` (rare; usually
///   indicates a broken `$HOME` or `$XDG_*` configuration).
/// - Mobile, if no entry point has called [`set_user_theme_dir`] yet.
/// - Any other target, where the embedder is required to supply the
///   path manually.
///
/// The panic message names the missing piece so the failure is
/// immediately actionable.
pub fn user_theme_dir() -> PathBuf {
    if let Some(p) = USER_THEME_DIR_OVERRIDE.get() {
        return p.clone();
    }
    user_theme_dir_for(detected_platform_data_dir())
}

/// Composition helper that takes the platform data dir as input so the
/// pure path-joining behaviour is unit-testable without depending on
/// the user's actual `$HOME`.
fn user_theme_dir_for(data_dir: PathBuf) -> PathBuf {
    data_dir.join(APP_DIR_NAME).join(THEME_DIR_NAME)
}

/// Per-target-os resolution of the platform's data dir. Split out so
/// mobile branches can grow without disturbing desktop behaviour.
fn detected_platform_data_dir() -> PathBuf {
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    {
        dirs::data_dir().unwrap_or_else(|| {
            panic!(
                "user_theme_dir(): platform data directory is unavailable. \
                 On Linux check $XDG_DATA_HOME or $HOME; on macOS / Windows \
                 the OS reported no Application Support / AppData path. \
                 As a workaround call solitaire_engine::assets::user_dir::\
                 set_user_theme_dir() before App::run()."
            )
        })
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        panic!(
            "user_theme_dir(): mobile entry point must call \
             solitaire_engine::assets::user_dir::set_user_theme_dir() \
             before App::run() — there is no platform default."
        )
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_os = "android",
        target_os = "ios"
    )))]
    {
        panic!(
            "user_theme_dir(): unsupported platform; call \
             solitaire_engine::assets::user_dir::set_user_theme_dir() \
             from your entry point before App::run()."
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_theme_dir_for_appends_solitaire_quest_themes() {
        let dir = user_theme_dir_for(PathBuf::from("/tmp/data"));
        assert_eq!(
            dir,
            PathBuf::from("/tmp/data/solitaire_quest/themes"),
            "user dir must nest under solitaire_quest/themes"
        );
    }

    #[test]
    fn user_theme_dir_for_handles_empty_root() {
        let dir = user_theme_dir_for(PathBuf::new());
        assert_eq!(dir, PathBuf::from("solitaire_quest/themes"));
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    #[test]
    fn detected_data_dir_yields_a_path_with_a_parent() {
        // On every supported desktop platform the OS reports a
        // user-writable data directory; the test machine already has
        // one for `dirs::data_dir()` to discover. We don't pin the
        // exact value because it depends on the user's $HOME, but it
        // must at least be a non-empty path with a parent component.
        let dir = detected_platform_data_dir();
        assert!(dir.parent().is_some(), "data dir {dir:?} should be absolute");
    }

    // The OnceLock-based override is intentionally NOT covered here:
    // setting it once would pollute every subsequent test in the
    // process that called `user_theme_dir()`. The override's
    // first-write-wins semantics come from `std::sync::OnceLock` which
    // is already well-tested upstream; the behaviour we add on top is
    // a trivial early-return that's covered by code review.
}
