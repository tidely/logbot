//! Find edge endpoint
use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use tokio::sync::oneshot;

use crate::{hardware::Command, state::LogbotState};

use super::HardwareResponse;

/// Rest API endpoint for the [Command::FindEdge] command.
pub async fn find_edge(
    State(state): State<Arc<LogbotState>>,
) -> Result<Json<HardwareResponse>, StatusCode> {
    let (wx, rx) = oneshot::channel();

    state
        .hardware
        .send((Command::FindEdge, wx))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = rx.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    tracing::debug!("Command response: {:?}", result);

    Ok(Json(HardwareResponse::from(result)))
}
