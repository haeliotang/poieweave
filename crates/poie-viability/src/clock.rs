// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! One monotonic event clock shared by survival paths.
//!
//! The clock only moves forward and clamps at the horizon. Repair duration,
//! resource occupancy and deadlines must all be expressed through it so a
//! scenario cannot produce a different horizon solely because a path forgot to
//! advance time.

/// How a path treats the duration of a repair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeSemantics {
    /// Legacy comparison mode: repair consumes inventory but takes no clock time.
    Instant,
    /// Canonical mode: repair advances the clock by its scheduled duration.
    AdvanceRepair,
}

/// Monotonic event clock for one simulation run.
#[derive(Clone, Debug, PartialEq)]
pub struct EventClock {
    now_s: f64,
    horizon_s: f64,
}

impl EventClock {
    pub fn new(horizon_s: f64) -> EventClock {
        EventClock { now_s: 0.0, horizon_s: horizon_s.max(0.0) }
    }

    pub fn now_s(&self) -> f64 {
        self.now_s
    }

    pub fn horizon_s(&self) -> f64 {
        self.horizon_s
    }

    pub fn remaining_s(&self) -> f64 {
        (self.horizon_s - self.now_s).max(0.0)
    }

    /// Moves to `target_s`, clamped to the horizon. Returns the elapsed time.
    pub fn advance_to(&mut self, target_s: f64) -> f64 {
        let target = target_s.clamp(self.now_s, self.horizon_s);
        let elapsed = target - self.now_s;
        self.now_s = target;
        elapsed
    }

    /// Moves forward by `duration_s`, clamped to the horizon.
    pub fn advance_by(&mut self, duration_s: f64) -> f64 {
        self.advance_to(self.now_s + duration_s.max(0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_is_monotonic_and_clamped() {
        let mut clock = EventClock::new(10.0);
        assert_eq!(clock.advance_to(4.0), 4.0);
        assert_eq!(clock.advance_to(3.0), 0.0);
        assert_eq!(clock.advance_by(100.0), 6.0);
        assert_eq!(clock.now_s(), 10.0);
        assert_eq!(clock.remaining_s(), 0.0);
    }

    #[test]
    fn semantics_are_distinct() {
        assert_ne!(TimeSemantics::Instant, TimeSemantics::AdvanceRepair);
    }
}
