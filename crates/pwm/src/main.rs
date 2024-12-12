//! Calibrate PWM pulse width
//!
//! DC Motors control their speed by having a constant pwm which they consider
//! the stop of the motor. To activate these kinds of motors, you first need to
//! set the correct pulse width for the pwm. However manufactured parts do not
//! always adhere to their specification, meaning that the stop pulse width
//! might vary across different motors, even though their model is the same.
//! This crate allows you to find the pulse width at which the motor actually
//! activates. Allowing you to use it in the future as a hardcoded value. The
//! current implementation is targeted towards our specific hardware component,
//! however it can be adapted to any motor by tweaking the given const values.

use std::time::Duration;

use anyhow::Result;

use clap::{Parser, ValueEnum};

use rppal::{
    gpio::Gpio,
    pwm::{Channel, Pwm},
};

/// Command Line Arguments for PWM Calibration
#[derive(Parser)]
struct Args {
    /// The pin which to test PWM with
    pin: u8,
    /// Pick software or hardware PWM
    #[clap(value_enum)]
    pwm: PWMVariant,
}

/// Allow specifying the PWM variant (hardware or software)
#[derive(ValueEnum, Clone, Debug)]
enum PWMVariant {
    /// Hardware PWM
    Hardware,
    /// Software PWM
    Software,
}

/// Convert a Raspberry Pi Pin to it's corresponding [`Channel`]
fn pin_to_channel(pin: u8) -> Result<Channel> {
    match pin {
        12 | 18 => Ok(Channel::Pwm0),
        13 | 19 => Ok(Channel::Pwm1),
        _ => Err(anyhow::anyhow!(
            "Pin `{}` doesn't support hardware PWM",
            pin
        )),
    }
}

/// Pwm period to use
const PERIOD: Duration = Duration::from_millis(20);

/// Minimum pulse with to test
const START_DURATION: Duration = Duration::from_micros(1_400);

/// Maximum pulse width to test
const STOP_DURATION: Duration = Duration::from_micros(1_600);

/// Amount to increase pulse width on each step
const STEP: Duration = Duration::from_micros(10);

/// [`Duration`] what to keep the current pulse width for
const INTERVAL: Duration = Duration::from_secs(1);

/// Calibrate Software pwm stop pulse width
fn software_pwm(pin: u8) -> Result<()> {
    let mut pin = Gpio::new()?.get(pin)?.into_output_low();

    let mut width = START_DURATION;

    while width <= STOP_DURATION {
        pin.set_pwm(PERIOD, width)?;
        println!("Pulse width: {}", width.as_micros());
        // Wait for INTERVAL and update pulse width
        std::thread::sleep(INTERVAL);
        width += STEP;
    }

    Ok(())
}

/// Calibrate Hardware [`Pwm`] stop pulse width
fn hardware_pwm(channel: Channel) -> Result<()> {
    let pwm = Pwm::new(channel)?;
    pwm.set_period(PERIOD)?;
    pwm.enable()?;

    let mut width = START_DURATION;

    while width <= STOP_DURATION {
        pwm.set_pulse_width(width)?;
        println!("Pulse width: {}", width.as_micros());
        // Wait for INTERVAL and update pulse width
        std::thread::sleep(INTERVAL);
        width += STEP;
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.pwm {
        PWMVariant::Hardware => {
            // Convert pin to a hardware pwm channel
            let channel = pin_to_channel(args.pin)?;
            hardware_pwm(channel)?;
        }
        PWMVariant::Software => {
            software_pwm(args.pin)?;
        }
    };
    Ok(())
}
