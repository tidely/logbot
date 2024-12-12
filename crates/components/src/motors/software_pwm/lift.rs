use std::time::Duration;

use rppal::gpio::{self, InputPin, OutputPin};
use speed::Speed;

/// Represents a [`LiftMotor`] that lifts objects
///
/// Reads its position from two [`InputPin`]s
#[derive(Debug)]
pub struct LiftMotor {
    /// [`OutputPin`] that moves the Lift Motor
    power: OutputPin,
    /// Direction [`OutputPin`] that sets the direction
    direction: OutputPin,
    /// Frequency of the Software PWM for the power pin
    frequency: f64,
    /// [`InputPin`] that checks whether Lift is in up position
    up: InputPin,
    /// [`InputPin`] that checks whether Lift is in down position
    down: InputPin,
}

impl LiftMotor {
    /// Create a new [`LiftMotor`]
    pub fn new(
        power: OutputPin,
        direction: OutputPin,
        frequency: f64,
        up: InputPin,
        down: InputPin,
    ) -> Self {
        Self {
            power,
            direction,
            frequency,
            up,
            down,
        }
    }

    /// Move the [`LiftMotor`] to its up position
    ///
    /// This is a blocking operation
    pub fn up(&mut self, speed: Speed) -> gpio::Result<()> {
        // Set the direction
        self.direction.set_low();

        if !self.is_up() {
            self.power
                .set_pwm_frequency(self.frequency, speed.value())?;

            while !self.is_up() {
                std::thread::sleep(Duration::from_millis(1));
            }
        };

        self.power.clear_pwm()?;

        Ok(())
    }

    /// Move the [`LiftMotor`] to its down position
    ///
    /// This is a blocking operation
    pub fn down(&mut self, speed: Speed) -> gpio::Result<()> {
        // Set the direction
        self.direction.set_high();

        if !self.is_down() {
            self.power
                .set_pwm_frequency(self.frequency, speed.value())?;

            while !self.is_down() {
                std::thread::sleep(Duration::from_millis(1));
            }
        };

        self.power.clear_pwm()?;

        Ok(())
    }

    /// Check whether the lift is in the up position
    pub fn is_up(&mut self) -> bool {
        self.up.is_low()
    }

    /// Check if the lift is in the down position
    pub fn is_down(&mut self) -> bool {
        self.down.is_low()
    }
}
