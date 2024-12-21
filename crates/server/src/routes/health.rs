//! Health check endpoint
use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};

use crate::{routes::HardwareResponse, state::LogbotState};

/// Rest API endpoint for the [Command::Health] command.
pub async fn health(
    State(state): State<Arc<LogbotState>>,
) -> Result<Json<HardwareResponse>, StatusCode> {
    if state.hardware.is_closed() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    Ok(Json(HardwareResponse::new(StatusCode::OK, "Health")))
}
