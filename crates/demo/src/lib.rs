//! logbot demo of following a line and lifting boxes

// https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use std::{num::NonZero, time::Duration};

use acceleration::{Accelerate, LinearAcceleration};
use calibration::{SensorCalibration, SingleSensorCalibration};
use consts::Sensors;
use directions::{SpinDirection, VehicleDirection};
use error::{DemoError, VehicleSensorError};
use interfaces::{Lift, SensorRead, Spin};
use line::{FollowLineConfig, FollowLineState};
use oscillate::Oscillate;
use speed::Speed;

/// Demo Error types
pub mod error;

/// Calibrate logbot
fn calibrate<Vehicle, SensorReader>(
    vehicle: &mut Vehicle,
    sensors: &mut SensorReader,
) -> Result<
    (SensorCalibration, SensorCalibration),
    VehicleSensorError<Vehicle::Error, SensorReader::Error>,
>
where
    Vehicle: Spin<SpinDirection = SpinDirection>,
    SensorReader: SensorRead<Output = u8>,
{
    let mut left_calibration = SingleSensorCalibration::default();
    let mut right_calibration = SingleSensorCalibration::default();

    // Configure and start oscillation
    let mut oscillate = Oscillate::new(
        Duration::from_millis(500),
        SpinDirection::Left(Speed::new_clamp(0.08)),
        NonZero::<u32>::new(2).unwrap(),
    )
    .start(vehicle)
    .map_err(VehicleSensorError::Vehicle)?;

    // Wait for the first change in direction until we start logging values
    oscillate.wait_until_next();
    oscillate
        .step(vehicle)
        .map_err(VehicleSensorError::Vehicle)?;

    // Read sensor values continuously until we're supposed to oscillate again
    while !oscillate.should_step() {
        let left_value = sensors
            .read(Sensors::Left)
            .map_err(VehicleSensorError::Sensor)?;
        let right_value = sensors
            .read(Sensors::Left)
            .map_err(VehicleSensorError::Sensor)?;

        left_calibration.log(left_value as f64);
        right_calibration.log(right_value as f64);
    }

    vehicle.stop().map_err(VehicleSensorError::Vehicle)?;

    // Evaluate sensor readings
    Ok((left_calibration.calibrate(), right_calibration.calibrate()))
}

/// Find the edge of the line
fn find_edge<Vehicle, SensorReader>(
    vehicle: &mut Vehicle,
    sensors: &mut SensorReader,
    calibration: &SensorCalibration,
    direction: SpinDirection,
) -> Result<(), VehicleSensorError<Vehicle::Error, SensorReader::Error>>
where
    Vehicle: Spin<SpinDirection = SpinDirection>,
    SensorReader: SensorRead<Output = u8>,
{
    vehicle
        .spin(direction)
        .map_err(VehicleSensorError::Vehicle)?;

    while sensors
        .read(Sensors::Right)
        .map_err(VehicleSensorError::Sensor)?
        < calibration.line.saturating_sub(1)
    {
        std::thread::sleep(Duration::from_micros(300));
    }

    // Stop logbot after edge is found
    vehicle.stop().map_err(VehicleSensorError::Vehicle)?;
    Ok(())
}

/// Spin logbot in-place from the line, until it finds the line again
///
/// Basically means making a 180 degree turn in most cases
fn turn_on_line<Vehicle, SensorReader>(
    vehicle: &mut Vehicle,
    sensors: &mut SensorReader,
    left_calibration: &SensorCalibration,
    direction: SpinDirection,
) -> Result<(), VehicleSensorError<Vehicle::Error, SensorReader::Error>>
where
    Vehicle: Spin<SpinDirection = SpinDirection, Direction = VehicleDirection>,
    SensorReader: SensorRead<Output = u8>,
{
    // Start spinning in a direction
    vehicle
        .spin(direction)
        .map_err(VehicleSensorError::Vehicle)?;

    // Give a little time of get off the line first
    std::thread::sleep(Duration::from_secs(1));

    // Wait until we find the line again
    while sensors
        .read(Sensors::Left)
        .map_err(VehicleSensorError::Sensor)?
        < left_calibration.line.saturating_sub(3)
    {
        std::thread::sleep(Duration::from_micros(300));
    }

    // Stop the vehicle once we are back on the line
    vehicle.stop().map_err(VehicleSensorError::Vehicle)?;

    Ok(())
}

/// Follow line until a stop line is detected
///
/// A stop line means that both sensors consider themselves ontop of the line at the same time
fn follow_until_line<Vehicle, SensorReader>(
    vehicle: &mut Vehicle,
    sensors: &mut SensorReader,
    left_calibration: &SensorCalibration,
    right_calibration: &SensorCalibration,
    config: FollowLineConfig,
) -> Result<(), VehicleSensorError<Vehicle::Error, SensorReader::Error>>
where
    Vehicle: Spin<SpinDirection = SpinDirection, Direction = VehicleDirection>,
    SensorReader: SensorRead<Output = u8>,
{
    // Create a new state from the config
    let mut state = FollowLineState::new(config.clone());

    let mut acceleration = LinearAcceleration::new(Duration::from_secs(2));

    let stop_left = left_calibration.line.saturating_sub(1);
    let stop_right = right_calibration.line.saturating_sub(1);

    loop {
        let left_sensor_value = sensors
            .read(Sensors::Left)
            .map_err(VehicleSensorError::Sensor)?;
        let right_sensor_value = sensors
            .read(Sensors::Right)
            .map_err(VehicleSensorError::Sensor)?;

        if left_sensor_value > stop_left && right_sensor_value > stop_right {
            break;
        };

        let direction = state.step(left_sensor_value);
        let direction = direction.accelerate(&mut acceleration);
        vehicle
            .drive(direction)
            .map_err(VehicleSensorError::Vehicle)?;
    }

    vehicle.stop().map_err(VehicleSensorError::Vehicle)?;
    Ok(())
}

/// Demo logbot, by following the line and lifting boxes in an pre-arranged setup
pub fn demo<Vehicle, SensorReader, LiftMotor>(
    vehicle: &mut Vehicle,
    sensors: &mut SensorReader,
    lift: &mut LiftMotor,
) -> Result<(), DemoError<Vehicle::Error, SensorReader::Error, LiftMotor::Error>>
where
    Vehicle: Spin<SpinDirection = SpinDirection, Direction = VehicleDirection>,
    SensorReader: SensorRead<Output = u8>,
    LiftMotor: Lift,
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

    lift.up(Speed::HALF).map_err(DemoError::Lift)?;

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

    lift.down(Speed::HALF).map_err(DemoError::Lift)?;

    Ok(())
}
