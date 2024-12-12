#!/usr/bin/env python3

# This module handles the motors.

import time
import logging
from enum import Enum
from typing import Literal, Self
from RPi import GPIO

import const


class MotorDirection(Enum):
    """
    Defines directions motors can take.
    A wheel can only move forwards or backwards.
    """
    FORWARD = 1
    BACKWARD = 2


class BaseMotor:
    """Lowest abstraction over a motor, handles pin setup and shutdown"""

    def __init__(self, power_pin: int, direction_pin: int, pwm_frequency: int):
        """Setup motor pins and pwm"""
        self.power_pin = power_pin
        self.direction_pin = direction_pin
        self.speed: float = 0.0

        assert GPIO.getmode() == GPIO.BCM, "GPIO Mode should be BCM"
        GPIO.setup(power_pin, GPIO.OUT)
        GPIO.setup(direction_pin, GPIO.OUT)

        assert pwm_frequency > 0, "pwm_frequency should be positive"
        self.pwm_frequency = pwm_frequency
        self.pwm = GPIO.PWM(power_pin, pwm_frequency)
        self.pwm.start(self.speed)

    def __enter__(self) -> Self:
        """ Use Motor using a with statement to automatically reset pins """
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """ Reset pins after Motor is dropped """
        logging.debug(
            "cleaning up motor: power %d, direction %d"
            % (self.power_pin, self.direction_pin)
        )
        try:
            self.pwm.stop()
            GPIO.output(self.power_pin, GPIO.LOW)
            GPIO.output(self.direction_pin, GPIO.LOW)
        finally:
            GPIO.cleanup((self.power_pin, self.direction_pin))

    def set_speed(self, speed: float) -> float:
        """
        Set the speed of the motor, speed should be between 0.0 and 100.0
        returns the previous speed
        """
        if speed < 0.0 or speed > 100.0:
            raise ValueError(
                "speed should be between 0.0 and 100.0 (inclusive)"
            )

        previous_speed = self.speed
        self.speed = speed
        self.pwm.ChangeDutyCycle(self.speed)
        return previous_speed

    def stop(self) -> float:
        """Stops the motor, returns the speed of the motor before the stop"""
        return self.set_speed(0.0)


class Motor(BaseMotor):
    """Abstraction for a motor that is used for driving"""

    @staticmethod
    def new_right(power_pin: int, direction_pin: int) -> "Motor":
        """ Create a new Motor instance with the right wheel configuation """
        return Motor(power_pin, direction_pin, GPIO.HIGH, GPIO.LOW)

    @staticmethod
    def new_left(power_pin: int, direction_pin: int) -> "Motor":
        """ Create a new Motor instance with the left wheel configuation """
        return Motor(power_pin, direction_pin, GPIO.LOW, GPIO.HIGH)

    def __init__(
        self,
        power_pin: int,
        direction_pin: int,
        forward: Literal[0, 1],
        backward: Literal[0, 1],
        pwm_frequency: int = const.PWM_FREQUENCY
    ):
        super().__init__(power_pin, direction_pin, pwm_frequency)

        assert forward in (0, 1), "forward needs to be 0 or 1"
        assert backward in (0, 1), "backwards needs to be 0 or 1"
        assert forward != backward, "forward and backward must be different"
        self.forward: Literal[0, 1] = forward
        self.backward: Literal[0, 1] = backward

        # Default direction pin to forward
        self.direction = self.forward
        GPIO.output(self.direction_pin, self.forward)

    def move(self, direction: MotorDirection, speed: float):
        """
        Starts moving to a given MotorDirection.
        Speed can optionally be overwritten.
        (Turns motor on or off)
        """
        self.set_speed(speed)

        match direction:
            case MotorDirection.FORWARD:
                GPIO.output(self.direction_pin, self.forward)
                self.direction = self.forward
            case MotorDirection.BACKWARD:
                GPIO.output(self.direction_pin, self.backward)
                self.direction = self.backward
            case _:
                raise NotImplementedError(
                    f"MotorDirection not covered: {repr(direction)}"
                )


