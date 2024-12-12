//! Crate for calibrating sensors based on recorded data

// struct that borrows a sensor
// possibly Sensor<'a> that borrows from the IRSensor struct
// implements kmeans clustering
// and returns a/multiple struct(s) of line and floor values
//
// should use kmeans clustering (https://docs.rs/kmeans/latest/kmeans/)

mod kmeans;
use kmeans::{average_cluster_sizes, kmeans};

/// Log sensor values to calibrate a sensor
#[derive(Debug, Default)]
pub struct SingleSensorCalibration {
    data: Vec<f64>,
}

impl SingleSensorCalibration {
    /// Log a value to the calibration
    pub fn log(&mut self, value: f64) {
        self.data.push(value);
    }

    /// Generate a [`SensorCalibration`] from the recorded values
    ///
    /// This uses kmeans clustering to find 2 clusters, these are then used to calculate the average for each
    /// Which we then return as a [`SensorCalibration`].
    /// The larger average is used as the [line](SensorCalibration::line),
    /// the smaller as the [floor](SensorCalibration::floor)
    pub fn calibrate(self) -> SensorCalibration {
        let assignments = kmeans(self.data.as_slice(), 2, 100);
        let averages = average_cluster_sizes(self.data.as_slice(), assignments.as_slice(), 2);

        let min = averages
            .iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let max = averages
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        // TODO: remove this
        dbg!(min);
        dbg!(max);

        SensorCalibration::new(max as u8, min as u8)
    }
}

/// The end result of calibrating a sensor
#[derive(Debug, Clone, Copy)]
pub struct SensorCalibration {
    /// The sensor value of the line
    pub line: u8,
    /// The sensor value of the floor
    pub floor: u8,
}

impl SensorCalibration {
    /// Create a new [`SensorCalibration`]
    pub fn new(line: u8, floor: u8) -> Self {
        Self { line, floor }
    }

    /// Get the average between [line](SensorCalibration::line) and [floor](SensorCalibration::floor)
    pub fn average(&self) -> f64 {
        (self.line as f64 + self.floor as f64) / 2.0
    }
}
