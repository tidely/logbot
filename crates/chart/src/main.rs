//! Create terminal chart from sensor values

use std::{collections::VecDeque, io::Stdout, time::Duration};

use anyhow::Result;
use components::SensorController;
use consts::Sensors;
use defaults::TryDefault;
use interfaces::SensorRead;
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    prelude::CrosstermBackend,
    style::Style,
    symbols,
    widgets::{Axis, Block, Chart, Dataset},
    Terminal,
};

/// How many values to show on the chart
const HISTORY_SIZE: usize = 256;

/// How often the sensors are polled for values
const INTERVAL: Duration = Duration::from_millis(1);

/// Produce a live [`Chart`] of sensor events to the terminal
fn chart(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    sensors: &mut SensorController,
) -> Result<()> {
    // Store values from the left sensor
    let mut left_history = VecDeque::with_capacity(HISTORY_SIZE);
    // Store values from the right sensor
    let mut right_history = VecDeque::with_capacity(HISTORY_SIZE);

    loop {
        // Read new values from the sensors
        let left = sensors.read(Sensors::Left)?;
        let right = sensors.read(Sensors::Right)?;

        // Ensure that history stays below history length
        // after inserting new value
        if left_history.len() == right_history.capacity() {
            left_history.pop_front();
        };
        left_history.push_back(left);

        // Do the same for the right history
        if right_history.len() == right_history.capacity() {
            right_history.pop_front();
        };
        right_history.push_back(right);

        // Check if the user wants to exit using Esc
        if event::poll(Duration::ZERO)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Esc {
                    return Ok(());
                }
            }
        }

        // Draw new graph to terminal
        terminal.draw(|frame| {
            // Create Datasets from history
            let left_data: Vec<(f64, f64)> = left_history
                .iter()
                .enumerate()
                .map(|(i, v)| (i as f64, v.clone() as f64))
                .collect();
            let left_dataset = Dataset::default()
                .name("Left Sensor")
                .marker(symbols::Marker::Block)
                .style(Style::default().fg(ratatui::style::Color::Red))
                .data(&left_data);
            let right_data: Vec<(f64, f64)> = right_history
                .iter()
                .enumerate()
                .map(|(i, v)| (i as f64, v.clone() as f64))
                .collect();
            let right_dataset = Dataset::default()
                .name("Right Sensor")
                .marker(symbols::Marker::Block)
                .style(Style::default().fg(ratatui::style::Color::Green))
                .data(&right_data);

            // Labels for sensor values on the y-axis
            let labels = [
                "0", "16", "32", "48", "64", "80", "96", "112", "128", "144", "160", "176", "192",
                "208", "224", "250", "256",
            ];

            // Generate chart for the datasets
            let chart = Chart::new(vec![left_dataset, right_dataset])
                .block(Block::bordered())
                .x_axis(
                    Axis::default()
                        .title("Time")
                        .style(Style::default().fg(ratatui::style::Color::Magenta))
                        .bounds([0.0, HISTORY_SIZE as f64]),
                )
                .y_axis(
                    Axis::default()
                        .title("Average")
                        .style(Style::default().fg(ratatui::style::Color::Magenta))
                        .bounds([0.0, 256.0])
                        .labels(labels),
                );

            // Render chart to terminal
            frame.render_widget(chart, frame.area());
        })?;

        // Sleep for the given interval
        std::thread::sleep(INTERVAL);
    }
}

/// Entrypoint for the `chart` binary
fn main() -> Result<()> {
    // Setup hardware
    let mut controller = SensorController::try_default()?;

    // Setup terminal
    let mut terminal = ratatui::init();

    // Produce a live chart of sensor events
    let result = chart(&mut terminal, &mut controller);

    // Restore terminal to original state
    ratatui::restore();

    result
}
