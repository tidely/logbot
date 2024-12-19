//! Crate for adding linear acceleration to a Speed

use std::ops::Mul;

use directions::Stop;
use speed::Speed;

mod linear;
pub use linear::LinearAcceleration;

mod sineinout;
pub use sineinout::SineInOutAcceleration;

/// Trait for defining a [`Accelerator`]
pub trait Accelerator<S>
where
    S: Mul<Speed, Output = S> + Stop + Sized,
{
    /// Apply acceleration to a type
    fn apply(&mut self, value: S) -> S;
}

/// Trait for applying acceleration to a type
pub trait Accelerate
where
    Self: Mul<Speed, Output = Self> + Stop + Sized,
{
    /// Apply acceleration to a type
    fn accelerate(self, acceleration: &mut impl Accelerator<Self>) -> Self {
        acceleration.apply(self)
    }
}

/// Implement Accelerate for all types which satisfy trait bounds
impl<T> Accelerate for T where T: Mul<Speed, Output = Self> + Stop + Sized {}
