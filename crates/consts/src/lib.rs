//! Provide constants only used by the current hardware implementations
//! of the project.

use interfaces::ToSensorChannel;

/// Address of the I2C bus used for sensors
pub const I2C_SENSOR_ADDRESS: u16 = 0x48;

/// Default PWM frequency recommended for a SignedMotor
pub const FREQUENCY: f64 = 4096.0;

/// Collection of hardware pins
pub mod pins {
    /// Right Motor power pin
    pub const RIGHT_MOTOR_POWER: u8 = 12;
    /// Right Motor direction pin
    pub const RIGHT_MOTOR_DIRECTION: u8 = 5;

    /// Left Motor power pin
    pub const LEFT_MOTOR_POWER: u8 = 13;
    /// Left Motor direction pin
    pub const LEFT_MOTOR_DIRECTION: u8 = 6;

    /// Lift Motor power pin
    pub const LIFT_MOTOR_POWER: u8 = 23;
    /// Lift Motor direction pin
    pub const LIFT_MOTOR_DIRECTION: u8 = 24;
    /// Lift Motor Up State
    pub const LIFT_UP: u8 = 27;
    /// Lift Motor Down State
    pub const LIFT_DOWN: u8 = 22;
}

/// An enum of all available sensors
///
/// Lists all available sensors as an enum. [`Sensors`] implements
/// [`ToSensorChannel`], which returns the I2c channel for the given sensor
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sensors {
    /// Left sensor
    Left,
    /// Right sensor
    Right,
}

impl ToSensorChannel for Sensors {
    fn to_channel(&self) -> u8 {
        match self {
            Self::Left => 0,
            Self::Right => 1,
        }
    }
}
