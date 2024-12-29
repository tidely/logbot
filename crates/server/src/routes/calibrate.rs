//! Calibration endpoint
use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};

use crate::{hardware::Command, state::LogbotState};

use super::HardwareResponse;

/// Rest API endpoint for the [Command::Calibrate] command.
pub async fn calibrate(
    State(state): State<Arc<LogbotState>>,
) -> Result<Json<HardwareResponse>, StatusCode> {
    let response = state
        .hardware
        .send(Command::Calibrate)
        .await
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::debug!("Command response: {:?}", response);
    Ok(Json(HardwareResponse::from(response)))
}
