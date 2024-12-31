//! Define a generic error for the logbot struct

use std::fmt::Display;

/// Generic Logbot Error
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogbotError<VE, SE, LE> {
    /// Vehicle error variant
    Vehicle(VE),
    /// Sensor error variant
    Sensor(SE),
    /// Lift error variant
    Lift(LE),
}

impl<VE, SE, LE> Display for LogbotError<VE, SE, LE>
where
    VE: Display,
    SE: Display,
    LE: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vehicle(e) => e.fmt(f),
            Self::Sensor(e) => e.fmt(f),
            Self::Lift(e) => e.fmt(f),
        }
    }
}

impl<VE, SE, LE> std::error::Error for LogbotError<VE, SE, LE>
where
    VE: std::error::Error,
    SE: std::error::Error,
    LE: std::error::Error,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Vehicle(e) => e.source(),
            Self::Sensor(e) => e.source(),
            Self::Lift(e) => e.source(),
        }
    }
}
