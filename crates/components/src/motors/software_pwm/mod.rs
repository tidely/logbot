//! Implementations of software PWM Motors

mod dcmotor;
mod lift;
mod signed;

pub use dcmotor::DCMotor;
pub use lift::LiftMotor;
pub use signed::SignedMotor;
