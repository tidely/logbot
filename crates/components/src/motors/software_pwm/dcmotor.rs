//! Motor using Brushless DC Software PWM Controls

use std::{marker::PhantomData, time::Duration};

use directions::MotorDirection;
use interfaces::Drive;
use rppal::gpio::{self, OutputPin};

use crate::{Left, PwmConfig, Right};

/// Brushless DC Motor that Locked Anti-phase PWM for controls
#[derive(Debug)]
pub struct DCMotor<Side> {
    /// [`OutputPin`] that controls [`Speed`] and [`MotorDirection`]
    power: OutputPin,
    /// Configuration of the pwm
    pwm_config: PwmConfig,
    /// State of the Motor
    state: Option<MotorDirection>,
    /// Zero-sized phantom data that stores the side of the Motor
    _phantom: PhantomData<Side>,
}

impl<Side> DCMotor<Side> {
    /// Create a new [`DCMotor`] using a [`PwmConfig`]
    pub fn new(mut power: OutputPin, pwm_config: PwmConfig) -> gpio::Result<Self> {
        // Start the motor
        power.set_pwm(pwm_config.period, pwm_config.stop_pulse_width)?;
        std::thread::sleep(Duration::from_secs(5));
        Ok(Self {
            power,
            pwm_config,
            state: None,
            _phantom: PhantomData,
        })
    }
}

impl Drive for DCMotor<Left> {
    type Direction = MotorDirection;
    type Error = gpio::Error;

    fn drive(
        &mut self,
        direction: Self::Direction,
    ) -> Result<Option<Self::Direction>, Self::Error> {
        match direction {
            Self::Direction::Forward(speed) => {
                let pulse_width = self.pwm_config.stop_pulse_width
                    - self.pwm_config.pulse_width_range.mul_f64(speed.value());
                self.power.set_pwm(self.pwm_config.period, pulse_width)?;
            }
            Self::Direction::Backward(speed) => {
                let pulse_width = self.pwm_config.stop_pulse_width
                    + self.pwm_config.pulse_width_range.mul_f64(speed.value());
                self.power.set_pwm(self.pwm_config.period, pulse_width)?;
            }
        };
        Ok(self.state.replace(direction))
    }

    fn stop(&mut self) -> Result<Option<Self::Direction>, Self::Error> {
        self.power
            .set_pwm(self.pwm_config.period, self.pwm_config.stop_pulse_width)?;
        Ok(self.state.take())
    }
}

impl Drive for DCMotor<Right> {
    type Direction = MotorDirection;
    type Error = gpio::Error;

    fn drive(
        &mut self,
        direction: Self::Direction,
    ) -> Result<Option<Self::Direction>, Self::Error> {
        match direction {
            Self::Direction::Forward(speed) => {
                let pulse_width = self.pwm_config.stop_pulse_width
                    + self.pwm_config.pulse_width_range.mul_f64(speed.value());
                self.power.set_pwm(self.pwm_config.period, pulse_width)?;
            }
            Self::Direction::Backward(speed) => {
                let pulse_width = self.pwm_config.stop_pulse_width
                    - self.pwm_config.pulse_width_range.mul_f64(speed.value());
                self.power.set_pwm(self.pwm_config.period, pulse_width)?;
            }
        };
        Ok(self.state.replace(direction))
    }

    fn stop(&mut self) -> Result<Option<Self::Direction>, Self::Error> {
        self.power
            .set_pwm(self.pwm_config.period, self.pwm_config.stop_pulse_width)?;
        Ok(self.state.take())
    }
}
