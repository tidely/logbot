#!/usr/bin/env python3

# This module is responsible for the main logbot logic.

import time
import logging

from RPi import GPIO

from calibrate import calibrate, CalibratedSensor
import const
from motors import Motor, LiftMotor, MotorDirection
from vehicle import Vehicle, VehicleDirection, SpinDirection
from sensors import I2CSensors, Sensors, Camera
from typing import Optional, Callable

logging.basicConfig(level=logging.DEBUG)

# PID Control Constants
Kp = 1.2  # Proportional gain, adjust as needed
Ki = 0.0  # Intergral gain, adjust as needed
Kd = 0.6  # Derivative gain, adjust as needed


class LogBot:
    """ This is the main class for logbot """

    def __init__(self, vehicle: Vehicle, sensors: I2CSensors):
        self.vehicle = vehicle
        self.sensors = sensors

        self.calibration: Optional[dict[Sensors, CalibratedSensor]] = None

    def follow_line_until(
        self,
        until: Callable[[], bool],
        timeout: Optional[int] = None
    ):
        """
        Follow line until function `until` returns False or timeout is met.
        """
        if timeout is not None:
            assert timeout >= 0, "timeout can't be negative"
            end_time = time.time() + timeout

        while not until():
            if self.sensors.read(Sensors.RIGHT) > const.SENSOR_THRESHOLD_RIGHT:
                self.vehicle.move(VehicleDirection.RIGHT)
            elif self.sensors.read(Sensors.LEFT) > const.SENSOR_THRESHOLD_LEFT:
                self.vehicle.move(VehicleDirection.LEFT)
            else:
                self.vehicle.move(VehicleDirection.FORWARD)

            if timeout is not None and time.time() > end_time:
                break

        self.vehicle.stop()

    def follow_pd_until(self, until: Callable[[], bool]):
        """Follows line using PD control until callable returns True"""

        last_error = 0
        base_speed = self.vehicle.speed  # Base speed of the vehicle

        while not until():
            # Read sensor values
            sensor_left = self.sensors.read(Sensors.LEFT)
            sensor_right = self.sensors.read(Sensors.RIGHT)

            # Calculate the error
            error = sensor_left - sensor_right

            # Calculate the derivative of the error
            derivative = error - last_error
            last_error = error

            # PD control output
            control = Kp * error + Kd * derivative

            # Adjust motor speeds based on control output
            left_speed = base_speed - control
            right_speed = base_speed + control

            # Clamp speed
            left_speed = max(0, min(100, left_speed))
            right_speed = max(0, min(100, right_speed))

            # Move the vehicle with adjusted speeds
            # TODO: Since we call motors directly, we might want to get rid of
            # the vehicle abstraction
            self.vehicle.left.move(MotorDirection.FORWARD, left_speed)
            self.vehicle.right.move(MotorDirection.FORWARD, right_speed)

        self.vehicle.stop()

    def oscillate_until(
        self,
        until: Callable[[], bool],
        timeout: float,
        speed: float = 20.0
    ) -> bool:
        """
        Oscillate until a condition or the timeout is met

        returns whether the condition was met
        """
        assert timeout >= 0, "Timeout should not be negative"
        assert 100 >= speed >= 0, "Speed should be between 0-100 (inclusive)"
        start = time.monotonic()

        direction = SpinDirection.LEFT  # Current direction we are spinning in

        duration: int = 1  # Duration of the current rotation
        duration_start = time.monotonic()  # Timestamp of spin start

        # Start spinning
        self.vehicle.spin(direction, speed=speed)

        while not until():
            timestamp = time.monotonic()

            if (timestamp - start) > timeout:
                self.vehicle.stop()
                return False

            # Swap spin direction at intervals
            if (timestamp - duration_start) > duration:
                duration *= 2
                duration_start = timestamp
                match direction:
                    case SpinDirection.RIGHT:
                        direction = SpinDirection.LEFT
                    case SpinDirection.LEFT:
                        direction = SpinDirection.RIGHT
                    case _:
                        raise NotImplementedError()
                self.vehicle.spin(direction, speed=speed)

        self.vehicle.stop()
        return True

    def find_edge(
        self,
        sensor: Sensors,
        timeout: float = 20,
        threshold: float = 5.0,
        calibration: Optional[CalibratedSensor] = None
    ) -> bool:
        """ Find edge of the line using a given sensor """
        assert timeout >= 0, "timeout should not be negative"

        if calibration is None:
            if self.calibration is None:
                raise ValueError("No calibration data provided")
            calibration = self.calibration[sensor]

        return self.oscillate_until(
            until=lambda: abs(self.sensors.read(sensor) - calibration.line) < threshold,
            timeout=timeout,
        )

    def follow_line_single_sensor_until(
        self,
        sensor: Sensors,
        until: Callable[[], bool],
        acceleration_time: float = 2.0,
        integral: bool = True,
        dynamic_speed: bool = True,
        calibration: Optional[CalibratedSensor] = None,
    ):
        """
        Follow line using a single sensor, until a callable returns true.
        Supports PD and PID steering and acceleration.
        Uses self.vehicle.default_speed for its base speed.

        sensor: Which sensor to use for following, must be calibrated
        until: Function that returns True when we should stop
        acceleration_time: How long we should take accelerating
        integral: Whether to use the integral (PD vs PID)
        dynamic_speed: Whether to dynamically adjust speed based on error
        calibration: Overwrite calibration data manually
        """
        assert acceleration_time >= 0, "Acceleration should not be negative"

        if calibration is None:
            if self.calibration is None or sensor not in self.calibration:
                raise ValueError("No calibration data provided")
            calibration = self.calibration[sensor]

        assert calibration.line > calibration.floor, "this function assumes 'line > floor'"

        last_error = 0.0
        integral_value = 0.0

        # Calculate estimated max error based on calibration
        max_error = calibration.average() - min(calibration.line, calibration.floor)

        start = time.monotonic()

        while not until():
            # Read sensor values
            sensor_left = self.sensors.read(sensor)

            # Calculate the error
            error = sensor_left - calibration.average()

            # Calculate the derivative of the error
            derivative = error - last_error
            last_error = error

            # PD control output
            control = Kp * error + Kd * derivative

            # Add calculate and add integral if requested
            if integral:
                integral_value += error
                control += integral_value * Ki

            # Get vehicle speed on each iteration to allow dynamic speed
            # changes from outside of the function
            dynamic_base_speed = self.vehicle.default_speed

            # Adjust speed based on error (slows down on large error)
            if dynamic_speed:
                error_ratio = abs(error) / max_error
                multiplier =  1 - max(0.5, min(1, error_ratio))
                dynamic_base_speed *= multiplier

            # Adjust motor speeds based on control output
            left_speed = dynamic_base_speed - control
            right_speed = dynamic_base_speed + control

            # Reduce speed to mimic acceleration when applicable
            if acceleration_time > 0:
                multiplier = min(1, (time.monotonic() - start) / acceleration_time)
                left_speed *= multiplier
                right_speed *= multiplier

            # Clamp speed
            left_speed = max(0, min(100, left_speed))
            right_speed = max(0, min(100, right_speed))

            # Move the vehicle with adjusted speeds
            # TODO: Since we call motors directly, we might want to get rid of
            # the vehicle abstraction
            self.vehicle.left.move(MotorDirection.FORWARD, left_speed)
            self.vehicle.right.move(MotorDirection.FORWARD, right_speed)

        self.vehicle.stop()
        logging.debug(
            "Followed line for %d seconds" % round(time.time() - start, 2)
        )

    def detect_stop_line(
        self,
        left_fallback: float = const.SENSOR_THRESHOLD_LEFT,
        right_fallback: float = const.SENSOR_THRESHOLD_RIGHT,
    ) -> bool:
        """ Return true once a stop line is detected """
        if self.calibration is not None:
            left = (
                self.calibration[Sensors.LEFT].line
                if Sensors.LEFT in self.calibration
                else left_fallback
            )
            right = (
                self.calibration[Sensors.RIGHT].line
                if Sensors.RIGHT in self.calibration
                else right_fallback
            )
        else:
            left = left_fallback
            right = right_fallback

        return (
            self.sensors.read(Sensors.LEFT) > left
            and self.sensors.read(Sensors.RIGHT) > right
        )

    def turn_until_line(
        self,
        direction: SpinDirection,
        speed: float,
        initial_spin: float = 1.25,
        calibration: Optional[CalibratedSensor] = None,
    ):
        """
        Turn logbot until a line is detected

        direction: Direction in which to spin to
        speed: Optional overwrite for default speed
        initial_spin: Seconds to spin before checking for line
        calibration: Allow overwriting sensor calibration data
        """
        assert speed >= 0, "speed should not be negative"
        assert initial_spin >= 0, "initial spin should not be negative"

        match direction:
            case SpinDirection.LEFT:
                sensor = Sensors.LEFT
            case SpinDirection.RIGHT:
                sensor = Sensors.RIGHT
            case _:
                raise NotImplementedError()

        if calibration is None:
            if self.calibration is None or sensor not in self.calibration:
                raise ValueError("No calibration data provided")
            calibration = self.calibration[sensor]

        assert calibration.line > calibration.floor, "this function assumes 'line > floor'"
        threshold = calibration.average()

        # Start spinning for initial duration
        self.vehicle.spin(direction, speed)
        time.sleep(initial_spin)

        # Wait until line is detected, and wait until it passes
        while True:
            if self.sensors.read(sensor) > threshold:
                while self.sensors.read(sensor) > threshold:
                    time.sleep(0.01)
                break

        self.vehicle.stop()


