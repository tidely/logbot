use anyhow::Result;

use tokio::sync::mpsc;

use crate::hardware::{spawn_default, Command};

/// Store api global state
#[derive(Debug)]
pub struct LogbotState {
    /// Channel for sending hardware commands
    pub hardware: mpsc::Sender<Command>,
}

impl LogbotState {
    pub fn new() -> Result<Self> {
        let channel = spawn_default()?;
        Ok(Self { hardware: channel })
    }
}
