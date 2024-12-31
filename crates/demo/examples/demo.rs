//! Demo example

// https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use components::{hardware_pwm::DCMotor, software_pwm::LiftMotor, Left, Right, SensorController};
use defaults::TryDefault;
use demo::demo;
use logbot::Logbot;
use vehicle::Vehicle;

/// Run demo as an example
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let vehicle: Vehicle<DCMotor<Left>, DCMotor<Right>> = Vehicle::try_default()?;

    let mut logbot = Logbot::new(
        vehicle,
        SensorController::try_default()?,
        LiftMotor::try_default()?,
    );

    demo(&mut logbot)?;

    Ok(())
}
