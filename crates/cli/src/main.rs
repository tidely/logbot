//! Command-line Interface for controlling logbot using the keyboard

use std::{
    io::stdout,
    num::NonZero,
    time::{Duration, Instant},
};

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{
        self, Event, KeyCode, KeyEventKind, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute, terminal,
};

use components::{hardware_pwm::DCMotor, software_pwm::LiftMotor};
//use components::software_pwm::DCMotor;
use calibration::{SensorCalibration, SingleSensorCalibration};
use components::{Left, Right, SensorController};
use consts::Sensors;
use defaults::TryDefault;
use directions::{MotorDirection, SpinDirection, VehicleDirection};
use interfaces::{Drive, Lift, SensorRead, Spin};
use line::{FollowLineConfig, FollowLineState};
use oscillate::Oscillate;
use speed::Speed;
use vehicle::Vehicle;

const FORWARD: u8 = 0b0001;
const BACKWARD: u8 = 0b0010;
const LEFT: u8 = 0b0100;
const RIGHT: u8 = 0b1000;

/// Control logbot using the keyboard
#[derive(Parser)]
struct Args {
    /// [`Speed`] of logbot (from 0 to 100)
    #[arg(short, long, value_parser = clap::value_parser!(u8).range(0..100), default_value_t = 10)]
    speed: u8,
}

/// Logbot - bundle vehicle and sensors into a single struct
#[derive(Debug)]
struct Logbot {
    vehicle: Vehicle<DCMotor<Left>, DCMotor<Right>>,
    sensors: SensorController,
    lift: LiftMotor,
    calibration: Option<SensorCalibration>,
}

/// Turn a [`u8`] that represents state into a [`VehicleDirection`]
fn u8_into_state(mut state: u8, speed: Speed) -> Option<VehicleDirection> {
    // First remove contradicting states
    if state & RIGHT != 0 && state & LEFT != 0 {
        state &= !RIGHT & !LEFT;
    };
    if state & FORWARD != 0 && state & BACKWARD != 0 {
        state &= !FORWARD & !BACKWARD;
    };

    // This is the ratio at which logbot turns when a horizontal and vertical
    // state are selected
    let turn_speed = speed / NonZero::<usize>::new(3).unwrap();

    if state & (FORWARD | LEFT) == (FORWARD | LEFT) {
        Some(VehicleDirection::new(
            MotorDirection::Forward(turn_speed),
            MotorDirection::Forward(speed),
        ))
    } else if state & (FORWARD | RIGHT) == (FORWARD | RIGHT) {
        Some(VehicleDirection::new(
            MotorDirection::Forward(speed),
            MotorDirection::Forward(turn_speed),
        ))
    } else if state & (BACKWARD | LEFT) == (BACKWARD | LEFT) {
        Some(VehicleDirection::new(
            MotorDirection::Backward(turn_speed),
            MotorDirection::Backward(speed),
        ))
    } else if state & (BACKWARD | RIGHT) == (BACKWARD | RIGHT) {
        Some(VehicleDirection::new(
            MotorDirection::Backward(speed),
            MotorDirection::Backward(turn_speed),
        ))
    } else if state & FORWARD != 0 {
        Some(VehicleDirection::forward(speed))
    } else if state & BACKWARD != 0 {
        Some(VehicleDirection::backward(speed))
    } else if state & RIGHT != 0 {
        Some(VehicleDirection::spin_right(speed))
    } else if state & LEFT != 0 {
        Some(VehicleDirection::spin_left(speed))
    } else {
        None
    }
}

/// Result of a [`check_key`] poll
#[derive(Debug, Clone, Copy)]
enum KeyPoll {
    Target,
    Esc,
}

/// Helper method for checking if a target key was pressed
fn check_key(target: char) -> Result<Option<KeyPoll>> {
    if !event::poll(Duration::ZERO)? {
        return Ok(None);
    };
    if let Event::Key(key) = event::read()? {
        match key.code {
            KeyCode::Esc => return Ok(Some(KeyPoll::Esc)),
            KeyCode::Char(c) if c == target && key.kind == KeyEventKind::Press => {
                return Ok(Some(KeyPoll::Target))
            }
            _ => {}
        };
    };
    Ok(None)
}

/// Calibrate sensors by oscillating over the line
/// Logbot should be above the line when this is called,
/// however this cannot be enforced by the program, but rather
/// something the user has to know
///
/// Returns Some(key) when a exit method was detected, and if the
/// calibration ended successfully we return None
fn calibrate(logbot: &mut Logbot) -> Result<Option<KeyPoll>> {
    // Records sensor values and produces calibrated sensor
    let mut log = SingleSensorCalibration::default();

    // Configure and start oscillation
    let mut oscillate = Oscillate::new(
        Duration::from_secs(1),
        SpinDirection::Left(Speed::HALF),
        NonZero::<u32>::new(2).unwrap(),
    )
    .start(&mut logbot.vehicle)?;

    // Actively wait for the first oscillation step
    while !oscillate.should_step() {
        // Check for incoming events
        if let Some(key) = check_key('c')? {
            logbot.vehicle.stop()?;
            return Ok(Some(key));
        }
    }
    // Change directions for the first time
    // Now we want to start recording data
    oscillate.step(&mut logbot.vehicle)?;

    // Read sensor values continuously until we're supposed to oscillate again
    // while checking for cancelling events
    while !oscillate.should_step() {
        // Check for keypresses that could cancel the operation
        if let Some(key) = check_key('c')? {
            logbot.vehicle.stop()?;
            return Ok(Some(key));
        };

        // Read and log values from sensor
        let left_value = logbot.sensors.read(Sensors::Left)?;
        log.log(left_value as f64);
    }

    // Move logbot back to its original position
    // We want to spin back left for one second
    let start = Instant::now();
    logbot.vehicle.spin(SpinDirection::Left(Speed::HALF))?;

    while start.elapsed() < Duration::from_secs(1) {
        // Once again listen for cancelling event
        if let Some(key) = check_key('c')? {
            logbot.vehicle.stop()?;
            return Ok(Some(key));
        };
    }

    logbot.vehicle.stop()?;
    logbot.calibration = Some(log.calibrate());
    Ok(None)
}

