//! Health check endpoint
use std::{sync::Arc, time::Instant};

use axum::{extract::State, http::StatusCode};
use tokio::sync::oneshot;

use crate::{hardware::Command, state::LogbotState};

/// Rest API endpoint for the [Command::FollowLine] command.
pub async fn health(State(state): State<Arc<LogbotState>>) -> Result<StatusCode, StatusCode> {
    let (wx, rx) = oneshot::channel();

    let start = Instant::now();
    state
        .hardware
        .send(Command::Health(wx))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    rx.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    tracing::debug!("Health check took: {}ms", start.elapsed().as_millis());

    Ok(StatusCode::OK)
}
