//! logbot demo of following a line and lifting boxes

// https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use std::{num::NonZero, time::Duration};

use acceleration::{Accelerate, LinearAcceleration};
use calibration::{SensorCalibration, SingleSensorCalibration};
use components::{software_pwm::LiftMotor, SensorController};
use consts::Sensors;
use directions::{SpinDirection, VehicleDirection};
use interfaces::{Lift, SensorRead, Spin};
use line::{FollowLineConfig, FollowLineState};
use oscillate::Oscillate;
use speed::Speed;

/// Calibrate logbot
fn calibrate<Vehicle>(
    vehicle: &mut Vehicle,
    sensors: &mut SensorController,
) -> Result<(SensorCalibration, SensorCalibration), Box<dyn core::error::Error>>
where
    Vehicle: Spin<SpinDirection = SpinDirection>,
    Vehicle::Error: core::error::Error + 'static,
{
    let mut left_calibration = SingleSensorCalibration::default();
    let mut right_calibration = SingleSensorCalibration::default();

    // Configure and start oscillation
    let mut oscillate = Oscillate::new(
        Duration::from_millis(500),
        SpinDirection::Left(Speed::new_clamp(0.08)),
        NonZero::<u32>::new(2).unwrap(),
    )
    .start(vehicle)?;

    // Wait for the first change in direction until we start logging values
    oscillate.wait_until_next();
    oscillate.step(vehicle)?;

    // Read sensor values continuously until we're supposed to oscillate again
    while !oscillate.should_step() {
        let left_value = sensors.read(Sensors::Left)?;
        let right_value = sensors.read(Sensors::Left)?;

        left_calibration.log(left_value as f64);
        right_calibration.log(right_value as f64);
    }

    vehicle.stop()?;

    // Evaluate sensor readings
    Ok((left_calibration.calibrate(), right_calibration.calibrate()))
}

/// Find the edge of the line
fn find_edge<Vehicle>(
    vehicle: &mut Vehicle,
    sensors: &mut SensorController,
    calibration: &SensorCalibration,
    direction: SpinDirection,
) -> Result<(), Box<dyn core::error::Error>>
where
    Vehicle: Spin<SpinDirection = SpinDirection>,
    Vehicle::Error: core::error::Error + 'static,
{
    vehicle.spin(direction)?;

    while sensors.read(Sensors::Right)? < calibration.line.saturating_sub(1) {
        std::thread::sleep(Duration::from_micros(300));
    }

    // Stop logbot after edge is found
    vehicle.stop()?;
    Ok(())
}

/// Spin logbot in-place from the line, until it finds the line again
///
/// Basically means making a 180 degree turn in most cases
pub fn turn_on_line<Vehicle>(
    vehicle: &mut Vehicle,
    sensors: &mut SensorController,
    left_calibration: &SensorCalibration,
    direction: SpinDirection,
) -> Result<(), Box<dyn core::error::Error>>
where
    Vehicle: Spin<SpinDirection = SpinDirection, Direction = VehicleDirection>,
    Vehicle::Error: core::error::Error + 'static,
{
    // Start spinning in a direction
    vehicle.spin(direction)?;

    // Give a little time of get off the line first
    std::thread::sleep(Duration::from_secs(1));

    // Wait until we find the line again
    while sensors.read(Sensors::Left)? < left_calibration.line.saturating_sub(3) {
        std::thread::sleep(Duration::from_micros(300));
    }

    // Stop the vehicle once we are back on the line
    vehicle.stop()?;

    Ok(())
}

/// Follow line until a stop line is detected
///
/// A stop line means that both sensors consider themselves ontop of the line at the same time
pub fn follow_until_line<Vehicle>(
    vehicle: &mut Vehicle,
    sensors: &mut SensorController,
    left_calibration: &SensorCalibration,
    right_calibration: &SensorCalibration,
    config: FollowLineConfig,
) -> Result<(), Box<dyn core::error::Error>>
where
    Vehicle: Spin<SpinDirection = SpinDirection, Direction = VehicleDirection>,
    Vehicle::Error: core::error::Error + 'static,
{
    // Create a new state from the config
    let mut state = FollowLineState::new(config.clone());

    let mut acceleration = LinearAcceleration::new(Duration::from_secs(2));

    let stop_left = left_calibration.line.saturating_sub(1);
    let stop_right = right_calibration.line.saturating_sub(1);

    loop {
        let left_sensor_value = sensors.read(Sensors::Left)?;
        let right_sensor_value = sensors.read(Sensors::Right)?;

        if left_sensor_value > stop_left && right_sensor_value > stop_right {
            break;
        };

        let direction = state.step(left_sensor_value);
        let direction = direction.accelerate(&mut acceleration);
        vehicle.drive(direction)?;
    }

    vehicle.stop()?;
    Ok(())
}

/// Demo logbot, by following the line and lifting boxes in an pre-arranged setup
pub fn demo<Vehicle>(
    vehicle: &mut Vehicle,
    sensors: &mut SensorController,
    lift: &mut LiftMotor,
) -> Result<(), Box<dyn core::error::Error>>
where
    Vehicle: Spin<SpinDirection = SpinDirection, Direction = VehicleDirection>,
    Vehicle::Error: core::error::Error + 'static,
{
    let (left_calibration, right_calibration) = calibrate(vehicle, sensors)?;

    find_edge(
        vehicle,
        sensors,
        &right_calibration,
        SpinDirection::Left(Speed::new_clamp(0.1)),
    )?;

    std::thread::sleep(Duration::from_millis(200));

    // Create the config for following the line
    let config = FollowLineConfig {
        default_speed: Speed::new_clamp(0.1),
        proportional: 0.001,
        derivative: 0.0005,
        integral: None,
        calibration: left_calibration,
        reset_integral_on_target: true,
    };

    // Follow line until the first stopline
    follow_until_line(
        vehicle,
        sensors,
        &left_calibration,
        &right_calibration,
        config,
    )?;

    lift.up(Speed::HALF)?;

    // Turn the logbot 180 degrees in relation to the line
    turn_on_line(
        vehicle,
        sensors,
        &left_calibration,
        SpinDirection::Right(Speed::new_clamp(0.08)),
    )?;

    std::thread::sleep(Duration::from_millis(200));

    find_edge(
        vehicle,
        sensors,
        &right_calibration,
        SpinDirection::Left(Speed::new_clamp(0.1)),
    )?;

    std::thread::sleep(Duration::from_millis(200));

    follow_until_line(
        vehicle,
        sensors,
        &left_calibration,
        &right_calibration,
        config,
    )?;

    lift.down(Speed::HALF)?;

    Ok(())
}
