#!/usr/bin/env python3

# This module handles the sensors.

import smbus2
from enum import Enum
from typing import Self
from collections import deque, defaultdict

# QR Code dependencies
from PIL import Image
from io import BytesIO
from pyzbar.pyzbar import decode
from picamera2 import Picamera2, Preview


class Sensors(Enum):
    """Represents the I2c channels of sensors"""
    LEFT = 0
    RIGHT = 1


class I2CSensors:
    """
    Represents two sensors passed by a single I2C connection
    Keeps track of the average over the last 'maxlen' reads
    """

    def __init__(self, address: int = 0x48, maxlen: int = 100):
        self.bus = smbus2.SMBus(1)
        self.address = address
        # Dictionary that maps Sensors to a deque of it's last 'maxlen' values,
        self.averages: defaultdict[Sensors, deque[int]] = defaultdict(lambda: deque(maxlen=maxlen))

    def __enter__(self) -> Self:
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """ Disconnect the i2c connection. """
        self.bus.close()

    def _read(self, channel: int) -> int:
        self.bus.write_byte(self.address, 0x40 | channel)

        # Dummy read to start ADC conversion
        self.bus.read_byte(self.address)
        return self.bus.read_byte(self.address)

    def read(self, sensor: Sensors) -> int:
        """Read the current sensor value (updates averages)"""
        value = self._read(sensor.value)
        self.averages[sensor].append(value)

        return value

    def average(self, sensor: Sensors) -> float:
        """Read the average over the last maxlen reads"""
        assert sensor in self.averages, "Sensor history empty, call .read() first"
        values = self.averages[sensor]
        return sum(values) / len(values)


class Camera:
    def __init__(self):
        self.camera = Picamera2()

        cam_config = self.camera.create_still_configuration(
            main={"size": (1920, 1080)},
            lores={"size": (640, 480)},
            display="lores"
        )

        self.camera.configure(cam_config)

        # The NULL preview is required when GUI is not available
        self.camera.start_preview(Preview.NULL)
        self.camera.start()

    def capture_image(self) -> BytesIO:
        image_data = BytesIO()
        self.camera.capture_file(image_data, format="png")
        return image_data

    def read_qr(self) -> list[str]:
        image = self.capture_image()
        image_obj = Image.open(image)
        decoded_data = decode(image_obj)

        return [code.data.decode() for code in decoded_data]
