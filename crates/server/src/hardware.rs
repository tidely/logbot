//! Actor thread for handling hardware operations

use std::{num::NonZero, time::Duration};

use acceleration::{Accelerate, LinearAcceleration};
use anyhow::Result;

use calibration::{SensorCalibration, SingleSensorCalibration};
use components::{hardware_pwm::DCMotor, software_pwm::LiftMotor, Left, Right, SensorController};
use consts::Sensors;
use defaults::TryDefault;
use demo::demo;
use directions::SpinDirection;
use interfaces::{Drive, Lift, SensorRead};
use line::{FollowLineConfig, FollowLineState};
use oscillate::Oscillate;
use speed::Speed;
use tokio::sync::{mpsc, oneshot};
use vehicle::Vehicle;

const DEFAULT_SPEED: Speed = unsafe { Speed::new_unchecked(0.1) };

/// The [`Result`] of a [`Request`]
///
/// The Ok() variant means the command was executed successfully.
/// Containing the previous Command it cancelled.
///
/// The Err() variant means the command failed. The reason is contained
/// within the Err.
pub type CommandResult = Result<Command, CommandDenied>;

/// [`Request`] execution of a [`Command`] from the [`Hardware`] thread
pub type Request = (Command, oneshot::Sender<CommandResult>);

/// [`Command`]s that control [`Hardware`]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    FollowLine,
    Calibrate,
    FindEdge,
    LiftUp,
    LiftDown,
    Stop,
    Demo,
}

impl Command {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stop => "Stop",
            Self::LiftUp => "LiftUp",
            Self::LiftDown => "LiftDown",
            Self::Calibrate => "Calibrate",
            Self::FindEdge => "FindEdge",
            Self::FollowLine => "FollowLine",
            Self::Demo => "Demo",
        }
    }
}

/// Reasons for a [`Command`] being denied
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandDenied {
    Busy(Command),
    Required(Command),
}

// TODO: Make handler generic over hardware
/// Process hardware requests syncronously
fn handle_commands(
    mut hardware: Hardware,
    mut channel: mpsc::Receiver<Request>,
) -> Result<(), Box<dyn core::error::Error>> {
    // Store the current calibration status
    let mut left_calibration: Option<SensorCalibration> = None;
    let mut _right_calibration: Option<SensorCalibration> = None;

    // Store the state whether logbot is currently on the line or not
    let mut on_line = false;

    'outer: while let Some((command, response)) = channel.blocking_recv() {
        match command {
            Command::Demo => {
                // Run the full demo, not responding to any incoming hardware commands
                let _ = response.send(Ok(Command::Stop));
                demo(
                    &mut hardware.vehicle,
                    &mut hardware.sensors,
                    &mut hardware.lift,
                )?;
                on_line = false;
                continue 'outer;
            }
            Command::FollowLine => {
                if !on_line {
                    let _ = response.send(Err(CommandDenied::Required(Command::FindEdge)));
                    continue 'outer;
                };

                // Check that we have calibrated, so we can follow the line
                let calibration = match left_calibration {
                    Some(calibration) => {
                        // Line following can proceed
                        let _ = response.send(Ok(Command::Stop));
                        calibration
                    }
                    None => {
                        // Fail, since no calibration data is available
                        let _ = response.send(Err(CommandDenied::Required(Command::Calibrate)));
                        continue 'outer;
                    }
                };

                let mut acceleration = LinearAcceleration::new(Duration::from_secs(2));

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
                    if let Ok((command, response)) = channel.try_recv() {
                        match command {
                            Command::Stop => {
                                // Stop the vehicle and break out the following loop
                                hardware.vehicle.stop()?;
                                let _ = response.send(Ok(Command::FollowLine));
                                continue 'outer;
                            }
                            _ => {
                                let _ =
                                    response.send(Err(CommandDenied::Busy(Command::FollowLine)));
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
            Command::Calibrate => {
                on_line = false;

                // Respond with successful oscillation
                let _ = response.send(Ok(Command::Stop));

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
                    if let Ok((command, response)) = channel.try_recv() {
                        match command {
                            Command::Stop => {
                                // Stop the vehicle and stop oscillation
                                hardware.vehicle.stop()?;
                                let _ = response.send(Ok(Command::Calibrate));
                                continue 'outer;
                            }
                            _ => {
                                let _ = response.send(Err(CommandDenied::Busy(Command::Calibrate)));
                            }
                        }
                    }
                }

                oscillate.step(&mut hardware.vehicle)?;

                // Read sensor values continuously until we're supposed to oscillate again
                while !oscillate.should_step() {
                    // Check for incoming messages
                    if let Ok((command, response)) = channel.try_recv() {
                        match command {
                            Command::Stop => {
                                // Stop the vehicle and stop oscillation
                                hardware.vehicle.stop()?;
                                let _ = response.send(Ok(Command::Calibrate));
                                continue 'outer;
                            }
                            _ => {
                                let _ = response.send(Err(CommandDenied::Busy(Command::Calibrate)));
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
            Command::FindEdge => {
                let calibration = match left_calibration {
                    Some(calibration) => calibration,
                    None => {
                        let _ = response.send(Err(CommandDenied::Required(Command::Calibrate)));
                        continue 'outer;
                    }
                };

                let _ = response.send(Ok(Command::Stop));

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
                        if let Ok((command, response)) = channel.try_recv() {
                            match command {
                                Command::Stop => {
                                    // Stop finding edge
                                    hardware.vehicle.stop()?;
                                    on_line = false;
                                    let _ = response.send(Ok(Command::FindEdge));
                                    continue 'outer;
                                }
                                _ => {
                                    let _ =
                                        response.send(Err(CommandDenied::Busy(Command::FindEdge)));
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
            Command::LiftUp => {
                // Vehicle should be stopped, since lift is a blocking operating
                // It should be stopped anyway, but this makes sure it is
                let _ = hardware.vehicle.stop();

                let _ = response.send(Ok(Command::LiftUp));
                hardware.lift.up(Speed::HALF)?;
            }
            Command::LiftDown => {
                // Vehicle should be stopped, since lift is a blocking operating
                // It should be stopped anyway, but this makes sure it is
                let _ = hardware.vehicle.stop();

                let _ = response.send(Ok(Command::LiftDown));
                hardware.lift.down(Speed::HALF)?;
            }
            Command::Stop => {
                hardware.vehicle.stop()?;
                // The logbot is already currently not doing anything
                // We can simply return with a success value
                let _ = response.send(Ok(Command::Stop));
            }
        };
    }
    Ok(())
}

/// Spawn actor thread that proccesses [`Command`] requests for [`Hardware`]
pub fn spawn_default() -> Result<mpsc::Sender<Request>> {
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
