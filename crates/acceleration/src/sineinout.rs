use std::{
    ops::Mul,
    time::{Duration, Instant},
};

use directions::Stop;
use speed::Speed;

use crate::Accelerator;

/// Apply sine-in-out acceleration
#[derive(Debug, Copy, Clone)]
pub struct SineInOutAcceleration {
    /// The [`Duration`] it takes to accelerate from zero to full [`Speed`]
    duration: Duration,
    /// Timestamp when the acceleration started
    last: Option<Instant>,
}

impl SineInOutAcceleration {
    /// Create a new [`SineInOutAcceleration`]
    ///
    /// The [`Duration`] is the amount of time before the speed reaches full value.
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            last: None,
        }
    }

    /// Reset the [`SineInOutAcceleration`], removing any notion of start time.
    pub fn reset(&mut self) -> Option<Instant> {
        self.last.take()
    }

    /// The sine-in-out easing function.
    /// Input `value` is normalized between `[0,1]`, returns a value also in `[0,1]`.
    fn sine_in_out(value: Speed) -> Speed {
        use std::f64::consts::PI;
        let result = 0.5 * (1.0 - (PI * value.value()).cos());
        Speed::new_clamp(result)
    }
}

impl<S> Accelerator<S> for SineInOutAcceleration
where
    S: Mul<Speed, Output = S> + Stop + Sized,
{
    fn apply(&mut self, value: S) -> S {
        // If the value is at stop, reset the timer and return max (no acceleration needed)
        if value.is_stop() {
            self.last = None;
            return value * Speed::MAX;
        }

        // If this is the first time applying acceleration after not being stopped, record the start time
        if self.last.is_none() {
            self.last = Some(Instant::now());
        }

        // Calculate how far we are into the duration
        let elapsed = self.last.unwrap().elapsed();
        let fraction = elapsed.as_secs_f64() / self.duration.as_secs_f64();

        // Clamp fraction into [0, 1], because the easing function is defined in that range
        let speed = Speed::new_clamp(fraction);
        let eased = Self::sine_in_out(speed);

        // Apply the eased speed multiplier
        value * eased
    }
}
