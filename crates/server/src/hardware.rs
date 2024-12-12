//! Actor thread for handling hardware operations

use std::{num::NonZero, time::Duration};

use acceleration::{Accelerate, Acceleration};
use anyhow::Result;

use calibration::{SensorCalibration, SingleSensorCalibration};
use components::{hardware_pwm::DCMotor, software_pwm::LiftMotor, Left, Right, SensorController};
use consts::Sensors;
use defaults::TryDefault;
use demo::demo;
use directions::SpinDirection;
use interfaces::{Drive, SensorRead};
use line::{FollowLineConfig, FollowLineState};
use oscillate::Oscillate;
use speed::Speed;
use tokio::sync::{mpsc, oneshot};
use vehicle::Vehicle;

const DEFAULT_SPEED: Speed = unsafe { Speed::new_unchecked(0.1) };

/// Command that contains a channel to send a response into
#[derive(Debug)]
pub enum Command {
    /// Health check
    Health(oneshot::Sender<()>),
    FollowLine(oneshot::Sender<FollowLineResult>),
    Calibrate(oneshot::Sender<CalibrateResult>),
    FindEdge(oneshot::Sender<FindEdgeResult>),
    Stop(oneshot::Sender<StopResult>),
    // Runs the full demo, also prevents health checks from going through
    // conveniently turning off the control buttons
    Demo(oneshot::Sender<DemoResult>),
}

/// Result of a [`Command::Demo`] call
#[derive(Debug, Clone, Copy)]
pub enum DemoResult {
    Success,
}

/// Result of a [`Command::FindEdge`] call
#[derive(Debug, Clone, Copy)]
pub enum FindEdgeResult {
    Success,
    AlreadyEdging,
    Follow,
    Calibrate,
    NoCalibration,
}

/// Result of a [`Command::Calibrate`] call
#[derive(Debug, Clone, Copy)]
pub enum CalibrateResult {
    Success,
    AlreadyCalibrating,
    FindingEdge,
    Following,
}

/// Result of a [`Command::Stop`] call, these represent which action was stopped
#[derive(Debug, Clone, Copy)]
pub enum StopResult {
    Nothing,
    FollowLine,
    FindEdge,
    Calibrate,
}

/// Result of a [`Command::FollowLine`] call
///
/// Success and different failure reasons are represented
#[derive(Debug, Clone, Copy)]
pub enum FollowLineResult {
    Success,
    AlreadyFollowing,
    NoCalibration,
    FindingEdge,
    MidCalibration,
    NotOnTheLine,
}

