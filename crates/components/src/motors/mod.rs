//! Useful abstractions for interacting with hardware and software pwm motor implementations
use std::time::Duration;

pub mod hardware_pwm;
pub mod software_pwm;

/// Indicate that a component is on the [`Left`] side
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Left;

/// Indicate that a component is on the [`Right`] side
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Right;

/// PWM Configuration that's used by both hardware and software PWM
#[derive(Debug, Clone, Copy)]
pub struct PwmConfig {
    /// Duration of a pwm period
    pub period: Duration,
    /// The pulse width for the stop signal
    pub stop_pulse_width: Duration,
    /// The range of the pulse width in one direction
    pub pulse_width_range: Duration,
}

impl Default for PwmConfig {
    fn default() -> Self {
        Self {
            period: Duration::from_millis(20),
            stop_pulse_width: Duration::from_micros(1500),
            pulse_width_range: Duration::from_micros(500),
        }
    }
}
