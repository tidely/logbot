//! DCMotor with a Hardware [`Pwm`] Implementation

use std::marker::PhantomData;

use directions::MotorDirection;
use interfaces::Drive;
use rppal::pwm::{self, Pwm};

use crate::{Left, PwmConfig, Right};

/// DC Motor that uses Hardware [`Pwm`]
#[derive(Debug)]
pub struct DCMotor<Side> {
    /// The underlying [`Pwm`] that the Motor uses
    pwm: Pwm,
    /// The [`Pwm`] Configuration for the specific [`HardwareDCMotor`]
    config: PwmConfig,
    /// State of the Motor
    state: Option<MotorDirection>,
    /// Zero-sized phantom data that stores the side of the Motor
    _phantom: PhantomData<Side>,
}

impl<Side> DCMotor<Side> {
    /// Create a new [`DCMotor`] using a [`PwmConfig`]
    /// This activates the motor and the caller should wait a few seconds before
    /// using the motor (activation period)
    pub fn new(pwm: Pwm, config: PwmConfig) -> pwm::Result<Self> {
        // Set period
        pwm.set_period(config.period)?;
        pwm.enable()?;
        // Make sure motor is activated by sending stop pulse width
        pwm.set_pulse_width(config.stop_pulse_width)?;

        Ok(Self {
            pwm,
            config,
            state: None,
            _phantom: PhantomData,
        })
    }
}

impl Drive for DCMotor<Left> {
    type Direction = MotorDirection;
    type Error = pwm::Error;

    fn drive(
        &mut self,
        direction: Self::Direction,
    ) -> Result<Option<Self::Direction>, Self::Error> {
        match direction {
            Self::Direction::Forward(speed) => {
                let pulse_width = self.config.stop_pulse_width
                    - self.config.pulse_width_range.mul_f64(speed.value());
                self.pwm.set_pulse_width(pulse_width)?;
            }
            Self::Direction::Backward(speed) => {
                let pulse_width = self.config.stop_pulse_width
                    + self.config.pulse_width_range.mul_f64(speed.value());
                self.pwm.set_pulse_width(pulse_width)?;
            }
        };

        Ok(self.state.replace(direction))
    }

    fn stop(&mut self) -> Result<Option<Self::Direction>, Self::Error> {
        self.pwm.set_pulse_width(self.config.stop_pulse_width)?;
        Ok(self.state.take())
    }
}

impl Drive for DCMotor<Right> {
    type Direction = MotorDirection;
    type Error = pwm::Error;

    fn drive(
        &mut self,
        direction: Self::Direction,
    ) -> Result<Option<Self::Direction>, Self::Error> {
        match direction {
            Self::Direction::Forward(speed) => {
                let pulse_width = self.config.stop_pulse_width
                    + self.config.pulse_width_range.mul_f64(speed.value());
                self.pwm.set_pulse_width(pulse_width)?;
            }
            Self::Direction::Backward(speed) => {
                let pulse_width = self.config.stop_pulse_width
                    - self.config.pulse_width_range.mul_f64(speed.value());
                self.pwm.set_pulse_width(pulse_width)?;
            }
        };

        Ok(self.state.replace(direction))
    }

    fn stop(&mut self) -> Result<Option<Self::Direction>, Self::Error> {
        self.pwm.set_pulse_width(self.config.stop_pulse_width)?;
        Ok(self.state.take())
    }
}
