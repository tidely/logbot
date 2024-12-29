//! Define core abstractions which are completely generic

use std::ops::Not;

use speed::Speed;

/// Trait that defines a component as driveable
/// Provides a drive and a stop method
pub trait Drive {
    /// Direction type which is used by the [`Drive`] implementation
    type Direction;
    /// Error type
    type Error;

    /// Being driving the component into a given direction
    fn drive(&mut self, direction: Self::Direction)
        -> Result<Option<Self::Direction>, Self::Error>;

    /// Stop the components movement
    fn stop(&mut self) -> Result<Option<Self::Direction>, Self::Error>;
}

/// Trait that defines a driveable as being able to spin
pub trait Spin: Drive {
    /// The enum/struct used for indicating the spin direction
    type SpinDirection: Not;

    /// Spin the driveable in a given direction
    fn spin(
        &mut self,
        direction: Self::SpinDirection,
    ) -> Result<Option<Self::Direction>, Self::Error>;
}

/// Trait for defining a Lift that moves up or down
///
/// The Lift should have a way of reading it's current position to prevent the
/// lift from going out of bounds.
pub trait Lift {
    /// Error type
    type Error;

    /// Move the Lift up
    fn up(&mut self, speed: Speed) -> Result<(), Self::Error>;
    /// Move the Lift down
    fn down(&mut self, speed: Speed) -> Result<(), Self::Error>;

    /// Whether the Lift is in the up position
    fn is_up(&self) -> bool;
    /// Whether the Lift is in the down position
    fn is_down(&self) -> bool;
}

/// Get the Sensor channel for a given sensor
///
/// This trait returns a channel that is associated with a sensor
/// Often combined with the [`SensorRead`] trait
pub trait ToSensorChannel {
    /// Return a i2c channel for a given sensor
    fn to_channel(&self) -> u8;
}

/// Trait that allows reading a value from a sensor
pub trait SensorRead {
    /// The output of a sensor read operation
    type Output;
    /// The Error type of a failed sensor read
    type Error;

    /// Read a value from a sensor given a sensor channel
    fn read(&mut self, sensor: impl ToSensorChannel) -> Result<Self::Output, Self::Error>;
}
