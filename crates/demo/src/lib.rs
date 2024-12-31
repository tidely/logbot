//! logbot demo of following a line and lifting boxes

// https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use std::{num::NonZero, time::Duration};

use acceleration::{Accelerate, LinearAcceleration};
use calibration::{SensorCalibration, SingleSensorCalibration};
use consts::Sensors;
use directions::{SpinDirection, VehicleDirection};
use interfaces::{Drive, Lift, SensorRead, Spin};
use line::{FollowLineConfig, FollowLineState};
use logbot::error::LogbotError;
use oscillate::Oscillate;
use speed::Speed;

// Result of a calibration
type Calibration = (SensorCalibration, SensorCalibration);

/// Calibrate logbot
fn calibrate<L, LiftError>(
    logbot: &mut L,
) -> Result<Calibration, LogbotError<<L as Drive>::Error, <L as SensorRead>::Error, LiftError>>
where
    L: Spin<SpinDirection = SpinDirection>,
    L: SensorRead<Output = u8>,
{
    let mut left_calibration = SingleSensorCalibration::default();
    let mut right_calibration = SingleSensorCalibration::default();

    // Configure and start oscillation
    let mut oscillate = Oscillate::new(
        Duration::from_millis(500),
        SpinDirection::Left(Speed::new_clamp(0.08)),
        NonZero::<u32>::new(2).unwrap(),
    )
    .start(logbot)
    .map_err(LogbotError::Vehicle)?;

    // Wait for the first change in direction until we start logging values
    oscillate.wait_until_next();
    oscillate.step(logbot).map_err(LogbotError::Vehicle)?;

    // Read sensor values continuously until we're supposed to oscillate again
    while !oscillate.should_step() {
        let left_value = logbot.read(Sensors::Left).map_err(LogbotError::Sensor)?;
        let right_value = logbot.read(Sensors::Left).map_err(LogbotError::Sensor)?;

        left_calibration.log(left_value as f64);
        right_calibration.log(right_value as f64);
    }

    logbot.stop().map_err(LogbotError::Vehicle)?;

    // Evaluate sensor readings
    Ok((left_calibration.calibrate(), right_calibration.calibrate()))
}

/// Find the edge of the line
fn find_edge<L, LiftError>(
    logbot: &mut L,
    calibration: &SensorCalibration,
    direction: SpinDirection,
) -> Result<(), LogbotError<<L as Drive>::Error, <L as SensorRead>::Error, LiftError>>
where
    L: Spin<SpinDirection = SpinDirection>,
    L: SensorRead<Output = u8>,
{
    logbot.spin(direction).map_err(LogbotError::Vehicle)?;

    while logbot.read(Sensors::Right).map_err(LogbotError::Sensor)?
        < calibration.line.saturating_sub(1)
    {
        std::thread::sleep(Duration::from_micros(300));
    }

    // Stop logbot after edge is found
    logbot.stop().map_err(LogbotError::Vehicle)?;
    Ok(())
}

/// Spin logbot in-place from the line, until it finds the line again
///
/// Basically means making a 180 degree turn in most cases
fn turn_on_line<L, LiftError>(
    logbot: &mut L,
    left_calibration: &SensorCalibration,
    direction: SpinDirection,
) -> Result<(), LogbotError<<L as Drive>::Error, <L as SensorRead>::Error, LiftError>>
where
    L: Spin<SpinDirection = SpinDirection, Direction = VehicleDirection>,
    L: SensorRead<Output = u8>,
{
    // Start spinning in a direction
    logbot.spin(direction).map_err(LogbotError::Vehicle)?;

    // Give a little time of get off the line first
    std::thread::sleep(Duration::from_secs(1));

    // Wait until we find the line again
    while logbot.read(Sensors::Left).map_err(LogbotError::Sensor)?
        < left_calibration.line.saturating_sub(3)
    {
        std::thread::sleep(Duration::from_micros(300));
    }

    // Stop the vehicle once we are back on the line
    logbot.stop().map_err(LogbotError::Vehicle)?;

    Ok(())
}

/// Follow line until a stop line is detected
///
/// A stop line means that both sensors consider themselves ontop of the line at the same time
fn follow_until_line<L, LiftError>(
    logbot: &mut L,
    left_calibration: &SensorCalibration,
    right_calibration: &SensorCalibration,
    config: FollowLineConfig,
) -> Result<(), LogbotError<<L as Drive>::Error, <L as SensorRead>::Error, LiftError>>
where
    L: Spin<SpinDirection = SpinDirection, Direction = VehicleDirection>,
    L: SensorRead<Output = u8>,
{
    // Create a new state from the config
    let mut state = FollowLineState::new(config.clone());

    let mut acceleration = LinearAcceleration::new(Duration::from_secs(2));

    let stop_left = left_calibration.line.saturating_sub(1);
    let stop_right = right_calibration.line.saturating_sub(1);

    loop {
        let left_sensor_value = logbot.read(Sensors::Left).map_err(LogbotError::Sensor)?;
        let right_sensor_value = logbot.read(Sensors::Right).map_err(LogbotError::Sensor)?;

        if left_sensor_value > stop_left && right_sensor_value > stop_right {
            break;
        };

        let direction = state.step(left_sensor_value);
        let direction = direction.accelerate(&mut acceleration);
        logbot.drive(direction).map_err(LogbotError::Vehicle)?;
    }

    logbot.stop().map_err(LogbotError::Vehicle)?;
    Ok(())
}

/// Demo logbot, by following the line and lifting boxes in an pre-arranged setup
pub fn demo<L>(
    logbot: &mut L,
) -> Result<(), LogbotError<<L as Drive>::Error, <L as SensorRead>::Error, <L as Lift>::Error>>
where
    L: Spin<SpinDirection = SpinDirection, Direction = VehicleDirection>,
    L: SensorRead<Output = u8>,
    L: Lift,
{
    let (left_calibration, right_calibration) = calibrate(logbot)?;

    find_edge(
        logbot,
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
    follow_until_line(logbot, &left_calibration, &right_calibration, config)?;

    logbot.up(Speed::HALF).map_err(LogbotError::Lift)?;

    // Turn the logbot 180 degrees in relation to the line
    turn_on_line(
        logbot,
        &left_calibration,
        SpinDirection::Right(Speed::new_clamp(0.08)),
    )?;

    std::thread::sleep(Duration::from_millis(200));

    find_edge(
        logbot,
        &right_calibration,
        SpinDirection::Left(Speed::new_clamp(0.1)),
    )?;

    std::thread::sleep(Duration::from_millis(200));

    follow_until_line(logbot, &left_calibration, &right_calibration, config)?;

    logbot.down(Speed::HALF).map_err(LogbotError::Lift)?;

    Ok(())
}
