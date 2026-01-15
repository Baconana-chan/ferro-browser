/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Smooth scrolling animation implementation for Ferro Browser.
//!
//! This module provides momentum/inertia scrolling with easing for mouse wheel
//! and trackpad scrolling events.

use std::time::{Duration, Instant};

use webrender_api::units::{DevicePoint, DeviceVector2D};

/// Duration of the smooth scroll animation in milliseconds.
const SMOOTH_SCROLL_DURATION_MS: u64 = 300;

/// Easing function for smooth scrolling.
/// Uses an ease-out cubic function: 1 - (1 - t)^3
fn ease_out_cubic(t: f32) -> f32 {
    let t_inv = 1.0 - t;
    1.0 - t_inv * t_inv * t_inv
}

/// Represents an active smooth scroll animation.
#[derive(Clone, Debug)]
pub struct SmoothScrollAnimation {
    /// The starting scroll offset when the animation began.
    start_offset: DeviceVector2D,
    /// The target scroll offset to animate towards.
    target_offset: DeviceVector2D,
    /// The time when the animation started.
    start_time: Instant,
    /// Duration of the animation.
    duration: Duration,
}

impl SmoothScrollAnimation {
    /// Create a new smooth scroll animation.
    pub fn new(current_offset: DeviceVector2D, delta: DeviceVector2D) -> Self {
        Self {
            start_offset: current_offset,
            target_offset: current_offset + delta,
            start_time: Instant::now(),
            duration: Duration::from_millis(SMOOTH_SCROLL_DURATION_MS),
        }
    }

    /// Add more delta to the animation (for continued wheel scrolling).
    pub fn add_delta(&mut self, delta: DeviceVector2D) {
        // Update target while keeping the animation going
        self.target_offset = self.target_offset + delta;
        // Extend the animation duration from now
        self.start_offset = self.current_offset();
        self.start_time = Instant::now();
    }

    /// Get the current interpolated scroll offset.
    pub fn current_offset(&self) -> DeviceVector2D {
        let elapsed = self.start_time.elapsed();
        let progress = (elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0);
        let eased_progress = ease_out_cubic(progress);

        DeviceVector2D::new(
            self.start_offset.x + (self.target_offset.x - self.start_offset.x) * eased_progress,
            self.start_offset.y + (self.target_offset.y - self.start_offset.y) * eased_progress,
        )
    }

    /// Check if the animation is complete.
    pub fn is_complete(&self) -> bool {
        self.start_time.elapsed() >= self.duration
    }

    /// Get the target offset.
    pub fn target_offset(&self) -> DeviceVector2D {
        self.target_offset
    }
}

/// Manages smooth scroll state for a scroll container.
#[derive(Clone, Debug, Default)]
pub struct SmoothScrollState {
    /// Active animation, if any.
    animation: Option<SmoothScrollAnimation>,
    /// Last applied scroll offset for computing frame delta.
    last_applied_offset: DeviceVector2D,
    /// The scroll point (where user clicked/scrolled) for hit testing.
    scroll_point: Option<DevicePoint>,
}

impl SmoothScrollState {
    /// Create a new smooth scroll state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start or update a smooth scroll animation with the given delta.
    pub fn scroll_by(&mut self, delta: DeviceVector2D, scroll_point: DevicePoint) {
        match &mut self.animation {
            Some(animation) => {
                // Add to existing animation
                animation.add_delta(delta);
            },
            None => {
                // Start new animation from zero offset
                self.animation = Some(SmoothScrollAnimation::new(DeviceVector2D::zero(), delta));
                self.last_applied_offset = DeviceVector2D::zero();
            },
        }
        self.scroll_point = Some(scroll_point);
    }

    /// Get the delta to apply this frame, if any animation is active.
    /// Returns None if no animation is active or if animation is complete.
    pub fn frame_delta(&mut self) -> Option<(DeviceVector2D, DevicePoint)> {
        let animation = self.animation.as_ref()?;
        let scroll_point = self.scroll_point?;

        if animation.is_complete() {
            // Return final delta and clear animation
            let final_delta = animation.target_offset() - self.last_applied_offset;
            self.animation = None;
            self.scroll_point = None;
            if final_delta.x.abs() > 0.01 || final_delta.y.abs() > 0.01 {
                self.last_applied_offset = DeviceVector2D::zero();
                return Some((final_delta, scroll_point));
            }
            return None;
        }

        let current = animation.current_offset();
        let delta = current - self.last_applied_offset;
        self.last_applied_offset = current;

        // Only return delta if it's significant
        if delta.x.abs() > 0.01 || delta.y.abs() > 0.01 {
            Some((delta, scroll_point))
        } else {
            None
        }
    }

    /// Check if there's an active animation.
    pub fn is_animating(&self) -> bool {
        self.animation.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ease_out_cubic() {
        assert!((ease_out_cubic(0.0) - 0.0).abs() < 0.001);
        assert!((ease_out_cubic(1.0) - 1.0).abs() < 0.001);
        // Ease-out should be faster at the beginning
        assert!(ease_out_cubic(0.5) > 0.5);
    }

    #[test]
    fn test_smooth_scroll_animation() {
        let start = DeviceVector2D::new(0.0, 0.0);
        let delta = DeviceVector2D::new(0.0, 100.0);
        let animation = SmoothScrollAnimation::new(start, delta);

        // At start, current should be close to start
        let current = animation.current_offset();
        assert!((current.y - 0.0).abs() < 10.0);

        // Target should be start + delta
        assert!((animation.target_offset().y - 100.0).abs() < 0.001);
    }
}
