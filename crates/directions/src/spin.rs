use std::ops::{Mul, Not};

use speed::Speed;

use crate::{SpeedControl, Stop};

/// Directions in which a Vehicle can spin in-place
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpinDirection {
    /// Left spin
    Left(Speed),
    /// Right spin
    Right(Speed),
}

impl SpeedControl for SpinDirection {
    fn speed(&self) -> Speed {
        match self {
            Self::Left(speed) => *speed,
            Self::Right(speed) => *speed,
        }
    }

    fn with_speed(self, speed: Speed) -> Self {
        match self {
            Self::Left(_) => Self::Left(speed),
            Self::Right(_) => Self::Right(speed),
        }
    }
}

impl Stop for SpinDirection {
    fn is_stop(&self) -> bool {
        self.speed().value() == 0.0
    }
}

impl Mul<Speed> for SpinDirection {
    type Output = Self;

    fn mul(self, rhs: Speed) -> Self::Output {
        self.with_speed(self.speed() * rhs)
    }
}

impl Not for SpinDirection {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Left(speed) => Self::Right(speed),
            Self::Right(speed) => Self::Left(speed),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Not;

    use speed::Speed;

    use super::SpinDirection;

    /// Test that the [`Not`] trait returns the opposite [`SpinDirection`]
    /// while preserving [`Speed`]
    #[test]
    fn not_returns_opposite() {
        let speed = Speed::HALF;
        let direction = SpinDirection::Right(speed);
        assert_eq!(direction.not(), SpinDirection::Left(speed));
    }
}
