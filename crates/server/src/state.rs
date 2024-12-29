use anyhow::Result;

use defaults::TryDefault;

use crate::hardware::{Hardware, HardwareThread};

/// Global state for the Logbot API
#[derive(Debug)]
pub struct LogbotState {
    /// Thread for handling [`Hardware`] components
    pub hardware: HardwareThread,
}

impl LogbotState {
    pub fn new() -> Result<Self> {
        let hardware = Hardware::try_default()?;
        let thread = HardwareThread::spawn(hardware);

        Ok(Self { hardware: thread })
    }
}
