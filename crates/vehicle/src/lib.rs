//! Vehicle abstraction for a two wheeled vehicle

use directions::{MotorDirection, SpinDirection, VehicleDirection};
use interfaces::{Drive, Spin};

mod error;
pub use error::VehicleError;

/// Describes a dual motored Vehicle
#[derive(Debug, Clone, Copy)]
pub struct Vehicle<LD, RD>
where
    LD: Drive,
    RD: Drive,
{
    /// Left hardware component that implements [`Drive`]
    left: LD,
    /// Right hardware component that implements [`Drive`]
    right: RD,
    /// The current [`VehicleDirection`]
    state: Option<VehicleDirection>,
}

impl<LD, RD> Drive for Vehicle<LD, RD>
where
    LD: Drive<Direction = MotorDirection>,
    RD: Drive<Direction = MotorDirection>,
{
    type Direction = VehicleDirection;
    type Error = VehicleError<LD::Error, RD::Error>;

    /// [`Drive`] the [`Vehicle`] in a given [`VehicleDirection`].
    /// This instructs the left and right driveables to move into their
    /// corresponding [`MotorDirection`]'s
    fn drive(
        &mut self,
        direction: Self::Direction,
    ) -> Result<Option<Self::Direction>, Self::Error> {
        self.left
            .drive(direction.left)
            .map_err(|e| VehicleError::Left(e))?;
        self.right
            .drive(direction.right)
            .map_err(|e| VehicleError::Right(e))?;
        Ok(self.state.replace(direction))
    }

    /// Stop the [`Vehicle`] by stopping the underlying driveables
    fn stop(&mut self) -> Result<Option<Self::Direction>, Self::Error> {
        self.left.stop().map_err(|e| VehicleError::Left(e))?;
        self.right.stop().map_err(|e| VehicleError::Right(e))?;
        Ok(self.state.take())
    }
}

impl<LD, RD> Vehicle<LD, RD>
where
    LD: Drive,
    RD: Drive,
{
    /// Create a new [`Vehicle`]
    pub fn new(left: LD, right: RD) -> Self {
        Self {
            left,
            right,
            state: Default::default(),
        }
    }

    /// Get the current state of the [`Vehicle`]
    pub fn state(&self) -> Option<VehicleDirection> {
        self.state
    }
}

impl<LD, RD> Spin for Vehicle<LD, RD>
where
    LD: Drive<Direction = MotorDirection>,
    RD: Drive<Direction = MotorDirection>,
{
    type SpinDirection = SpinDirection;

    /// [`Spin`] the [`Vehicle`] in-place into a given [`SpinDirection`]
    fn spin(
        &mut self,
        direction: SpinDirection,
    ) -> Result<Option<VehicleDirection>, VehicleError<LD::Error, RD::Error>> {
        let vehicle_direction = VehicleDirection::from(direction);
        self.drive(vehicle_direction)
    }
}
