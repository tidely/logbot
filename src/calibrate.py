import time
import random
import logging
from collections import Counter, defaultdict
from typing import NamedTuple, TYPE_CHECKING, Optional, Sequence

from sensors import Sensors
from vehicle import SpinDirection

if TYPE_CHECKING:
    from logbot import LogBot


class CalibratedSensor(NamedTuple):
    line: float
    floor: float

    def average(self) -> float:
        """
        Get the average of the line and floor.
        This operation is so common, lets have a function for it.
        """
        return (self.line + self.floor) / 2


def find_two_values_kmeans(
    data: list[int],
    max_iterations: int = 100,
    tolerance: float = 1e-4
) -> CalibratedSensor:
    """Find floor and line values using kmeans clustering"""
    assert len(data) > 1, "there need to be atleast 2 sensor readings"
    assert max(data) <= 255 and min(data) >= 0, "Sensor data should be between 0-255"

    # Step 1: Initialize two cluster centers randomly from the data
    cluster_centers: list[float] = random.sample(list(set(data)), 2)
    cluster_centers.sort()

    for _ in range(max_iterations):
        clusters: dict[int, list[float]] = {
            0: [], 1: []
        }

        # Step 2: Assign data points to the nearest cluster center
        for value in data:
            distances = [abs(value - center) for center in cluster_centers]
            cluster_index = distances.index(min(distances))
            clusters[cluster_index].append(value)

        prev_cluster_centers = cluster_centers.copy()

        # Step 3: Update cluster centers
        for i in range(2):
            if clusters[i]:
                cluster_centers[i] = sum(clusters[i]) / len(clusters[i])
            else:
                # Reinitialize the cluster center if it has no data points
                cluster_centers[i] = random.choice(data)

        # Check for convergence
        shifts = [
            abs(prev_cluster_centers[i] - cluster_centers[i])
            for i in range(2)
        ]
        if max(shifts) < tolerance:
            break

    # Floor and line value
    return CalibratedSensor(max(cluster_centers), min(cluster_centers))


def find_two_values_historgram(
    data: list[int],
    gap: float = 10.0
) -> CalibratedSensor:
    """ Seperate floor and line values by using a histogram """
    assert len(data) > 1, "there need to be atleast 2 sensor readings"
    assert max(data) <= 255 and min(data) >= 0, "Sensor data should be between 0-255"
    assert gap > 0, "gap must be larger than 0"

    # Step 1: Create a histogram of the sensor values
    value_counts = Counter(data)
    sorted_values = value_counts.most_common()

    logging.debug(
        "calibration: found %d unique values in sensor data"
        % len(value_counts)
    )

    # The most common value is always either the floor or the line
    most_common = sorted_values[0][0]

    # Step 2: Find a threshold that separates the two peaks
    # This can be done by identifying a "gap" in the sorted values
    threshold = None
    for i in range(1, len(sorted_values)):
        if abs(sorted_values[i][0] - most_common) > gap:
            threshold = (sorted_values[i][0] + most_common) / 2
            logging.debug("calibration: gap found at index %d " % (i - 1))
            logging.debug("calibration: threshold set at %d" % threshold)
            break

    if threshold is None:
        logging.warning("calibration: no gap found in data, using fallback")
        threshold = max(data) + min(data) / 2  # Fallback if no gap is found

    # Step 3: Separate the values into floor and line based on the threshold
    floor_values = [value for value in data if value < threshold]
    line_values = [value for value in data if value >= threshold]
    logging.debug("calibration: found %d floor values" % len(floor_values))
    logging.debug("calibration: found %d line vales" % len(line_values))

    # Step 4: Calculate the mean values
    try:
        floor_value = sum(floor_values) / len(floor_values)
        line_value = sum(line_values) / len(line_values)
    except ZeroDivisionError:
        raise ZeroDivisionError("LogBot didn't detect a line during calibration")

    logging.debug("calibration: using floor value %d" % floor_value)
    logging.debug("calibration: using line value %d" % line_value)

    return CalibratedSensor(line_value, floor_value)


def calibrate(
    logbot: 'LogBot',
    sensors: Sequence[Sensors] = (Sensors.LEFT, Sensors.RIGHT),
    speed: Optional[float] = None
) -> dict[Sensors, CalibratedSensor]:
    """
    Calibrate the logbot and determine the sensor value for being inbetween
    the line and the floor, this value is used to steer the logbot using a
    single sensor by tracking the edge of the line.
    """
    if speed is None:
        speed = logbot.vehicle.default_speed

    # Map of values
    values: defaultdict[Sensors, list[int]] = defaultdict(list)

    # Turn the logbot in-place in both directions and record sensor values
    spin_range_in_seconds = 3
    logging.debug(
        "calibration: setting spin range to %d seconds"
        % spin_range_in_seconds
    )

    # Turn to start reading from the left side
    logbot.vehicle.spin(SpinDirection.LEFT, speed=speed)
    time.sleep(spin_range_in_seconds / 2)
    logbot.vehicle.stop()

    # Record sensor values while turning from left to right
    start = time.time()
    logbot.vehicle.spin(SpinDirection.RIGHT, speed=speed)

    while (time.time() - start) < spin_range_in_seconds:
        for sensor in sensors:
            values[sensor].append(logbot.sensors.read(sensor))

    logbot.vehicle.stop()

    # Turn back to initial position
    logbot.vehicle.spin(SpinDirection.LEFT, speed=speed)
    time.sleep(spin_range_in_seconds / 2)
    logbot.vehicle.stop()

    for sensor in sensors:
        logging.debug(
            "calibration: recorded %d values from '%s' sensor"
            % (len(values[sensor]), repr(sensor))
        )

    # Map dict values from sensor data to CalibratedSensor
    return {k: find_two_values_kmeans(v) for k, v in values.items() }
