use std::fmt::Display;

/// Geneirc Error for interacting with a Vehicle and Sensors at once
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VehicleSensorError<VE, SE> {
    /// Vehicle error variant
    Vehicle(VE),
    /// Sensor error variant
    Sensor(SE),
}

/// Geneic Error for the demo
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DemoError<VE, SE, LE> {
    /// Vehicle error variant
    Vehicle(VE),
    /// Sensor error variant
    Sensor(SE),
    /// Lift error variant
    Lift(LE),
}

impl<VE, SE, LE> From<VehicleSensorError<VE, SE>> for DemoError<VE, SE, LE> {
    fn from(value: VehicleSensorError<VE, SE>) -> Self {
        match value {
            VehicleSensorError::Vehicle(e) => Self::Vehicle(e),
            VehicleSensorError::Sensor(e) => Self::Sensor(e),
        }
    }
}

impl<VE, SE, LE> Display for DemoError<VE, SE, LE>
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

impl<VE, SE, LE> std::error::Error for DemoError<VE, SE, LE>
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
