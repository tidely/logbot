//! Utilities for dealing with tracking and finding a line
//!
//! This crate provides implementations for line following and other helpful
//! functions interacting with a line of the floor

mod follow;

pub use follow::{FollowLineConfig, FollowLineState};
