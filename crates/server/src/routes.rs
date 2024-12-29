use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;

use crate::{
    hardware::{Command, CommandDenied, CommandResult},
    state::LogbotState,
};

/// Macro for generating a route handler for a given [`Command`]
macro_rules! command_route {
    ($fn_name:ident, $command_variant:expr) => {
        pub async fn $fn_name(
            State(state): State<Arc<LogbotState>>,
        ) -> Result<Json<HardwareResponse>, StatusCode> {
            let response = state
                .hardware
                .send($command_variant)
                .await
                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

            tracing::debug!("Command response: {:?}", response);
            Ok(Json(HardwareResponse::from(response)))
        }
    };
}

// Generate routes for Commands
command_route!(stop, Command::Stop);
command_route!(calibrate, Command::Calibrate);
command_route!(find_edge, Command::FindEdge);
command_route!(follow, Command::FollowLine);
command_route!(demo, Command::Demo);
command_route!(lift_up, Command::LiftUp);
command_route!(lift_down, Command::LiftDown);

/// Rest API endpoint for [`Command::Health`]
pub async fn health(
    State(state): State<Arc<LogbotState>>,
) -> Result<Json<HardwareResponse>, StatusCode> {
    if state.hardware.is_finished() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    Ok(Json(HardwareResponse::new(StatusCode::OK, "Health")))
}

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
