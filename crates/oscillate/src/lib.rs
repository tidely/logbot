//! Oscillate a type with the [`Spin`] trait

use std::{
    num::NonZero,
    ops::Not,
    time::{Duration, Instant},
};

use directions::SpinDirection;
use interfaces::Spin;

/// Store the state of an oscillation
///
/// This struct should be called with step to advance the state
#[derive(Debug, Clone, Copy)]
pub struct Oscillate {
    duration: Duration,
    direction: SpinDirection,
    multiplier: NonZero<u32>,
}

impl Oscillate {
    /// Create a [`Oscillate`] with given settings
    pub fn new(duration: Duration, direction: SpinDirection, multiplier: NonZero<u32>) -> Self {
        Self {
            duration,
            direction,
            multiplier,
        }
    }

    /// Turn the [`Oscillate`] active by starting to spin
    pub fn start<D>(self, driveable: &mut D) -> Result<ActiveOscillation, D::Error>
    where
        D: Spin<SpinDirection = SpinDirection>,
    {
        driveable.spin(self.direction)?;
        Ok(ActiveOscillation {
            config: self,
            since_last: Instant::now(),
        })
    }
}

/// State of an active oscillation
#[derive(Debug, Clone, Copy)]
pub struct ActiveOscillation {
    config: Oscillate,
    since_last: Instant,
}

impl ActiveOscillation {
    /// Move ahead with the oscillation if enough time has passed
    ///
    /// Returns whether or not the step made the oscillation change directions
    pub fn step<D>(&mut self, driveable: &mut D) -> Result<bool, D::Error>
    where
        D: Spin<SpinDirection = SpinDirection>,
    {
        if self.since_last.elapsed() > self.config.duration {
            // Switch pin direction and double the duration
            self.config.direction = self.config.direction.not();
            self.config.duration *= self.config.multiplier.get();
            self.since_last = Instant::now();
            driveable.spin(self.config.direction)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Boolean indicating whether [step](Self::step) is ready to be called
    pub fn should_step(&self) -> bool {
        self.next_oscillation().is_zero()
    }

    /// [`Duration`] until the next oscillation should occur
    pub fn next_oscillation(&self) -> Duration {
        // Don't allow negative durations
        self.config
            .duration
            .saturating_sub(self.since_last.elapsed())
    }

    /// Wait until the next oscillation should occur
    ///
    /// The caller still needs to call [step](Self::step) manually,
    ///
    /// returns the [`Duration`] which the thread waited
    pub fn wait_until_next(&self) -> Duration {
        let amount = self.next_oscillation();
        std::thread::sleep(amount);
        amount
    }
}
