use std::{
    ops::Mul,
    time::{Duration, Instant},
};

use directions::Stop;
use speed::Speed;

use crate::Accelerator;

/// Apply linear acceleration using [`Decelerate`]
#[derive(Debug, Copy, Clone)]
pub struct LinearAcceleration {
    /// The [`Duration`] it takes to accelerate to full [`Speed`]
    duration: Duration,
    /// Last acceleration start
    last: Option<Instant>,
}

impl LinearAcceleration {
    /// Create a new [`LinearAcceleration`]
    ///
    /// The [`Duration`] is the amount of time before the speed reaches it's
    /// full value
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            last: None,
        }
    }

    /// Reset the [`LinearAcceleration`]
    pub fn reset(&mut self) -> Option<Instant> {
        self.last.take()
    }
}

impl<S> Accelerator<S> for LinearAcceleration
where
    S: Mul<Speed, Output = S> + Stop + Sized,
{
    fn apply(&mut self, value: S) -> S {
        let multi = if value.is_stop() {
            self.last = None;
            Speed::MAX
        } else {
            if self.last.is_none() {
                self.last = Some(Instant::now());
            };
            let since_last = self.last.unwrap().elapsed().as_millis() as f64;
            let multiplier = since_last / self.duration.as_millis() as f64;
            Speed::new_clamp(multiplier)
        };

        value * multi
    }
}
