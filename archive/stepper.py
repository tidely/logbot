#!/usr/bin/env python3

"""
This demo is for spinning a Stepper motor clickwise and counter-clockwise
"""

import time
from enum import Enum
from typing import Union

from RPi import GPIO


class Direction(Enum):
    """Enum for directions in which a stepper can rotate"""
    CLOCKWISE = GPIO.LOW
    COUNTERCLOCKWISE = GPIO.HIGH


def spin(power_pin: int, direction_pin: int, direction: Direction, length_seconds: float, interval: float = 0.0208):
    """Rotate Stepper in a given direction for n seconds"""
    assert GPIO.getmode() == GPIO.BCM, "GPIO mode should be BCM"

    # Set the direction pin
    GPIO.output(direction_pin, direction.value)

    # Log the start time
    start = time.monotonic()

    # Rotate until we reach timeout
    while (time.monotonic() - start) < length_seconds:
        GPIO.output(power_pin, GPIO.HIGH)
        time.sleep(interval)
        GPIO.output(power_pin, GPIO.LOW)


if __name__ == "__main__":
    GPIO.setmode(GPIO.BCM)

    # Raspberry Pi GPIO Pins in our example
    POWER_PIN = 24
    DIRECTION_PIN = 26

    # Setup pins
    GPIO.setup(24, GPIO.OUT)
    GPIO.setup(26, GPIO.OUT)

    try:
        spin(POWER_PIN, DIRECTION_PIN, Direction.CLOCKWISE, length_seconds=3.0)
        spin(POWER_PIN, DIRECTION_PIN, Direction.COUNTERCLOCKWISE, length_seconds=3.0)
    except KeyboardInterrupt:
        pass
    finally:
        GPIO.cleanup()
