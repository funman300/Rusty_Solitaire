//! Animation chaining — play a sequence of [`CardAnimation`] segments in order.
//!
//! Insert [`AnimationChain`] on a card entity alongside the *first* segment as
//! a [`CardAnimation`] to sequence multi-step motion. When the active
//! [`CardAnimation`] finishes and is removed, [`advance_animation_chains`]
//! pops the next segment and inserts it automatically.
//!
//! # Example — arc then settle
//!
//! ```ignore
//! // Arc up to a midpoint, then settle onto the foundation with a soft bounce.
//! let mid = (start + end) / 2.0 + Vec2::new(0.0, 30.0);
//!
//! let first_leg = CardAnimation::slide(start, z, mid, z + 20.0, MotionCurve::SmoothSnap)
//!     .with_z_lift(15.0);
//! let second_leg = CardAnimation::slide(mid, z + 20.0, end, resting_z, MotionCurve::SoftBounce);
//!
//! commands.entity(card_entity).insert((
//!     first_leg,                              // plays immediately
//!     AnimationChain::new().then(second_leg), // queued
//! ));
//! ```
//!
//! # Invariant
//!
//! The chain holds only the *queued* segments — the segment currently playing
//! lives on the entity as a [`CardAnimation`] component and has already been
//! removed from the queue. When the queue is exhausted the `AnimationChain`
//! component removes itself.

use std::collections::VecDeque;

use bevy::prelude::*;

use super::animation::CardAnimation;

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// A FIFO queue of [`CardAnimation`] segments to be played one after another.
///
/// The currently playing segment lives on the entity as a [`CardAnimation`]
/// component (already removed from this queue). When that animation completes,
/// [`advance_animation_chains`] pops the next entry and inserts it.
///
/// Remove this component to cancel the entire chain mid-flight. The in-progress
/// [`CardAnimation`] (if any) will still play to completion unless also removed.
#[derive(Component, Debug, Clone)]
pub struct AnimationChain {
    pub(crate) queue: VecDeque<CardAnimation>,
}

impl AnimationChain {
    /// Creates an empty chain with no queued segments.
    #[must_use]
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// Appends `anim` to the end of the chain.
    ///
    /// Returns `self` for builder-style chaining.
    #[must_use]
    pub fn then(mut self, anim: CardAnimation) -> Self {
        self.queue.push_back(anim);
        self
    }

    /// Number of segments waiting in the queue (not including any
    /// currently active [`CardAnimation`]).
    pub fn remaining(&self) -> usize {
        self.queue.len()
    }

    /// Returns `true` when no segments remain in the queue.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

impl Default for AnimationChain {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Pops the next queued segment when the active [`CardAnimation`] has finished.
///
/// Must run **after** `advance_card_animations` so the completed animation has
/// already been removed before this system inspects the entity.
pub(crate) fn advance_animation_chains(
    mut commands: Commands,
    mut chains: Query<(Entity, &mut AnimationChain), Without<CardAnimation>>,
) {
    for (entity, mut chain) in &mut chains {
        match chain.queue.pop_front() {
            Some(next) => {
                // Insert the next segment; the chain component stays until empty.
                commands.entity(entity).insert(next);
            }
            None => {
                // Queue exhausted — clean up the chain component.
                commands.entity(entity).remove::<AnimationChain>();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card_animation::MotionCurve;

    fn slide(end_x: f32) -> CardAnimation {
        CardAnimation::slide(
            Vec2::ZERO,
            0.0,
            Vec2::new(end_x, 0.0),
            0.0,
            MotionCurve::SmoothSnap,
        )
    }

    #[test]
    fn new_chain_is_empty() {
        let c = AnimationChain::new();
        assert_eq!(c.remaining(), 0);
        assert!(c.is_empty());
    }

    #[test]
    fn then_appends_and_increments_remaining() {
        let c = AnimationChain::new().then(slide(1.0)).then(slide(2.0));
        assert_eq!(c.remaining(), 2);
        assert!(!c.is_empty());
    }

    #[test]
    fn queue_is_fifo() {
        let mut c = AnimationChain::new().then(slide(1.0)).then(slide(2.0));
        let first = c.queue.pop_front().expect("must have first segment");
        assert!(
            (first.end.x - 1.0).abs() < 1e-6,
            "first dequeued must be the first appended (end.x=1), got {}",
            first.end.x
        );
        let second = c.queue.pop_front().expect("must have second segment");
        assert!(
            (second.end.x - 2.0).abs() < 1e-6,
            "second dequeued must be the second appended (end.x=2), got {}",
            second.end.x
        );
    }

    #[test]
    fn default_equals_new() {
        assert_eq!(AnimationChain::default().remaining(), 0);
    }

    #[test]
    fn chain_with_three_segments() {
        let c = AnimationChain::new()
            .then(slide(1.0))
            .then(slide(2.0))
            .then(slide(3.0));
        assert_eq!(c.remaining(), 3);
    }

    #[test]
    fn advance_system_inserts_next_segment() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(crate::card_animation::CardAnimationPlugin);

        let chain = AnimationChain::new().then(slide(100.0));
        // Spawn an entity with only AnimationChain (no CardAnimation) so the
        // system fires immediately on the first update.
        let entity = app
            .world_mut()
            .spawn((Transform::from_translation(Vec3::ZERO), chain))
            .id();

        app.update();

        // After one update, the chain system should have popped `slide(100)` and
        // inserted it as a `CardAnimation`.
        assert!(
            app.world().entity(entity).get::<CardAnimation>().is_some(),
            "advance_animation_chains must insert CardAnimation from first queued segment"
        );
        // The chain component should still be present (but now empty).
        // Actually, since we popped the last item, the chain removes itself too.
        // Whether it's present or not depends on system ordering, but the
        // CardAnimation must definitely be present.
    }
}
