use std::fmt::Display;

/// Represent both possible error types from [`Vehicle`]
#[derive(Debug)]
pub enum VehicleError<LE, RE> {
    /// The [`Self::Left`] Error variant
    Left(LE),
    /// The [`Self::Right`] Error variant
    Right(RE),
}

impl<LE, RE> Display for VehicleError<LE, RE>
where
    LE: Display,
    RE: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Left(e) => e.fmt(f),
            Self::Right(e) => e.fmt(f),
        }
    }
}

impl<LE, RE> core::error::Error for VehicleError<LE, RE>
where
    LE: core::error::Error,
    RE: core::error::Error,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Left(e) => e.source(),
            Self::Right(e) => e.source(),
        }
    }
}
