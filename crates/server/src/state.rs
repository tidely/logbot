use anyhow::Result;

use components::{hardware_pwm::DCMotor, software_pwm::LiftMotor, Left, Right, SensorController};
use defaults::TryDefault;
use logbot::Logbot;
use vehicle::Vehicle;

use crate::hardware::HardwareThread;

/// Global state for the Logbot API
#[derive(Debug)]
pub struct LogbotState {
    /// Thread for processing hardware commands
    pub hardware:
        HardwareThread<Logbot<Vehicle<DCMotor<Left>, DCMotor<Right>>, SensorController, LiftMotor>>,
}

impl LogbotState {
    pub fn new() -> Result<Self> {
        let logbot = Logbot::new(
            Vehicle::try_default()?,
            SensorController::try_default()?,
            LiftMotor::try_default()?,
        );
        let thread = HardwareThread::spawn(logbot);

        Ok(Self { hardware: thread })
    }
}
