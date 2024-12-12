//! Demo example

// https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use std::time::Duration;

use components::{hardware_pwm::DCMotor, software_pwm::LiftMotor, Left, Right, SensorController};
use defaults::TryDefault;
use demo::demo;
use vehicle::Vehicle;

/// Run demo as an example
fn main() -> Result<(), Box<dyn core::error::Error>> {
    let left = DCMotor::<Left>::try_default()?;
    let right = DCMotor::<Right>::try_default()?;
    std::thread::sleep(Duration::from_secs(5));

    let mut vehicle = Vehicle::new(left, right);
    let mut sensors = SensorController::try_default()?;

    let mut lift = LiftMotor::try_default()?;

    demo(&mut vehicle, &mut sensors, &mut lift)?;

    Ok(())
}
