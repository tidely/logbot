//! Actor thread for handling hardware operations

use std::{fmt::Debug, num::NonZero, time::Duration};

use acceleration::{Accelerate, LinearAcceleration};

use calibration::{SensorCalibration, SingleSensorCalibration};
use consts::Sensors;
use demo::demo;
use directions::{SpinDirection, VehicleDirection};
use interfaces::{Drive, Lift, SensorRead, Spin};
use line::{FollowLineConfig, FollowLineState};
use logbot::error::LogbotError;
use oscillate::Oscillate;
use speed::Speed;
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

/// Default [`Speed`] at which the [`HardwareThread`] should operate
const DEFAULT_SPEED: Speed = Speed::new_const(0.1);

/// The [`Result`] of a [`Request`]
///
/// The Ok() variant means the command was executed successfully.
/// Containing the previous Command it cancelled.
///
/// The Err() variant means the command failed. The reason is contained
/// within the Err.
pub type CommandResult = std::result::Result<Command, CommandDenied>;

/// [`Request`] execution of a [`Command`] on the [`HardwareThread`]
pub type Request = (Command, oneshot::Sender<CommandResult>);

/// [`Command`]s that control hardware
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

impl ToString for Command {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl Command {
    /// Convert the [`Command`] to a string slice
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

/// Thread for handling hardware operations
#[derive(Debug)]
pub struct HardwareThread<L>
where
    L: Send,

    L: Drive<Direction = VehicleDirection>,
    <L as Drive>::Error: Debug + Send,

    L: Spin<SpinDirection = SpinDirection>,

    L: SensorRead<Output = u8>,
    <L as SensorRead>::Error: Debug + Send,

    L: Lift,
    <L as Lift>::Error: Debug + Send,
{
    channel: mpsc::Sender<Request>,
    handle: JoinHandle<
        Result<(), LogbotError<<L as Drive>::Error, <L as SensorRead>::Error, <L as Lift>::Error>>,
    >,
}

impl<L> HardwareThread<L>
where
    L: Send + 'static,

    L: Drive<Direction = VehicleDirection>,
    <L as Drive>::Error: Debug + Send,

    L: Spin<SpinDirection = SpinDirection>,

    L: SensorRead<Output = u8>,
    <L as SensorRead>::Error: Debug + Send,

    L: Lift,
    <L as Lift>::Error: Debug + Send,
{
    /// Spawn a new [`HardwareThread`]
    pub fn spawn(logbot: L) -> Self {
        let (wx, rx) = mpsc::channel(10);
        let handle = tokio::task::spawn_blocking(|| handle_commands(logbot, rx));
        Self {
            channel: wx,
            handle,
        }
    }

    /// Send a [`Command`] to the [`HardwareThread`]
    ///
    /// Returns [None](`Option::None`) when the [`HardwareThread`] is no longer running.
    pub async fn send(&self, command: Command) -> Option<CommandResult> {
        let (wx, rx) = oneshot::channel();
        // Both calls are successful when the thread is active
        self.channel.send((command, wx)).await.ok()?;
        rx.await.ok()
    }

    /// Whether the [`HardwareThread`] is finished
    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }
}

/// Process hardware requests syncronously
fn handle_commands<L>(
    mut logbot: L,
    mut channel: mpsc::Receiver<Request>,
) -> Result<(), LogbotError<<L as Drive>::Error, <L as SensorRead>::Error, <L as Lift>::Error>>
where
    L: Drive<Direction = VehicleDirection>,
    L: Spin<SpinDirection = SpinDirection>,
    L: SensorRead<Output = u8>,
    L: Lift,
{
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
                demo(&mut logbot)?;
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
                                logbot.stop().map_err(LogbotError::Vehicle)?;
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
                    let sensor_value = logbot.read(Sensors::Left).map_err(LogbotError::Sensor)?;
                    let direction = state.step(sensor_value);
                    let direction = direction.accelerate(&mut acceleration);
                    logbot.drive(direction).map_err(LogbotError::Vehicle)?;
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
                let mut oscillate = oscillate.start(&mut logbot).map_err(LogbotError::Vehicle)?;

                // Wait until we first change direction, since we want to record
                // one contiguous line with the sensors

                // Do active waiting since we don't want to block incoming stop messages
                while !oscillate.should_step() {
                    // Check for incoming messages
                    if let Ok((command, response)) = channel.try_recv() {
                        match command {
                            Command::Stop => {
                                // Stop the vehicle and stop oscillation
                                logbot.stop().map_err(LogbotError::Vehicle)?;
                                let _ = response.send(Ok(Command::Calibrate));
                                continue 'outer;
                            }
                            _ => {
                                let _ = response.send(Err(CommandDenied::Busy(Command::Calibrate)));
                            }
                        }
                    }
                }

                oscillate.step(&mut logbot).map_err(LogbotError::Vehicle)?;

                // Read sensor values continuously until we're supposed to oscillate again
                while !oscillate.should_step() {
                    // Check for incoming messages
                    if let Ok((command, response)) = channel.try_recv() {
                        match command {
                            Command::Stop => {
                                // Stop the vehicle and stop oscillation
                                logbot.stop().map_err(LogbotError::Vehicle)?;
                                let _ = response.send(Ok(Command::Calibrate));
                                continue 'outer;
                            }
                            _ => {
                                let _ = response.send(Err(CommandDenied::Busy(Command::Calibrate)));
                            }
                        }
                    }

                    // Read values from sensors
                    let left_value = logbot.read(Sensors::Left).map_err(LogbotError::Sensor)?;
                    let right_value = logbot.read(Sensors::Right).map_err(LogbotError::Sensor)?;

                    left_sensor.log(left_value as f64);
                    right_sensor.log(right_value as f64);
                }

                // Stop the vehicle once the oscillation is done
                logbot.stop().map_err(LogbotError::Vehicle)?;

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
                .start(&mut logbot)
                .map_err(LogbotError::Vehicle)?;

                'edge: loop {
                    while !oscillate.should_step() {
                        // Check for incoming messages
                        if let Ok((command, response)) = channel.try_recv() {
                            match command {
                                Command::Stop => {
                                    // Stop finding edge
                                    logbot.stop().map_err(LogbotError::Vehicle)?;
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
                        let value =
                            logbot.read(Sensors::Right).map_err(LogbotError::Sensor)? as f64;
                        if (value - calibration.line as f64).abs() < 2.0 {
                            break 'edge;
                        };
                    }
                    // We should change directions
                    oscillate.step(&mut logbot).map_err(LogbotError::Vehicle)?;
                }
                logbot.stop().map_err(LogbotError::Vehicle)?;
                on_line = true;
            }
            Command::LiftUp => {
                // Vehicle should be stopped, since lift is a blocking operating
                // It should be stopped anyway, but this makes sure it is
                let _ = logbot.stop();

                let _ = response.send(Ok(Command::LiftUp));
                logbot.up(Speed::HALF).map_err(LogbotError::Lift)?;
            }
            Command::LiftDown => {
                // Vehicle should be stopped, since lift is a blocking operating
                // It should be stopped anyway, but this makes sure it is
                let _ = logbot.stop();

                let _ = response.send(Ok(Command::LiftDown));
                logbot.down(Speed::HALF).map_err(LogbotError::Lift)?;
            }
            Command::Stop => {
                logbot.stop().map_err(LogbotError::Vehicle)?;
                // The logbot is already currently not doing anything
                // We can simply return with a success value
                let _ = response.send(Ok(Command::Stop));
            }
        };
    }
    Ok(())
}
