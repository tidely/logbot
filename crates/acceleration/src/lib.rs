//! Crate for adding linear acceleration to a Speed

use std::{
    ops::Mul,
    time::{Duration, Instant},
};

use directions::Stop;
use speed::Speed;

/// Trait for decelerating
pub trait Accelerate {
    /// Decelerate using a concrete implementation
    fn accelerate(self, acceleration: &mut Acceleration) -> Self;
}

impl<T> Accelerate for T
where
    T: Mul<Speed, Output = T> + Stop,
{
    fn accelerate(self, acceleration: &mut Acceleration) -> Self {
        if self.is_stop() {
            acceleration.last = None;
            self
        } else {
            if acceleration.last.is_none() {
                acceleration.last = Some(Instant::now());
            };
            let since_last = acceleration.last.unwrap().elapsed().as_millis() as f64;
            let multiplier = since_last / acceleration.duration.as_millis() as f64;

            self * Speed::new_clamp(multiplier)
        }
    }
}

/// Apply linear acceleration using [`Decelerate`]
#[derive(Debug, Copy, Clone)]
pub struct Acceleration {
    /// The [`Duration`] it takes to accelerate to full [`Speed`]
    duration: Duration,
    /// Last acceleration start
    last: Option<Instant>,
}

impl Acceleration {
    /// Create a new [`Acceleration`]
    ///
    /// The [`Duration`] is the amount of time before the speed reaches it's
    /// full value
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            last: None,
        }
    }
}
