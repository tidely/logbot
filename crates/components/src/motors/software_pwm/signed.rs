//! Motor using Signed Magnitude Software PWM Controls

use std::marker::PhantomData;

use directions::MotorDirection;
use interfaces::Drive;
use rppal::gpio::{self, OutputPin};

use crate::{Left, Right};

/// Motor Component
///
/// A motor component can be mounted either on the [`Left`] or [`Right`] side.
/// The power pin of the [`SignedMotor`] is controlled using software PWM.
#[derive(Debug)]
pub struct SignedMotor<Side> {
    /// [`OutputPin`] for controlling the [`Speed`] of the [`SignedMotor`].
    /// This is controlled using software PWM.
    power: OutputPin,
    /// The operating frequency of the power pin PWM. 4096.0 is a good default.
    frequency: f64,
    /// [`OutputPin`] for controlling the [`MotorDirection`]
    /// The output state will be different depending on the 'Side' of the motor
    direction: OutputPin,
    /// Stores the current state of the motor
    state: Option<MotorDirection>,
    /// Zero-sized phantom data that stores the side of the Motor
    _phantom: PhantomData<Side>,
}

impl<Side> SignedMotor<Side> {
    /// Create a new [`SignedMotor`] instance
    pub fn new(power: OutputPin, frequency: f64, direction: OutputPin) -> Self {
        Self {
            power,
            frequency,
            direction,
            state: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl Drive for SignedMotor<Right> {
    type Direction = MotorDirection;
    type Error = gpio::Error;

    fn drive(&mut self, direction: Self::Direction) -> gpio::Result<Option<Self::Direction>> {
        match direction {
            Self::Direction::Forward(speed) => {
                self.direction.set_high();
                self.power
                    .set_pwm_frequency(self.frequency, speed.value())?;
            }
            Self::Direction::Backward(speed) => {
                self.direction.set_low();
                self.power
                    .set_pwm_frequency(self.frequency, speed.value())?;
            }
        };
        Ok(self.state.replace(direction))
    }

    fn stop(&mut self) -> gpio::Result<Option<Self::Direction>> {
        self.power.set_low();
        self.power.clear_pwm()?;
        Ok(self.state.take())
    }
}

impl Drive for SignedMotor<Left> {
    type Direction = MotorDirection;
    type Error = gpio::Error;

    fn drive(&mut self, direction: Self::Direction) -> gpio::Result<Option<Self::Direction>> {
        match direction {
            Self::Direction::Forward(speed) => {
                self.direction.set_low();
                self.power
                    .set_pwm_frequency(self.frequency, speed.value())?;
            }
            Self::Direction::Backward(speed) => {
                self.direction.set_high();
                self.power
                    .set_pwm_frequency(self.frequency, speed.value())?;
            }
        };
        Ok(self.state.replace(direction))
    }

    fn stop(&mut self) -> gpio::Result<Option<Self::Direction>> {
        self.power.set_low();
        self.power.clear_pwm()?;
        Ok(self.state.take())
    }
}
