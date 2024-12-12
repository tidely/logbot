#!/usr/bin/env python3

# Demo for testing sensor output
#
#  With this demo we read sensor output from two channels of a I2C bus.
# We assume the following I2C address (using PCF8591 hardware component)
ADDRESS = 0x48

import smbus2
import time


# Create an I2C bus instance
bus = smbus2.SMBus(1)


# Function to read from the PCF8591
def read_adc(channel):
    if channel < 0 or channel > 3:
        return -1
    bus.write_byte(ADDRESS, 0x40 | channel)  # Set control byte for ADC channel
    bus.read_byte(ADDRESS)  # Dummy read to start ADC conversion
    return bus.read_byte(ADDRESS)  # Read the ADC value

try:
    while True:
        adc0_value = read_adc(0)  # Read from ADC0
        adc1_value = read_adc(1)  # Read from ADC1
        print(f"ADC0: {adc0_value}, ADC1: {adc1_value}")
        time.sleep(1)  # Delay for 1 second
except KeyboardInterrupt:
    print("Program stopped")
