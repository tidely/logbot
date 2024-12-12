use interfaces::{SensorRead, ToSensorChannel};
use rppal::i2c::{self, I2c};

/// Sensor Controller that allows fetching state from multiple sensors
///
/// [`SensorController`] is actually a Analog Digital Converter (ADC) and a
/// Digital Analog Converter (DAC) in one. The hardware component represented
/// is the Adafruit PCF8591 Quad 8-bit ADC/DAC. We use a [`I2c`] bus for communication.
/// However we use it strictly for interfacing with a sensor array.
#[derive(Debug)]
pub struct SensorController {
    i2c: I2c,
}

impl SensorController {
    /// Create a new [`SensorController`] from a [`I2c`] bus
    pub fn new(i2c: I2c) -> Self {
        Self { i2c }
    }
}

impl SensorRead for SensorController {
    type Output = u8;
    type Error = i2c::Error;

    /// Read a value from a sensor
    fn read(&mut self, sensor: impl ToSensorChannel) -> Result<Self::Output, Self::Error> {
        let channel = sensor.to_channel();
        let control_byte = 0x40 | channel;
        self.i2c.write(&[control_byte])?;

        // Dummy read to trigger ADC conversion
        self.i2c.read(&mut [0])?;

        // Read the ADC value
        let mut buffer = [0];
        self.i2c.read(&mut buffer)?;
        Ok(buffer[0])
    }
}
