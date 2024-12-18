//! Abstractions for different directions our hardware can move
//!
//! Primarily we implement [`MotorDirection`], [`SpinDirection`] and [`VehicleDirection`]

mod motor;
mod spin;
mod vehicle;

pub use motor::MotorDirection;
use speed::Speed;
pub use spin::SpinDirection;
pub use vehicle::VehicleDirection;

/// Trait for allowing types to change their [`Speed`]
pub trait SpeedControl {
    /// Get the ascossiated [`Speed`] of the type
    fn speed(&self) -> Speed;

    /// Change the ascossiated [`Speed`] of the type by consuming Self
    fn with_speed(self, speed: Speed) -> Self;
}

/// Generic trait for checking if a direction is a stop
pub trait Stop {
    /// Whether the value means stop
    fn is_stop(&self) -> bool;
}
