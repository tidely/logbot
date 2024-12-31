//! The logbot crate provides a wrapper struct for all hardware components,
//! which then exports interfaces as a single struct. This allows for easy
//! trait bounds checking.

use interfaces::{Drive, Lift, SensorRead, Spin, ToSensorChannel};
use speed::Speed;

pub mod error;

/// Logbot struct that wraps all hardware components
#[derive(Debug)]
pub struct Logbot<V, S, L> {
    vehicle: V,
    sensors: S,
    lift: L,
}

impl<V, S, L> Logbot<V, S, L> {
    /// Create a new Logbot struct
    pub fn new(vehicle: V, sensors: S, lift: L) -> Self {
        Self {
            vehicle,
            sensors,
            lift,
        }
    }
}

// Export Drive Trait for Logbot
impl<V, S, L> Drive for Logbot<V, S, L>
where
    V: Drive,
{
    type Direction = V::Direction;
    type Error = V::Error;

    fn drive(
        &mut self,
        direction: Self::Direction,
    ) -> Result<Option<Self::Direction>, Self::Error> {
        self.vehicle.drive(direction)
    }

    fn stop(&mut self) -> Result<Option<Self::Direction>, Self::Error> {
        self.vehicle.stop()
    }
}

// Export Spin Trait for Logbot
impl<V, S, L> Spin for Logbot<V, S, L>
where
    V: Spin,
{
    type SpinDirection = V::SpinDirection;

    fn spin(
        &mut self,
        direction: Self::SpinDirection,
    ) -> Result<Option<Self::Direction>, Self::Error> {
        self.vehicle.spin(direction)
    }
}

// Export SensorRead Trait for Logbot
impl<V, S, L> SensorRead for Logbot<V, S, L>
where
    S: SensorRead,
{
    type Output = S::Output;
    type Error = S::Error;

    fn read(&mut self, sensor: impl ToSensorChannel) -> Result<Self::Output, Self::Error> {
        self.sensors.read(sensor)
    }
}

// Export Lift Trait for Logbot
impl<V, S, L> Lift for Logbot<V, S, L>
where
    L: Lift,
{
    type Error = L::Error;

    fn up(&mut self, speed: Speed) -> Result<(), Self::Error> {
        self.lift.up(speed)
    }

    fn down(&mut self, speed: Speed) -> Result<(), Self::Error> {
        self.lift.down(speed)
    }

    fn is_up(&self) -> bool {
        self.lift.is_up()
    }

    fn is_down(&self) -> bool {
        self.lift.is_down()
    }
}
