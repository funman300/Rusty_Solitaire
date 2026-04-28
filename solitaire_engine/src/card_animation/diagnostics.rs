//! Lightweight frame-time diagnostics.
//!
//! [`FrameTimeDiagnostics`] is a Bevy resource that maintains a rolling window
//! of the last [`WINDOW_SIZE`] frame durations. Any system can read it to make
//! performance-aware decisions — for example, disabling settle-bounce animations
//! when the game is running below 30 FPS on a low-end device.
//!
//! # Reading diagnostics
//!
//! ```ignore
//! fn my_system(diag: Res<FrameTimeDiagnostics>) {
//!     if diag.is_low_performance() {
//!         // Skip expensive visual effects.
//!         return;
//!     }
//!     println!("avg FPS: {:.1}", diag.fps());
//! }
//! ```
//!
//! # Update
//!
//! [`update_frame_time_diagnostics`] runs every frame via [`CardAnimationPlugin`]
//! (or whichever plugin registers it). The window is circular so only the last
//! `WINDOW_SIZE` frames influence the statistics.

use bevy::prelude::*;

/// Number of frames kept in the rolling statistics window.
pub const WINDOW_SIZE: usize = 60;

/// Rolling frame-time statistics over the last [`WINDOW_SIZE`] frames.
///
/// All times are in seconds. Statistics are updated every frame by
/// [`update_frame_time_diagnostics`].
#[derive(Resource, Debug)]
pub struct FrameTimeDiagnostics {
    samples: [f32; WINDOW_SIZE],
    head: usize,
    count: usize,
    /// Smoothed average frame duration over the window (seconds).
    pub avg_secs: f32,
    /// Worst-case (slowest) frame duration in the window (seconds).
    pub max_secs: f32,
    /// Best-case (fastest) frame duration in the window (seconds).
    pub min_secs: f32,
}

impl Default for FrameTimeDiagnostics {
    fn default() -> Self {
        Self {
            samples: [0.0; WINDOW_SIZE],
            head: 0,
            count: 0,
            avg_secs: 0.0,
            max_secs: 0.0,
            min_secs: 0.0,
        }
    }
}

impl FrameTimeDiagnostics {
    /// Estimated frames per second based on the rolling average.
    ///
    /// Returns `0.0` until at least one frame has been recorded.
    pub fn fps(&self) -> f32 {
        if self.avg_secs > 0.0 {
            1.0 / self.avg_secs
        } else {
            0.0
        }
    }

    /// Returns `true` when the rolling-average FPS is above `target`.
    ///
    /// Always returns `false` until the window is fully populated.
    pub fn is_above_target(&self, target_fps: f32) -> bool {
        self.count >= WINDOW_SIZE && self.fps() > target_fps
    }

    /// Returns `true` when the device appears to be running below 30 FPS.
    ///
    /// Only asserted after the window is fully populated so a single slow
    /// startup frame does not permanently suppress visual effects.
    pub fn is_low_performance(&self) -> bool {
        self.count >= WINDOW_SIZE && self.fps() < 30.0
    }

    /// Appends `dt` to the ring buffer and recomputes statistics.
    ///
    /// O(WINDOW_SIZE) — acceptable because WINDOW_SIZE is small and constant.
    fn push(&mut self, dt: f32) {
        self.samples[self.head] = dt;
        self.head = (self.head + 1) % WINDOW_SIZE;
        if self.count < WINDOW_SIZE {
            self.count += 1;
        }

        let n = self.count;
        let mut sum = 0.0_f32;
        let mut max_val = 0.0_f32;
        let mut min_val = f32::MAX;

        for &s in &self.samples[..n] {
            sum += s;
            if s > max_val {
                max_val = s;
            }
            if s < min_val {
                min_val = s;
            }
        }

        self.avg_secs = sum / n as f32;
        self.max_secs = max_val;
        self.min_secs = if min_val == f32::MAX { 0.0 } else { min_val };
    }
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Records the current frame's delta time in [`FrameTimeDiagnostics`].
///
/// Registered by [`CardAnimationPlugin`]. Runs every frame in `Update`.
pub(crate) fn update_frame_time_diagnostics(
    time: Res<Time>,
    mut diag: ResMut<FrameTimeDiagnostics>,
) {
    diag.push(time.delta_secs());
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fps_zero_when_no_samples() {
        assert_eq!(FrameTimeDiagnostics::default().fps(), 0.0);
    }

    #[test]
    fn fps_correct_after_uniform_frames() {
        let mut d = FrameTimeDiagnostics::default();
        for _ in 0..WINDOW_SIZE {
            d.push(1.0 / 60.0);
        }
        assert!(
            (d.fps() - 60.0).abs() < 0.5,
            "expected ~60 fps, got {}",
            d.fps()
        );
    }

    #[test]
    fn is_low_performance_requires_full_window() {
        let mut d = FrameTimeDiagnostics::default();
        // Partial window filled with very slow frames.
        for _ in 0..(WINDOW_SIZE / 2) {
            d.push(1.0 / 5.0); // 5 FPS
        }
        assert!(
            !d.is_low_performance(),
            "must not report low performance until the window is full"
        );
    }

    #[test]
    fn is_low_performance_true_below_30fps() {
        let mut d = FrameTimeDiagnostics::default();
        for _ in 0..WINDOW_SIZE {
            d.push(1.0 / 20.0); // 20 FPS
        }
        assert!(
            d.is_low_performance(),
            "20 FPS should be reported as low performance"
        );
    }

    #[test]
    fn is_above_target_false_below_target() {
        let mut d = FrameTimeDiagnostics::default();
        for _ in 0..WINDOW_SIZE {
            d.push(1.0 / 30.0); // exactly 30 FPS
        }
        // is_above_target(30.0) is strict: fps must be > 30, not >=.
        // At exactly 30 FPS the result depends on floating-point rounding,
        // so just check that it's consistent with > 60 being false.
        assert!(!d.is_above_target(60.0), "30 FPS is not above 60 FPS target");
    }

    #[test]
    fn max_and_min_track_extremes() {
        let mut d = FrameTimeDiagnostics::default();
        d.push(0.010); // fast frame (100 FPS)
        d.push(0.050); // slow frame (20 FPS)
        assert!(
            d.max_secs >= 0.050,
            "max_secs must be at least the slow frame, got {}",
            d.max_secs
        );
        assert!(
            d.min_secs <= 0.010,
            "min_secs must be at most the fast frame, got {}",
            d.min_secs
        );
    }

    #[test]
    fn circular_buffer_overwrites_oldest() {
        let mut d = FrameTimeDiagnostics::default();
        // Fill with 60-FPS samples.
        for _ in 0..WINDOW_SIZE {
            d.push(1.0 / 60.0);
        }
        // Overwrite every slot with 10-FPS samples.
        for _ in 0..WINDOW_SIZE {
            d.push(1.0 / 10.0);
        }
        assert!(
            d.fps() < 15.0,
            "after full overwrite, avg must reflect new slow frames; got fps={}",
            d.fps()
        );
    }

    #[test]
    fn count_does_not_exceed_window_size() {
        let mut d = FrameTimeDiagnostics::default();
        for _ in 0..WINDOW_SIZE * 3 {
            d.push(0.016);
        }
        assert_eq!(d.count, WINDOW_SIZE);
    }
}