def main():
    with (
        Motor.new_right(
            const.RIGHT_MOTOR_POWER_PIN, const.RIGHT_MOTOR_DIRECTION_PIN
        ) as right_motor,
        Motor.new_left(
            const.LEFT_MOTOR_POWER_PIN, const.LEFT_MOTOR_DIRECTION_PIN
        ) as left_motor,
        LiftMotor(
            const.LIFT_POWER_PIN, const.LIFT_DIRECTION_PIN
        ) as lift_motor,
    ):
        sensors = I2CSensors()
        camera = Camera()
        logbot = LogBot(Vehicle(left_motor, right_motor, default_speed=80), sensors)

        # Hardcoded calibration data for left sensor
        calibration = CalibratedSensor(210, 160)

        logbot.calibration = calibrate(logbot, (Sensors.LEFT, Sensors.RIGHT), speed=30.0)
        assert logbot.find_edge(Sensors.LEFT), "Didn't find the edge"

        logbot.follow_line_single_sensor_until(
            sensor=Sensors.LEFT,
            until=lambda: False,
            acceleration_time=0,
            integral=False,
            dynamic_speed=False,
        )
        return

        logbot.follow_pd_until(logbot.detect_stop_line)
        time.sleep(0.5)
        lift_motor.up()

        logbot.turn_until_line(SpinDirection.LEFT)
        time.sleep(0.5)
        logbot.follow_pd_until(logbot.detect_stop_line)
        logbot.vehicle.move(VehicleDirection.FORWARD)
        time.sleep(0.5)
        logbot.follow_pd_until(logbot.detect_stop_line)

        lift_motor.down()
        logbot.vehicle.move(VehicleDirection.BACKWARD)
        time.sleep(2.5)
        logbot.turn_until_line(SpinDirection.LEFT)
        time.sleep(0.3)

        # Starts to fetch next package (skips twice)
        logbot.follow_pd_until(logbot.detect_stop_line)
        logbot.vehicle.move(VehicleDirection.FORWARD)
        time.sleep(0.5)
        logbot.follow_pd_until(logbot.detect_stop_line)
        logbot.vehicle.move(VehicleDirection.FORWARD)
        time.sleep(0.3)
        logbot.follow_pd_until(logbot.detect_stop_line)
        lift_motor.up()

        logbot.turn_until_line(SpinDirection.LEFT)
        time.sleep(0.3)
        logbot.follow_pd_until(logbot.detect_stop_line)
        logbot.vehicle.move(VehicleDirection.FORWARD)
        time.sleep(0.3)
        logbot.follow_pd_until(logbot.detect_stop_line)

        lift_motor.down()


if __name__ == '__main__':
    GPIO.setmode(GPIO.BCM)

    try:
        main()
    except KeyboardInterrupt:
        pass
    finally:
        GPIO.cleanup()
