//! Axum server for controlling logbot hardware using a REST-api

use std::sync::Arc;

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use routes::{
    calibrate::calibrate,
    demo::demo,
    edge::find_edge,
    follow::post_follow,
    health::health,
    lift::{lift_down, lift_up},
    stop::stop,
};
use state::LogbotState;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod hardware;
mod routes;
mod state;

/// Logbot REST-api
#[derive(Parser)]
struct Args {
    /// IP Address at which to serve at
    #[clap(default_value = "0.0.0.0:9999")]
    ip: String,
}

/// Entry point for the server
#[tokio::main]
async fn main() -> Result<()> {
    // parse command line arguments
    let args = Args::parse();

    // Setup tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    // bind to a port
    let listener = TcpListener::bind(args.ip).await?;

    // new state
    let state = Arc::new(LogbotState::new()?);

    // create routes
    let router = Router::new()
        .route("/v1/health", get(health))
        .route("/v1/stop", post(stop))
        .route("/v1/demo", post(demo))
        .route("/v1/calibrate", post(calibrate))
        .route("/v1/follow", post(post_follow))
        .route("/v1/edge", post(find_edge))
        .route("/v1/lift/up", post(lift_up))
        .route("/v1/lift/down", post(lift_down))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // serve
    axum::serve(listener, router).await?;

    Ok(())
}