/// Follow the line until 'e' or Esc is pressed
fn follow_line(logbot: &mut Logbot) -> Result<KeyPoll> {
    assert!(logbot.calibration.is_some());

    // Create config from calibration
    let config = FollowLineConfig {
        default_speed: Speed::HALF,
        proportional: 0.6,
        derivative: 0.3,
        integral: None,
        calibration: logbot.calibration.unwrap(),
        reset_integral_on_target: true,
    };

    // Set up state for following a line
    let mut follow_line = FollowLineState::new(config);

    // Indefinitely follow the line
    loop {
        // Check for cancelling events
        if let Some(key) = check_key('e')? {
            logbot.vehicle.stop()?;
            return Ok(key);
        };

        let sensor_value = logbot.sensors.read(Sensors::Left)?;
        let direction = follow_line.step(sensor_value);
        logbot.vehicle.drive(direction)?;
    }
}

/// The main CLI of the program, terminal raw mode needs to be enabled
fn cli(logbot: &mut Logbot, speed: Speed) -> Result<()> {
    // Enforce that raw mode is enabled
    anyhow::ensure!(terminal::is_raw_mode_enabled()?);

    let mut state: u8 = 0b0000;
    let lift_speed = Speed::HALF;

    // Read keyboard events
    loop {
        if let Event::Key(key) = event::read()? {
            match key.kind {
                KeyEventKind::Press => match key.code {
                    // Add the modifier to the state
                    KeyCode::Char(c) => {
                        let modifier = match c {
                            'w' => FORWARD,
                            's' => BACKWARD,
                            'a' => LEFT,
                            'd' => RIGHT,
                            // Calibration
                            'c' => {
                                match calibrate(logbot)? {
                                    // Exit program
                                    Some(KeyPoll::Esc) => break,
                                    // Completed successfully or cancelled
                                    _ => continue,
                                };
                            }
                            // Follow line
                            'e' => {
                                if logbot.calibration.is_some() {
                                    match follow_line(logbot)? {
                                        KeyPoll::Esc => break,
                                        KeyPoll::Target => continue,
                                    };
                                } else {
                                    continue;
                                }
                            }
                            _ => continue,
                        };

                        state |= modifier;
                        match u8_into_state(state, speed) {
                            Some(direction) => logbot.vehicle.drive(direction)?,
                            None => logbot.vehicle.stop()?,
                        };
                    }
                    // Exit the program
                    KeyCode::Esc => {
                        break;
                    }
                    // Moving the lift is a blocking operation, this means any
                    // current movement could not be cancelled during the lift operation
                    //
                    // To prevent collisions we should only allow lift movement when
                    // logbot is stationary
                    KeyCode::Up if state == 0 => {
                        logbot.lift.up(lift_speed)?;
                        continue;
                    }
                    // Moving the lift is a blocking operation, this means any
                    // current movement could not be cancelled during the lift operation
                    //
                    // To prevent collisions we should only allow lift movement when
                    // logbot is stationary
                    KeyCode::Down if state == 0 => {
                        logbot.lift.down(lift_speed)?;
                        continue;
                    }
                    _ => {}
                },
                KeyEventKind::Release => match key.code {
                    // Remove the modifier from the state
                    KeyCode::Char(c) => {
                        let modifier = match c {
                            'w' => !FORWARD,
                            's' => !BACKWARD,
                            'a' => !LEFT,
                            'd' => !RIGHT,
                            _ => continue,
                        };

                        state &= modifier;
                        match u8_into_state(state, speed) {
                            Some(direction) => logbot.vehicle.drive(direction)?,
                            None => logbot.vehicle.stop()?,
                        };
                    }
                    _ => {}
                },
                _ => {}
            };
        };
    }

    // Stop the vehicle when user exists the program
    logbot.vehicle.stop()?;
    Ok(())
}

/// Entrypoint for the `cli` binary
fn main() -> Result<()> {
    let args = Args::parse();

    // Get the logbot speed from args
    let speed = Speed::new_clamp(args.speed as f64 / 100.0);

    let right_motor: DCMotor<Right> = DCMotor::try_default()?;
    let left_motor: DCMotor<Left> = DCMotor::try_default()?;
    // Make sure to sleep through activation period
    std::thread::sleep(Duration::from_secs(5));

    let vehicle = Vehicle::new(left_motor, right_motor);
    let sensors = SensorController::try_default()?;

    let lift = LiftMotor::try_default()?;

    let mut logbot = Logbot {
        vehicle,
        sensors,
        lift,
        calibration: None,
    };

    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    let flag = PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES);
    execute!(stdout, flag)?;

    // We run the main code in another function since we still need to disable
    // terminal raw mode even if we encounter an error
    let result = cli(&mut logbot, speed);

    execute!(stdout, PopKeyboardEnhancementFlags)?;
    terminal::disable_raw_mode()?;

    // Always stop the vehicle.
    logbot.vehicle.stop()?;

    result
}
