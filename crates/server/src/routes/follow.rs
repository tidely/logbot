//! Follow line endpoint
use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use tokio::sync::oneshot;

use crate::{hardware::Command, state::LogbotState};

use super::HardwareResponse;

/// Rest API endpoint for the [Command::FollowLine] command.
pub async fn post_follow(
    State(state): State<Arc<LogbotState>>,
) -> Result<Json<HardwareResponse>, StatusCode> {
    let (wx, rx) = oneshot::channel();

    state
        .hardware
        .send(Command::FollowLine(wx))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = rx.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    tracing::debug!("Command response: {:?}", result);

    Ok(Json(HardwareResponse::from(result)))
}
