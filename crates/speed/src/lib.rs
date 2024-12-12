//! Speed abstraction
//!
//! [`Speed`] is a wrapper around a [`f64`], which has its bounds set at 0.0 and 1.0
//! This is used to enforce limits when setting the speed of the PWM Duty cycle

use core::num::NonZero;
use core::ops::{Div, Mul};

/// Trait for allowing types to change their [`Speed`]
pub trait SpeedControl {
    /// Get the ascossiated [`Speed`] of the type
    fn speed(&self) -> Speed;

    /// Change the ascossiated [`Speed`] of the type by consuming Self
    fn with_speed(self, speed: Speed) -> Self;
}

/// Represent Speed
///
/// [`Speed`] is a simple wrapper around the [`f64`] type.
/// It's used to enforce that the underlying value is between
/// 0.0 and 1.0 (inclusive)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Speed(f64);

impl Speed {
    /// The minumum possible [`Speed`]
    pub const MIN: Self = Self(0.0);

    /// Half of the maximum possible [`Speed`]
    pub const HALF: Self = Self(0.5);

    /// The maxiumum possible [`Speed`]
    pub const MAX: Self = Self(1.0);

    /// Create a new [`Speed`] value, this returns an error if the value does not
    /// respect the bounds of [`Speed`] (0.0 to 1.0)
    pub fn new(value: f64) -> Result<Self, f64> {
        Self::try_from(value)
    }

    /// Create a new [`Speed`], clamping to stay in bounds
    pub fn new_clamp(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// SAFETY: value must be between 0.0 and 1.0 (inclusive)
    pub const unsafe fn new_unchecked(value: f64) -> Self {
        Self(value)
    }

    /// Get the underlying [`f64`] value
    pub fn value(self) -> f64 {
        self.0
    }
}

// Implement saturating operations with f64
impl Speed {
    /// Saturating float addition.
    /// Computes self.0 + rhs, saturating at the [`Speed`] bounds instead of overflowing.
    pub fn saturating_add_f64(&self, value: f64) -> Self {
        Self::new_clamp(self.0 + value)
    }

    /// Saturating float subtraction.
    /// Computes self.0 - rhs, saturating at the [`Speed`] bounds instead of underflowing.
    pub fn saturating_sub_f64(&self, value: f64) -> Self {
        Self::new_clamp(self.0 - value)
    }

    /// Saturating float multiplication.
    /// Computes self.0 * rhs, saturating at the [`Speed`] bounds.
    pub fn saturating_mul_f64(&self, value: f64) -> Self {
        Self::new_clamp(self.0 * value)
    }

    /// Saturating float division.
    /// Computes self.0 / rhs, saturating at the [`Speed`] bounds.
    pub fn saturating_div_f64(&self, value: f64) -> Self {
        Self::new_clamp(self.0 / value)
    }
}

// Implement saturating operations with another Speed
impl Speed {
    /// Saturating addition.
    /// Computes self.0 + other.0, saturating at the [`Speed`] bounds instead of overflowing.
    pub fn saturating_add(&self, other: Self) -> Self {
        self.saturating_add_f64(other.value())
    }

    /// Saturating subtraction.
    /// Computes self.0 - other.0, saturating at the [`Speed`] bounds instead of overflowing.
    pub fn saturating_sub(&self, other: Self) -> Self {
        self.saturating_sub_f64(other.value())
    }
}

/// [`Speed`] is a [`f64`] from 0.0 to 1.0 -> This means multiplying the value of
/// speed with another returns a new value still within bounds.
impl Mul for Speed {
    type Output = Speed;

    fn mul(self, rhs: Self) -> Self::Output {
        // Speed is always between 0.0 and 1.0, which means multiplying
        // will result in a value also between 0.0 and 1.0
        Self(self.0 * rhs.0)
    }
}

impl TryFrom<f64> for Speed {
    type Error = f64;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(value)
        }
    }
}

/// This macro implements the [`Div`] trait for a type wrapped in [`NonZero`]
macro_rules! impl_div_for_nonzero {
    ($($ty:ty),*) => {
        $(
            /// Since any number divided by a number larger than 1 only gets smaller.
            /// But also stays above 0.0
            impl Div<NonZero<$ty>> for Speed {
                type Output = Speed;

                fn div(self, rhs: NonZero<$ty>) -> Self::Output {
                    Self(self.0 / rhs.get() as f64)
                }
            }
        )*
    };
}

// Implement the Div op for all unsigned integers wrapper in NonZero.
impl_div_for_nonzero!(usize, u8, u16, u32, u64, u128);

#[cfg(test)]
mod tests {
    use crate::Speed;

    /// Test that [Speed::new] preserves the passed [`f64`] as the speed
    #[test]
    fn new_preserves_value() {
        let value: f64 = 0.53;
        let speed = Speed::new(value).unwrap();
        assert_eq!(speed.value(), value);
    }

    /// Test that new works when a valid value is passed
    #[test]
    fn new_works_with_valid() {
        let valid = 0.3;
        assert!(Speed::new(valid).is_ok())
    }

    /// Test that new returns an error on a invalid value
    /// and that the error returns the [`f64`] back to the caller
    #[test]
    fn new_errors_on_invalid() {
        // Test a too large value
        let too_large = 1.1;
        assert_eq!(Speed::new(too_large), Err(too_large));

        // Test a too small value
        let too_small = -3.0;
        assert_eq!(Speed::new(too_small), Err(too_small));
    }

    /// Test that clamp correctly clamps values to [`Speed`] bounds
    #[test]
    fn new_clamp_works() {
        // Test clamp at over 1.0
        assert_eq!(Speed::new_clamp(1.1).value(), 1.0);
        // Test clamp at under 0.0
        assert_eq!(Speed::new_clamp(-1.0).value(), 0.0);
    }

    /// Test that [`Speed::MIN`] and [`Speed::MAX`] are actually at [`Speed`] bounds
    #[test]
    fn min_max_are_at_bounds() {
        let min_speed = Speed::new_clamp(f64::MIN);
        assert_eq!(Speed::MIN, min_speed);

        let max_speed = Speed::new_clamp(f64::MAX);
        assert_eq!(Speed::MAX, max_speed);
    }

    /// Test that [`Speed::HALF`] is halfway between [`Speed::MAX`] and [`Speed::MIN`]
    #[test]
    fn half_in_middle() {
        let middle = (Speed::MIN.value() + Speed::MAX.value()) / 2.0;
        assert_eq!(Speed::HALF, Speed::new_clamp(middle));
    }

    /// Test that multiplying [`Speed`] with another keeps the resulting
    /// [`Speed`] in bounds
    #[test]
    fn mul_stays_in_bounds() {
        let min = Speed::MIN * Speed::MIN;
        assert_eq!(min, Speed::new_clamp(min.value()));

        let max = Speed::MAX * Speed::MAX;
        assert_eq!(max, Speed::new_clamp(max.value()));
    }

    /// Check that the [`PartialEq`] implementation correctly takes the [Speed::0] value
    /// into account
    #[test]
    fn partial_eq_test() {
        let value = 0.5;
        let value2 = 0.6;
        // Check that Speed with the same value are equal
        assert_eq!(Speed::new_clamp(value), Speed::new_clamp(value));

        // Check that Speed with different values are not equal
        assert_ne!(Speed::new_clamp(value), Speed::new_clamp(value2));
    }
}
