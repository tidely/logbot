from enum import Enum
from typing import Optional

from motors import Motor, MotorDirection


class SpinDirection(Enum):
    """
    Defines different directions a vehicle can spin
    """
    LEFT = 3
    RIGHT = 4


class VehicleDirection(Enum):
    """
    Defines directions a vehicle can take. All four directions.
    Values are tuples, where the left item is the MotorDirection for
    the left Motor and the right item is the MotorDirection for the right Motor
    """
    FORWARD = MotorDirection.FORWARD
    BACKWARD = MotorDirection.BACKWARD
    LEFT = SpinDirection.LEFT
    RIGHT = SpinDirection.RIGHT

# Assert that all variants in VehicleDirection are unique
assert len(set(VehicleDirection.__members__.values())) == 4


class Vehicle:
    """ This class represents a Vehicle. In this case with 2 motors. """

    def __init__(self, left: Motor, right: Motor, default_speed: float):
        self.left = left
        self.right = right
        self.speed = 0.0
        self.default_speed = default_speed

    def move(
        self,
        direction: VehicleDirection,
        speed: Optional[float] = None
    ) -> float:
        """
        Move the vehicle in a direction, allows overwriting current speed

        direction: VehicleDirection in which to steer the vehicle to
        speed: Allow overwriting the current default speed for this move call

        return: Returns the speed of the vehicle before this call
        """
        previous_speed = self.speed
        self.speed = self.default_speed if speed is None else speed

        match direction:
            case VehicleDirection.FORWARD:
                self.left.move(MotorDirection.FORWARD, self.speed)
                self.right.move(MotorDirection.FORWARD, self.speed)
            case VehicleDirection.BACKWARD:
                self.left.move(MotorDirection.BACKWARD, self.speed)
                self.right.move(MotorDirection.BACKWARD, self.speed)
            case VehicleDirection.LEFT:
                self.left.move(MotorDirection.FORWARD, self.speed / 3)
                self.right.move(MotorDirection.FORWARD, self.speed)
            case VehicleDirection.RIGHT:
                self.left.move(MotorDirection.FORWARD, self.speed)
                self.right.move(MotorDirection.FORWARD, self.speed / 3)
            case _:
                raise NotImplementedError(
                    f"VehicleDirection not covered: {repr(direction)}"
                )

        return previous_speed


    def spin(
        self,
        direction: SpinDirection,
        speed: Optional[float] = None
    ) -> float:
        """
        Spin in a given direction

        direction: SpinDirection in which to spin to
        speed: Optional speed to overwrite the default speed

        return: Returns the speed of the vehicle before this call
        """
        previous_speed = self.speed
        self.speed = self.default_speed if speed is None else speed

        match direction:
            case SpinDirection.LEFT:
                self.left.move(MotorDirection.BACKWARD, self.speed)
                self.right.move(MotorDirection.FORWARD, self.speed)
            case SpinDirection.RIGHT:
                self.left.move(MotorDirection.FORWARD, self.speed)
                self.right.move(MotorDirection.BACKWARD, self.speed)
            case _:
                raise NotImplementedError(
                    f"SpinDirection not covered: {repr(direction)}"
                )

        return previous_speed

    def stop(self) -> float:
        """
        Stop the vehicle

        return: Returns the speed of the vehicle before the stop
        """
        previous_speed = self.speed
        self.speed = 0
        self.left.stop()
        self.right.stop()
        return previous_speed
