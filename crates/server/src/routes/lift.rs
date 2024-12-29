//! Lift endpoint
use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};

use crate::{hardware::Command, state::LogbotState};

use super::HardwareResponse;

/// Rest API endpoint for the [Command::LiftUp] command.
pub async fn lift_up(
    State(state): State<Arc<LogbotState>>,
) -> Result<Json<HardwareResponse>, StatusCode> {
    let response = state
        .hardware
        .send(Command::LiftUp)
        .await
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::debug!("Command response: {:?}", response);
    Ok(Json(HardwareResponse::from(response)))
}

/// Rest API endpoint for the [Command::LiftDown] command.
pub async fn lift_down(
    State(state): State<Arc<LogbotState>>,
) -> Result<Json<HardwareResponse>, StatusCode> {
    let response = state
        .hardware
        .send(Command::LiftDown)
        .await
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::debug!("Command response: {:?}", response);
    Ok(Json(HardwareResponse::from(response)))
}
