//! Abstractions for different directions our hardware can move
//!
//! Primarily we implement [`MotorDirection`], [`SpinDirection`] and [`VehicleDirection`]

mod motor;
mod spin;
mod vehicle;

pub use motor::MotorDirection;
pub use spin::SpinDirection;
pub use vehicle::VehicleDirection;

/// Generic trait for checking if a direction is a stop
pub trait Stop {
    /// Whether the value means stop
    fn is_stop(&self) -> bool;
}
