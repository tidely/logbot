#!/usr/bin/env python3

# Calculate stepper rotation rate
#
# This demo allows calculating the rpm of an encoder.
# For hardware we assume pin at two encoders
ENCODER_A_PIN = 22
ENCODER_B_PIN = 27

import RPi.GPIO as GPIO
import time

# Setup GPIO
GPIO.setmode(GPIO.BCM)
GPIO.setup(ENCODER_A_PIN, GPIO.IN, pull_up_down=GPIO.PUD_UP)
GPIO.setup(ENCODER_B_PIN, GPIO.IN, pull_up_down=GPIO.PUD_UP)

# Variables for counting pulses
pulse_count = 0
pulses_per_revolution = 20  # Set this to the number of pulses per full rotation of the encoder

# Previous states
previous_state_a = GPIO.input(ENCODER_A_PIN)
previous_state_b = GPIO.input(ENCODER_B_PIN)

def count_pulses():
    global pulse_count, previous_state_a, previous_state_b

    # Read current states
    current_state_a = GPIO.input(ENCODER_A_PIN)
    current_state_b = GPIO.input(ENCODER_B_PIN)

    # Check for a rising edge on A
    if previous_state_a == 0 and current_state_a == 1:
        # Determine the direction based on B's state
        if current_state_b == 0:
            pulse_count += 1  # Forward rotation
        else:
            pulse_count -= 1  # Reverse rotation

    # Update previous states
    previous_state_a = current_state_a
    previous_state_b = current_state_b

try:
    while True:
        pulse_count = 0  # Reset pulse count
        start_time = time.time()

        # Measure pulses for 1 second
        while time.time() - start_time < 1:
            count_pulses()  # Continuously check the encoder state

        # Calculate RPM
        rpm = (pulse_count / pulses_per_revolution) * 60
        print(f"RPM: {rpm}")

except KeyboardInterrupt:
    print("Stopped by User")

finally:
    GPIO.cleanup()
