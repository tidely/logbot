//! Follow line endpoint
use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};

use crate::{hardware::Command, state::LogbotState};

use super::HardwareResponse;

/// Rest API endpoint for the [Command::FollowLine] command.
pub async fn post_follow(
    State(state): State<Arc<LogbotState>>,
) -> Result<Json<HardwareResponse>, StatusCode> {
    let response = state
        .hardware
        .send(Command::FollowLine)
        .await
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::debug!("Command response: {:?}", response);
    Ok(Json(HardwareResponse::from(response)))
}
