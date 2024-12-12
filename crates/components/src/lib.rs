//! Provide abstractions for hardware
//!
//! Provides abstractions for individual hardware components
//! Uses the [`rppal`] library for interfacing with hardware.
//! Often only the current state is saved in addition to the
//! required data for interfacing with them.

mod motors;
mod sensor;

pub use motors::hardware_pwm;
pub use motors::software_pwm;
pub use motors::{Left, PwmConfig, Right};

pub use sensor::SensorController;
