use std::ops::{Mul, Not};

use speed::{Speed, SpeedControl};

use crate::Stop;

/// Directions in which a Motor can move
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MotorDirection {
    /// Forward direction
    Forward(Speed),
    /// Backward direction
    Backward(Speed),
}

impl SpeedControl for MotorDirection {
    fn speed(&self) -> Speed {
        match self {
            Self::Forward(speed) => *speed,
            Self::Backward(speed) => *speed,
        }
    }

    fn with_speed(self, speed: Speed) -> Self {
        match self {
            Self::Forward(_) => Self::Forward(speed),
            Self::Backward(_) => Self::Backward(speed),
        }
    }
}

impl MotorDirection {
    /// Saturating float addition
    /// Adds a [`f64`] speed to the value, saturating at [`Speed`] bounds.
    pub fn saturating_add_f64(&self, value: f64) -> Self {
        self.with_speed(self.speed().saturating_add_f64(value))
    }

    /// Saturating float subtraction
    /// Subtracts a [`f64`] speed to the value, saturating at [`Speed`] bounds.
    pub fn wrapping_sub_f64(&self, value: f64) -> Self {
        let new_speed = self.speed().value() - value;
        if new_speed < 0.0 {
            self.not().with_speed(Speed::new_clamp(new_speed.abs()))
        } else {
            self.with_speed(Speed::new_clamp(new_speed))
        }
    }

    /// Saturating [`Speed`] addition
    /// Adds a [`Speed`] to the value, saturating at [`Speed`] bounds.
    pub fn saturating_add(self, speed: Speed) -> Self {
        self.with_speed(self.speed().saturating_add(speed))
    }

    /// Saturating [`Speed`] subtraction
    /// Subtracts a [`Speed`] from the value, saturating at [`Speed`] bounds.
    pub fn wrapping_sub(self, speed: Speed) -> Self {
        self.wrapping_sub_f64(speed.value())
    }
}

impl Stop for MotorDirection {
    fn is_stop(&self) -> bool {
        self.speed().value() == 0.0
    }
}

impl Mul<Speed> for MotorDirection {
    type Output = Self;

    fn mul(self, rhs: Speed) -> Self::Output {
        self.with_speed(self.speed() * rhs)
    }
}

impl Not for MotorDirection {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Forward(speed) => Self::Backward(speed),
            Self::Backward(speed) => Self::Forward(speed),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Not;

    use speed::{Speed, SpeedControl};

    use crate::MotorDirection;

    /// Verify that the .speed() function returns the correct speed
    #[test]
    fn speed_returns_speed() {
        let speed = Speed::HALF;
        assert_eq!(MotorDirection::Forward(speed).speed(), speed);
    }

    /// Verify that the .with_speed() function changes the speed
    #[test]
    fn with_speed_changes_speed() {
        let new_speed = Speed::MAX;
        let direction = MotorDirection::Forward(Speed::HALF);
        assert_eq!(direction.with_speed(new_speed).speed(), new_speed);
    }

    /// Verify that the [`Not`] implementation returns the opposite direction
    #[test]
    fn not_returns_opposite() {
        let speed = Speed::HALF;
        assert_eq!(
            MotorDirection::Forward(speed).not(),
            MotorDirection::Backward(speed)
        );
    }
}
