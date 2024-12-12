// Should contain logic that's needed to follow a line

use calibration::SensorCalibration;
use directions::{MotorDirection, VehicleDirection};
use speed::Speed;

/// Config for following a line using a single sensor
/// These parameters are not expected to change during a
/// line following 'session'
#[derive(Debug, Clone, Copy)]
pub struct FollowLineConfig {
    /// The default speed at which to follow the line at
    pub default_speed: Speed,
    /// Correction based on current error
    pub proportional: f64,
    /// Correction based on ratio of current and previous error
    pub derivative: f64,
    /// Correction based on all previous errors
    pub integral: Option<f64>,
    /// Calibration data of the sensor we are using for following
    pub calibration: SensorCalibration,
    /// Reset the integral when we hit the target sensor value
    /// This should always be true, since for example if we follow a line
    /// that forms a circle, the integral would creep up until it overpowers
    /// all other values
    pub reset_integral_on_target: bool,
}

/// Follow a line in steps, saves state between calls to [step](Self::step) here.
#[derive(Debug, Copy, Clone)]
pub struct FollowLineState {
    // Static config
    config: FollowLineConfig,
    // These are the values being kept track of
    last_error: f64,
    derivative: f64,
    integral: f64,
}

impl FollowLineState {
    /// Create a new [`FollowLineState`] given a [`FollowLineConfig`]
    pub fn new(config: FollowLineConfig) -> Self {
        Self {
            config,
            last_error: Default::default(),
            derivative: Default::default(),
            integral: Default::default(),
        }
    }

    /// Reset the [`FollowLineState`]
    pub fn reset(&mut self) {
        self.last_error = 0.0;
        self.derivative = 0.0;
        self.integral = 0.0;
    }

    /// Move the line following state forward.
    ///
    /// Takes a new sensor value and calculates a new [`VehicleDirection`]
    pub fn step(&mut self, sensor_value: u8) -> VehicleDirection {
        let error = sensor_value as f64 - self.config.calibration.average();

        self.derivative = error - self.last_error;
        self.last_error = error;

        // To prevent the integral from overpowering steering once the target
        // has been lost for long enough, reset the integral when the error
        // is less than 1.0
        if self.config.reset_integral_on_target && error.abs() < 1.0 {
            self.integral = 0.0;
        } else {
            self.integral += error;
        };

        let mut control =
            self.config.proportional * error + self.config.derivative * self.derivative;

        if let Some(integral_multi) = self.config.integral {
            control += integral_multi * self.integral;
        };

        let mut speed = self.config.default_speed;

        // Enforce that turning is always as strong as it needs to be
        // This means we hope to not saturate values at speed bounds
        let max_speed = self.config.default_speed.value() + control;
        let undershoot = max_speed - Speed::MAX.value();

        if undershoot > 0.0 {
            speed = speed.saturating_sub_f64(undershoot);
        };

        let left = MotorDirection::Forward(speed).wrapping_sub_f64(control);
        let right = MotorDirection::Forward(speed).saturating_add_f64(control);

        VehicleDirection::new(left, right)
    }
}
