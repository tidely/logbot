//! Fallible Default trait
//!
//! We also implement the trait for some hardware components using the [`consts`] crate

use std::time::Duration;

use components::hardware_pwm;
use components::software_pwm;
use components::software_pwm::LiftMotor;
use components::{Left, PwmConfig, Right, SensorController};
use consts::{
    pins::{self, LEFT_MOTOR_POWER, RIGHT_MOTOR_POWER},
    FREQUENCY, I2C_SENSOR_ADDRESS,
};
use interfaces::Drive;
use rppal::pwm::Channel;
use rppal::pwm::{self, Pwm};
use rppal::{
    gpio::{self, Gpio},
    i2c::{self, I2c},
};
use vehicle::Vehicle;
use vehicle::VehicleError;

/// The hardware PWM Channel for the left Motor
const LEFT_MOTOR_CHANNEL: Channel = Channel::Pwm0;
/// The hardware PWM Channel for the right Motor
const RIGHT_MOTOR_CHANNEL: Channel = Channel::Pwm1;

/// Trait for generating fallible [`Default`] implementations
pub trait TryDefault: Sized {
    /// The [Error](`core::error::Error`)
    type Error;

    /// Generate the default implementation using the [`consts`] crate
    fn try_default() -> Result<Self, Self::Error>;
}

impl TryDefault for software_pwm::SignedMotor<Left> {
    type Error = gpio::Error;

    fn try_default() -> Result<Self, Self::Error> {
        Ok(Self::new(
            Gpio::new()?.get(pins::LEFT_MOTOR_POWER)?.into_output_low(),
            FREQUENCY,
            Gpio::new()?
                .get(pins::LEFT_MOTOR_DIRECTION)?
                .into_output_low(),
        ))
    }
}

impl TryDefault for software_pwm::SignedMotor<Right> {
    type Error = gpio::Error;

    fn try_default() -> Result<Self, Self::Error> {
        Ok(Self::new(
            Gpio::new()?.get(pins::RIGHT_MOTOR_POWER)?.into_output_low(),
            FREQUENCY,
            Gpio::new()?
                .get(pins::RIGHT_MOTOR_DIRECTION)?
                .into_output_low(),
        ))
    }
}

impl TryDefault for software_pwm::DCMotor<Left> {
    type Error = gpio::Error;

    fn try_default() -> Result<Self, Self::Error> {
        let config = PwmConfig {
            period: Duration::from_millis(20),
            stop_pulse_width: Duration::from_micros(1500),
            pulse_width_range: Duration::from_micros(500),
        };
        Ok(software_pwm::DCMotor::new(
            Gpio::new()?.get(LEFT_MOTOR_POWER)?.into_output_low(),
            config,
        )?)
    }
}

impl TryDefault for software_pwm::DCMotor<Right> {
    type Error = gpio::Error;

    fn try_default() -> Result<Self, Self::Error> {
        let config = PwmConfig {
            period: Duration::from_millis(20),
            stop_pulse_width: Duration::from_micros(1468),
            pulse_width_range: Duration::from_micros(500),
        };
        Ok(software_pwm::DCMotor::new(
            Gpio::new()?.get(RIGHT_MOTOR_POWER)?.into_output_low(),
            config,
        )?)
    }
}

impl TryDefault for SensorController {
    type Error = i2c::Error;

    fn try_default() -> Result<Self, Self::Error> {
        let mut i2c = I2c::new()?;
        i2c.set_slave_address(I2C_SENSOR_ADDRESS)?;
        Ok(SensorController::new(i2c))
    }
}

impl TryDefault for hardware_pwm::DCMotor<Left> {
    type Error = pwm::Error;

    fn try_default() -> Result<Self, Self::Error> {
        let config = PwmConfig {
            period: Duration::from_millis(20),
            stop_pulse_width: Duration::from_micros(1480),
            pulse_width_range: Duration::from_micros(500),
        };
        let pwm = Pwm::new(LEFT_MOTOR_CHANNEL)?;
        let motor = hardware_pwm::DCMotor::new(pwm, config)?;
        Ok(motor)
    }
}

impl TryDefault for hardware_pwm::DCMotor<Right> {
    type Error = pwm::Error;

    fn try_default() -> Result<Self, Self::Error> {
        let config = PwmConfig {
            period: Duration::from_millis(20),
            stop_pulse_width: Duration::from_micros(1465),
            pulse_width_range: Duration::from_micros(500),
        };
        let pwm = Pwm::new(RIGHT_MOTOR_CHANNEL)?;
        let motor = hardware_pwm::DCMotor::new(pwm, config)?;
        Ok(motor)
    }
}

impl<LM, RM> TryDefault for Vehicle<LM, RM>
where
    LM: Drive + TryDefault,
    RM: Drive + TryDefault,
{
    type Error = VehicleError<<LM as TryDefault>::Error, <RM as TryDefault>::Error>;

    fn try_default() -> Result<Self, Self::Error> {
        let left = LM::try_default().map_err(|e| VehicleError::Left(e))?;
        let right = RM::try_default().map_err(|e| VehicleError::Right(e))?;
        Ok(Vehicle::new(left, right))
    }
}

impl TryDefault for LiftMotor {
    type Error = gpio::Error;

    fn try_default() -> Result<Self, Self::Error> {
        let power = Gpio::new()?.get(pins::LIFT_MOTOR_POWER)?.into_output_low();
        let direction = Gpio::new()?
            .get(pins::LIFT_MOTOR_DIRECTION)?
            .into_output_low();
        let up = Gpio::new()?.get(pins::LIFT_UP)?.into_input();
        let down = Gpio::new()?.get(pins::LIFT_DOWN)?.into_input();

        Ok(Self::new(power, direction, FREQUENCY, up, down))
    }
}
