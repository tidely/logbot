//! Defines different routes for controlling logbot
use axum::http::StatusCode;
use serde::Serialize;

use crate::hardware::{CalibrateResult, DemoResult, FindEdgeResult, FollowLineResult, StopResult};

pub mod calibrate;
pub mod demo;
pub mod edge;
pub mod follow;
pub mod health;
pub mod stop;

/// [`Serialize`] hardware responses using serde
#[derive(Serialize)]
pub struct HardwareResponse {
    status: u16,
    reason: Option<String>,
}

impl HardwareResponse {
    /// Create a new HardwareResponse
    fn new(status: StatusCode, reason: Option<String>) -> Self {
        Self {
            status: status.as_u16(),
            reason,
        }
    }
}

impl From<CalibrateResult> for HardwareResponse {
    fn from(value: CalibrateResult) -> Self {
        match value {
            CalibrateResult::Success => Self::new(StatusCode::OK, None),
            CalibrateResult::AlreadyCalibrating => Self::new(StatusCode::FOUND, None),
            CalibrateResult::FindingEdge => {
                Self::new(StatusCode::CONFLICT, Some("EDGE".to_string()))
            }
            CalibrateResult::Following => {
                Self::new(StatusCode::CONFLICT, Some("FOLLOWING".to_string()))
            }
        }
    }
}

impl From<StopResult> for HardwareResponse {
    fn from(value: StopResult) -> Self {
        match value {
            StopResult::Nothing => Self::new(StatusCode::OK, None),
            StopResult::Calibrate => Self::new(StatusCode::OK, Some("CALIBRATE".to_string())),
            StopResult::FindEdge => Self::new(StatusCode::OK, Some("EDGE".to_string())),
            StopResult::FollowLine => Self::new(StatusCode::OK, Some("FOLLOW".to_string())),
        }
    }
}

impl From<FollowLineResult> for HardwareResponse {
    fn from(value: FollowLineResult) -> Self {
        match value {
            FollowLineResult::Success => Self::new(StatusCode::OK, None),
            FollowLineResult::AlreadyFollowing => Self::new(StatusCode::FOUND, None),
            FollowLineResult::NotOnTheLine => Self::new(
                StatusCode::METHOD_NOT_ALLOWED,
                Some("NOTONLINE".to_string()),
            ),
            FollowLineResult::FindingEdge => {
                Self::new(StatusCode::CONFLICT, Some("EDGE".to_string()))
            }
            FollowLineResult::MidCalibration => {
                Self::new(StatusCode::CONFLICT, Some("CALIBRATE".to_string()))
            }
            FollowLineResult::NoCalibration => Self::new(
                StatusCode::METHOD_NOT_ALLOWED,
                Some("NOCALIBRATION".to_string()),
            ),
        }
    }
}

impl From<FindEdgeResult> for HardwareResponse {
    fn from(value: FindEdgeResult) -> Self {
        match value {
            FindEdgeResult::Success => Self::new(StatusCode::OK, None),
            FindEdgeResult::AlreadyEdging => Self::new(StatusCode::FOUND, None),
            FindEdgeResult::NoCalibration => Self::new(
                StatusCode::METHOD_NOT_ALLOWED,
                Some("NOCALIBRATIONO".to_string()),
            ),
            FindEdgeResult::Follow => Self::new(StatusCode::CONFLICT, Some("FOLLOW".to_string())),
            FindEdgeResult::Calibrate => {
                Self::new(StatusCode::CONFLICT, Some("CALIBRATE".to_string()))
            }
        }
    }
}

impl From<DemoResult> for HardwareResponse {
    fn from(value: DemoResult) -> Self {
        match value {
            DemoResult::Success => Self::new(StatusCode::OK, None),
        }
    }
}