/// Process hardware requests syncronously
fn handle_commands(
    mut hardware: Hardware,
    mut channel: mpsc::Receiver<Command>,
) -> Result<(), Box<dyn core::error::Error>> {
    // Store the current calibration status
    let mut left_calibration: Option<SensorCalibration> = None;
    let mut _right_calibration: Option<SensorCalibration> = None;

    // Store the state whether logbot is currently on the line or not
    let mut on_line = false;

    'outer: while let Some(command) = channel.blocking_recv() {
        match command {
            Command::Health(response) => {
                let _ = response.send(());
            }
            Command::Demo(response) => {
                // Run the full demo, not responding to any incoming hardware commands
                let _ = response.send(DemoResult::Success);
                demo(
                    &mut hardware.vehicle,
                    &mut hardware.sensors,
                    &mut hardware.lift,
                )?;
                on_line = false;
                continue 'outer;
            }
            Command::FollowLine(response) => {
                if !on_line {
                    let _ = response.send(FollowLineResult::NotOnTheLine);
                    continue 'outer;
                };

                // Check that we have calibrated, so we can follow the line
                let calibration = match left_calibration {
                    Some(calibration) => {
                        // Line following can proceed
                        let _ = response.send(FollowLineResult::Success);
                        calibration
                    }
                    None => {
                        // Fail, since no calibration data is available
                        let _ = response.send(FollowLineResult::NoCalibration);
                        continue 'outer;
                    }
                };

                // Acceleration
                let mut acceleration = Acceleration::new(Duration::from_secs(2));

                // Create the config for following the line
                let config = FollowLineConfig {
                    default_speed: DEFAULT_SPEED,
                    proportional: 0.001,
                    derivative: 0.0005,
                    integral: None,
                    calibration,
                    reset_integral_on_target: true,
                };

                // Create state for line following from config
                let mut state = FollowLineState::new(config);

                // Lets start following the line while listening to new commands
                loop {
                    // We want to handle each command differently
                    if let Ok(command) = channel.try_recv() {
                        match command {
                            Command::Health(response) => {
                                let _ = response.send(());
                            }
                            Command::Demo(response) => {
                                // Run the full demo, not responding to any incoming hardware commands
                                let _ = response.send(DemoResult::Success);
                                demo(
                                    &mut hardware.vehicle,
                                    &mut hardware.sensors,
                                    &mut hardware.lift,
                                )?;
                                on_line = false;
                                continue 'outer;
                            }
                            Command::FollowLine(response) => {
                                // We are already following the line
                                let _ = response.send(FollowLineResult::AlreadyFollowing);
                            }
                            Command::Calibrate(response) => {
                                // Calibrate command should not overwrite following line command
                                let _ = response.send(CalibrateResult::Following);
                            }
                            Command::FindEdge(response) => {
                                let _ = response.send(FindEdgeResult::Follow);
                            }
                            Command::Stop(response) => {
                                // Stop the vehicle and break out the following loop
                                hardware.vehicle.stop()?;
                                let _ = response.send(StopResult::FollowLine);
                                continue 'outer;
                            }
                        };
                    };

                    // Move following state forward
                    let sensor_value = hardware.sensors.read(Sensors::Left)?;
                    let direction = state.step(sensor_value);
                    let direction = direction.accelerate(&mut acceleration);
                    hardware.vehicle.drive(direction)?;
                }
            }
            Command::Calibrate(response) => {
                on_line = false;

                // Respond with successful oscillation
                let _ = response.send(CalibrateResult::Success);

                // Oscillation configuration
                let oscillate = Oscillate::new(
                    Duration::from_millis(1000),
                    SpinDirection::Left(DEFAULT_SPEED * Speed::HALF),
                    NonZero::<u32>::new(2).unwrap(),
                );

                // Calibrate sensors by oscillating and evaulating sensor readings
                // Calibrate both sensors by logging values
                let mut left_sensor = SingleSensorCalibration::default();
                let mut right_sensor = SingleSensorCalibration::default();

                // Oscillate the vehicle starting with one second, doubling the time
                // on each direction change
                let mut oscillate = oscillate.start(&mut hardware.vehicle)?;

                // Wait until we first change direction, since we want to record
                // one contiguous line with the sensors

                // Do active waiting since we don't want to block incoming stop messages
                while !oscillate.should_step() {
                    // Check for incoming messages
                    if let Ok(command) = channel.try_recv() {
                        match command {
                            Command::Health(response) => {
                                let _ = response.send(());
                            }
                            Command::Demo(response) => {
                                // Run the full demo, not responding to any incoming hardware commands
                                let _ = response.send(DemoResult::Success);
                                demo(
                                    &mut hardware.vehicle,
                                    &mut hardware.sensors,
                                    &mut hardware.lift,
                                )?;
                                on_line = false;
                                continue 'outer;
                            }
                            Command::FollowLine(response) => {
                                // Follow line not available during calibration
                                let _ = response.send(FollowLineResult::MidCalibration);
                            }
                            Command::Calibrate(response) => {
                                let _ = response.send(CalibrateResult::AlreadyCalibrating);
                            }
                            Command::FindEdge(response) => {
                                let _ = response.send(FindEdgeResult::Calibrate);
                            }
                            Command::Stop(response) => {
                                // Stop the vehicle and stop oscillation
                                hardware.vehicle.stop()?;
                                let _ = response.send(StopResult::Calibrate);
                                continue 'outer;
                            }
                        }
                    }
                }

                oscillate.step(&mut hardware.vehicle)?;

                // Read sensor values continuously until we're supposed to oscillate again
                while !oscillate.should_step() {
                    // Check for incoming messages
                    if let Ok(command) = channel.try_recv() {
                        match command {
                            Command::Health(response) => {
                                let _ = response.send(());
                            }
                            Command::Demo(response) => {
                                // Run the full demo, not responding to any incoming hardware commands
                                let _ = response.send(DemoResult::Success);
                                demo(
                                    &mut hardware.vehicle,
                                    &mut hardware.sensors,
                                    &mut hardware.lift,
                                )?;
                                on_line = false;
                                continue 'outer;
                            }
                            Command::FollowLine(response) => {
                                // Follow line not available during calibration
                                let _ = response.send(FollowLineResult::MidCalibration);
                            }
                            Command::Calibrate(response) => {
                                let _ = response.send(CalibrateResult::AlreadyCalibrating);
                            }
                            Command::FindEdge(response) => {
                                let _ = response.send(FindEdgeResult::Calibrate);
                            }
                            Command::Stop(response) => {
                                // Stop the vehicle and stop oscillation
                                hardware.vehicle.stop()?;
                                let _ = response.send(StopResult::Calibrate);
                                continue 'outer;
                            }
                        }
                    }

                    // Read values from sensors
                    let left_value = hardware.sensors.read(Sensors::Left)?;
                    let right_value = hardware.sensors.read(Sensors::Right)?;

                    left_sensor.log(left_value as f64);
                    right_sensor.log(right_value as f64);
                }

                // Stop the vehicle once the oscillation is done
                hardware.vehicle.stop()?;

                // Evaluate sensor readings to get calibrated sensors
                left_calibration = Some(left_sensor.calibrate());
                _right_calibration = Some(right_sensor.calibrate());
            }
            Command::FindEdge(response) => {
                let calibration = match left_calibration {
                    Some(calibration) => calibration,
                    None => {
                        let _ = response.send(FindEdgeResult::NoCalibration);
                        continue 'outer;
                    }
                };

                let _ = response.send(FindEdgeResult::Success);

                // Oscillation configuration
                let mut oscillate = Oscillate::new(
                    Duration::from_secs(2),
                    SpinDirection::Left(DEFAULT_SPEED),
                    NonZero::<u32>::new(2).unwrap(),
                )
                .start(&mut hardware.vehicle)?;

                'edge: loop {
                    while !oscillate.should_step() {
                        // Check for incoming messages
                        if let Ok(command) = channel.try_recv() {
                            match command {
                                Command::Health(response) => {
                                    let _ = response.send(());
                                }
                                Command::Demo(response) => {
                                    // Run the full demo, not responding to any incoming hardware commands
                                    let _ = response.send(DemoResult::Success);
                                    demo(
                                        &mut hardware.vehicle,
                                        &mut hardware.sensors,
                                        &mut hardware.lift,
                                    )?;
                                    on_line = false;
                                    continue 'outer;
                                }
                                Command::FollowLine(response) => {
                                    let _ = response.send(FollowLineResult::FindingEdge);
                                }
                                Command::Calibrate(response) => {
                                    let _ = response.send(CalibrateResult::FindingEdge);
                                }
                                Command::FindEdge(response) => {
                                    let _ = response.send(FindEdgeResult::AlreadyEdging);
                                }
                                Command::Stop(response) => {
                                    // Stop finding edge
                                    hardware.vehicle.stop()?;
                                    let _ = response.send(StopResult::FindEdge);
                                    continue 'outer;
                                }
                            };
                        };
                        // Check if we have found the edge
                        let value = hardware.sensors.read(Sensors::Right)? as f64;
                        if (value - calibration.line as f64).abs() < 2.0 {
                            break 'edge;
                        };
                    }
                    // We should change directions
                    oscillate.step(&mut hardware.vehicle)?;
                }
                hardware.vehicle.stop()?;
                on_line = true;
            }
            Command::Stop(response) => {
                hardware.vehicle.stop()?;
                // The logbot is already currently not doing anything
                // We can simply return with a success value
                let _ = response.send(StopResult::Nothing);
            }
        };
    }
    Ok(())
}

/// Spawn actor thread that proccesses [`Command`] requests for [`Hardware`]
pub fn spawn_default() -> Result<mpsc::Sender<Command>> {
    // Initialize hardware using defaults
    let hardware = Hardware::try_default()?;

    // Channel for sending commands
    let (wx, rx) = mpsc::channel(10);

    // Start blocking thread to handle commands
    tokio::task::spawn_blocking(|| {
        if let Err(e) = handle_commands(hardware, rx) {
            println!("{}", e);
        }
    });
    Ok(wx)
}

/// Convenience struct to pass hardware components around
#[derive(Debug)]
pub struct Hardware {
    pub vehicle: Vehicle<DCMotor<Left>, DCMotor<Right>>,
    pub sensors: SensorController,
    pub lift: LiftMotor,
}

impl TryDefault for Hardware {
    type Error = anyhow::Error;

    fn try_default() -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            vehicle: Vehicle::try_default()?,
            sensors: SensorController::try_default()?,
            lift: LiftMotor::try_default()?,
        })
    }
}
