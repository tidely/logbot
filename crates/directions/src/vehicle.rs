use std::ops::Mul;

use crate::{MotorDirection, SpeedControl, SpinDirection, Stop};
use speed::Speed;

/// Represents directions a vehicle can take
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VehicleDirection {
    /// The [`MotorDirection`] for the left motor
    pub left: MotorDirection,
    /// The [`MotorDirection`] for the right motor
    pub right: MotorDirection,
}

impl VehicleDirection {
    /// Create a new [`VehicleDirection`] from the individual [`MotorDirection`]s
    pub fn new(left: MotorDirection, right: MotorDirection) -> Self {
        Self { left, right }
    }
}

// Implement basic directions with a given [`Speed`]
impl VehicleDirection {
    /// [Forward](MotorDirection::Forward) direction with a given [`Speed`]
    pub fn forward(speed: Speed) -> Self {
        Self::new(
            MotorDirection::Forward(speed),
            MotorDirection::Forward(speed),
        )
    }

    /// [Backward](MotorDirection::Backward) direction with a given [`Speed`]
    pub fn backward(speed: Speed) -> Self {
        Self::new(
            MotorDirection::Backward(speed),
            MotorDirection::Backward(speed),
        )
    }

    /// Turn into a [`SpinDirection`] with a given [`Speed`]
    ///
    /// The [`Speed`] component of the [`SpinDirection`] is subtracted from the
    /// [`MotorDirection`] on the side on which the [`SpinDirection`] instructs
    pub fn turn(speed: Speed, direction: SpinDirection) -> Self {
        match direction {
            SpinDirection::Left(ratio) => Self::new(
                MotorDirection::Forward(speed).wrapping_sub_f64(2.0 * ratio.value()),
                MotorDirection::Forward(speed),
            ),
            SpinDirection::Right(ratio) => Self::new(
                MotorDirection::Forward(speed),
                MotorDirection::Forward(speed).wrapping_sub_f64(2.0 * ratio.value()),
            ),
        }
    }

    /// Spin the vehicle to the left in-place with a given [`Speed`]
    pub fn spin_left(speed: Speed) -> Self {
        Self::new(
            MotorDirection::Backward(speed),
            MotorDirection::Forward(speed),
        )
    }

    /// Spin the vehicle to the right in-place with a given [`Speed`]
    pub fn spin_right(speed: Speed) -> Self {
        Self::new(
            MotorDirection::Forward(speed),
            MotorDirection::Backward(speed),
        )
    }
}

impl Stop for VehicleDirection {
    fn is_stop(&self) -> bool {
        self.left.speed().value() == 0.0 && self.right.speed().value() == 0.0
    }
}

impl Mul<Speed> for VehicleDirection {
    type Output = Self;

    fn mul(self, rhs: Speed) -> Self::Output {
        Self::new(self.left * rhs, self.right * rhs)
    }
}

impl From<SpinDirection> for VehicleDirection {
    fn from(value: SpinDirection) -> Self {
        match value {
            SpinDirection::Left(speed) => VehicleDirection::new(
                MotorDirection::Backward(speed),
                MotorDirection::Forward(speed),
            ),
            SpinDirection::Right(speed) => VehicleDirection::new(
                MotorDirection::Forward(speed),
                MotorDirection::Backward(speed),
            ),
        }
    }
}