class LiftMotor(BaseMotor):
    """
    Motor that is used for lifting a platform, which uses a normal Motor under
    the hood, keeps track of movement of the lift, allows resetting to the
    inital position on exit
    """

    # The acceptable offset of the motor even after reset in seconds
    acceptable_offset_after_reset_in_seconds: float = 0.1

    def __init__(
        self,
        power_pin: int,
        direction_pin: int,
        reset_on_exit: bool = True,
        pwm_frequency: int = const.PWM_FREQUENCY
    ):
        """ Initialize motor and movement tracking variables """
        super().__init__(power_pin, direction_pin, pwm_frequency)
        self.reset_on_exit = reset_on_exit
        # This is the cumulative movement of the motor in the upwards direction
        # Negative value means the motor has moved down
        self.movement: float = 0

    def __enter__(self) -> Self:
        """ Enter Motor class context manager """
        return super().__enter__()

    def __exit__(self, exc_type, exc_val, exc_tb):
        """
        Exit Motor class context manager
        Resetting lift back to initial state if requested
        """
        try:
            if self.reset_on_exit:
                movement = self.movement
                if movement > 0.0:
                    self.down(movement)
                elif movement < 0.0:
                    self.up(movement)

                # Check that self.movement is within an acceptable range after reset
                if abs(self.movement) > self.acceptable_offset_after_reset_in_seconds:
                    logging.warning(
                        "lift motor is still %ds from inital position after reset"
                        % self.movement
                    )
        finally:
            super().__exit__(exc_type, exc_val, exc_tb)

    def up(self, duration: float = 3.0):
        """Move the lift up for a duration"""
        # Make sure the motor is not moving
        assert self.stop() == 0.0, "lift should not already be moving"

        # Set the direction of the rotation
        GPIO.output(self.direction_pin, GPIO.HIGH)

        # Log start time incase call gets cancelled
        start = time.time()
        self.set_speed(100.0)
        try:
            time.sleep(duration)
        finally:
            self.stop()

            # time.sleep(duration) might get cancelled by an interruption
            # this keeps track of the actual time spent which is used
            # for resetting the pin when the context-manager exists
            duration = time.time() - start
            self.movement += duration

    def down(self, duration: float = 3.0):
        """Move the lift down for a duration"""
        # Make sure the motor is not moving
        assert self.stop() == 0.0, "lift should not already be moving"

        # Set the direction of the rotation
        GPIO.output(self.direction_pin, GPIO.LOW)

        # Log start time incase call gets cancelled
        start = time.time()
        self.set_speed(100.0)
        try:
            time.sleep(duration)
        finally:
            self.stop()

            # time.sleep(duration) might get cancelled by an interruption
            # this keeps track of the actual time spent which is used
            # for resetting the pin when the context-manager exists
            duration = time.time() - start
            self.movement -= duration


class StepperMotor:
    """
    Represents a physical stepper motor, this component is used for precise
    movement of a motor-like turning mechanism
    """

    clockwise: Literal[0, 1] = GPIO.LOW
    counter_clockwise: Literal[0, 1] = GPIO.HIGH
    # The interval at which the steppers state is changed
    interval = 0.0208

    def __init__(self, power_pin: int, direction_pin: int):
        """
        Create new Stepper Motor instance and setup the corresponding pins
        """
        assert GPIO.getmode() == GPIO.BCM, "GPIO Mode should be BCM"

        self.power_pin = power_pin
        self.direction_pin = direction_pin
        self._up: Literal[0, 1] = self.clockwise
        self._down: Literal[0, 1] = self.counter_clockwise

        GPIO.setup(power_pin, GPIO.OUT)
        GPIO.setup(direction_pin, GPIO.OUT)

    def __enter__(self) -> Self:
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """
        Runs after the context-manager is closed.
        Should always put the stepper back to its resting position.
        """
        # TODO: This should only be uncommented once self.down
        # knows to stop after it hits a button
        # self.down()

    def up(self, seconds: int = 4):
        """ Move the stepper upwards """
        # Set the direction
        GPIO.output(self.direction_pin, self._up)

        start = time.time()
        while (time.time() - start) < seconds:
            GPIO.output(self.power_pin, GPIO.HIGH)
            time.sleep(0.0052)
            GPIO.output(self.power_pin, GPIO.LOW)

    # TODO: This should be updated to stop moving down after it
    # hits the button at the bottom
    def down(self, seconds: int = 4):
        """ Move the stepper downwards """
        # Set the direction
        GPIO.output(self.direction_pin, self._down)

        start = time.time()
        while (time.time() - start) < seconds:
            GPIO.output(self.power_pin, GPIO.HIGH)
            time.sleep(0.0052)
            GPIO.output(self.power_pin, GPIO.LOW)




if __name__ == "__main__":
    import time
    GPIO.setmode(GPIO.BCM)

    with Motor.new_right(const.RIGHT_MOTOR_POWER_PIN, const.RIGHT_MOTOR_DIRECTION_PIN) as motor:
        while True:
            motor.move(MotorDirection.FORWARD, speed=50.0)
            time.sleep(1)
            motor.move(MotorDirection.BACKWARD, speed=50.0)
            time.sleep(1)
