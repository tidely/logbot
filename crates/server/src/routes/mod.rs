//! Defines different routes for controlling logbot
use axum::http::StatusCode;
use serde::Serialize;

use crate::hardware::{CommandDenied, CommandResult};

pub mod calibrate;
pub mod demo;
pub mod edge;
pub mod follow;
pub mod health;
pub mod lift;
pub mod stop;

/// [`Serialize`] hardware responses using serde
#[derive(Serialize)]
pub struct HardwareResponse {
    status: u16,
    reason: &'static str,
}

impl HardwareResponse {
    /// Create a new HardwareResponse
    fn new(status: StatusCode, reason: &'static str) -> Self {
        Self {
            status: status.as_u16(),
            reason,
        }
    }
}

impl From<CommandResult> for HardwareResponse {
    fn from(value: CommandResult) -> Self {
        match value {
            Ok(command) => Self::new(StatusCode::OK, command.as_str()),
            Err(CommandDenied::Busy(busy)) => Self::new(StatusCode::CONFLICT, busy.as_str()),
            Err(CommandDenied::Required(required)) => {
                Self::new(StatusCode::FORBIDDEN, required.as_str())
            }
        }
    }
}
