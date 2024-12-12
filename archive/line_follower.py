#!/usr/bin/env python3

# Demo for following the line
#
# We use a simple HIGH LOW signals for speed and direction.
#
# For hardware we assume the below pin layout for the motors,
# and a I2C bus at address 0x11 which connects to a 5 IR Sensor array
# where each sensor returns a value between 0 and 256

import smbus2
from RPi import GPIO

### SET PINS ###
MC1PWM = 13
MC2PWM = 12
MC1DIR = 6
MC2DIR = 5
################

# Motor direction for forward
LeftMotorDir = GPIO.HIGH
RightMotorDir = GPIO.LOW


def reset_pins():
    GPIO.output(MC1PWM, GPIO.LOW)
    GPIO.output(MC2PWM, GPIO.LOW)
    GPIO.output(MC1DIR, GPIO.LOW)
    GPIO.output(MC2DIR, GPIO.LOW)


def setup_pins():
    GPIO.setmode(GPIO.BCM)
    GPIO.setup(MC1PWM, GPIO.OUT)  # MC1 PWM
    GPIO.setup(MC2PWM, GPIO.OUT)  # MC2 PWM
    GPIO.setup(MC1DIR, GPIO.OUT)  # MC1 DIR
    GPIO.setup(MC2DIR, GPIO.OUT)  # MC2 DIR


class Sensor:
    def __init__(self, address: int = 0x11):
        self.bus = smbus2.SMBus(1)
        self.address = address

    def read_raw(self):
        result = None
        for _ in range(0, 5):
            try:
                result = self.bus.read_i2c_block_data(self.address, 0, 10)
                break
            except Exception:
                pass

        return result

    def read_analog(self, trys: int = 5):
        for _ in range(trys):
            raw_result = self.read_raw()
            if raw_result:
                analog_result = [0, 0, 0, 0, 0]
                for i in range(0, 5):
                    high_byte = raw_result[i*2] << 8
                    low_byte = raw_result[i*2+1]
                    analog_result[i] = high_byte + low_byte
                    if analog_result[i] > 1024:
                        continue
                return analog_result
        else:
            raise IOError("Line follower read error. Please check the wiring.")


if __name__ == "__main__":
    try:
        setup_pins()
        reset_pins()

        sensor = Sensor()
        last_successful_read = [0, 0, 0, 0, 0]

        # TURN ON MOTORS
        GPIO.output(MC1PWM, GPIO.HIGH)
        GPIO.output(MC2PWM, GPIO.HIGH)

        # SET DIRECTION
        GPIO.output(MC1DIR, LeftMotorDir)
        GPIO.output(MC2DIR, RightMotorDir)

        while True:
            if (read := sensor.read_analog()) is not None:
                last_successful_read = read

            # (LEFT SENSOR, RIGHT SENSOR)
            simple_read = (last_successful_read[0], last_successful_read[-1])

            line_on_right_side = simple_read[0] / simple_read[-1] < 0.5
            line_on_left_side = simple_read[-1] / simple_read[0] < 0.5

            print(line_on_right_side, line_on_left_side)

            if line_on_right_side:
                # Turn off right motor
                GPIO.output(MC2PWM, GPIO.LOW)
            elif line_on_left_side:
                # Turn off left motor
                GPIO.output(MC1PWM, GPIO.LOW)
            else:
                # Set both motors to on again
                GPIO.output(MC1PWM, GPIO.HIGH)
                GPIO.output(MC2PWM, GPIO.HIGH)

            print(last_successful_read)
    finally:
        GPIO.cleanup()
