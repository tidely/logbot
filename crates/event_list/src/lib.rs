//! Event List
//!
//! Event list is used to track sequences of events. These sequences are stored in a vec
//! which can be searched using binary search to find the time of any event ocourrance

use std::{
    ops::Deref,
    time::{Duration, Instant},
};

/// Represents an event currently in progress.
#[derive(Debug, Clone, Copy)]
pub struct ActiveEvent<T> {
    /// Store the event data
    pub data: T,
    /// [`Instant`] at which the event started
    pub start_time: Instant,
}

impl<T> ActiveEvent<T> {
    /// Complete the [`ActiveEvent`] now
    pub fn complete_now(self) -> CompletedEvent<T> {
        self.complete_at(Instant::now())
    }

    /// Complete the active event at a given Instant
    pub fn complete_at(self, end: Instant) -> CompletedEvent<T> {
        CompletedEvent {
            data: self.data,
            elapsed_time: end.duration_since(self.start_time),
        }
    }
}

impl<T> From<T> for ActiveEvent<T> {
    fn from(value: T) -> Self {
        Self {
            data: value,
            start_time: Instant::now(),
        }
    }
}

/// Represents a completed event with its data and the duration it took.
#[derive(Debug, Clone, Copy)]
pub struct CompletedEvent<T> {
    /// Store the event data
    pub data: T,
    /// Duration of the event
    pub elapsed_time: Duration,
}

impl<T> From<ActiveEvent<T>> for CompletedEvent<T> {
    fn from(value: ActiveEvent<T>) -> Self {
        Self {
            data: value.data,
            elapsed_time: value.start_time.elapsed(),
        }
    }
}

/// A contiguous sequence of events that have a [`Duration`] and a start time
/// has an optional end time
#[derive(Debug, Clone)]
pub struct TimedSequence<T> {
    /// A Vec of [`CompletedEvent`]'s
    values: Vec<CompletedEvent<T>>,
    /// The start time of the first event
    pub start: Instant,
    /// Optional end time of the sequence
    pub end: Option<Instant>,
}

impl<T> TimedSequence<T> {
    /// Create a new [`TimedSequence`]
    pub fn new(value: CompletedEvent<T>, start: Instant) -> Self {
        Self {
            values: vec![value],
            start,
            end: None,
        }
    }

    /// Total [`Duration`] of the sequence, if self.end is set
    pub fn duration(&self) -> Option<Duration> {
        self.end.map(|v| v.duration_since(self.start))
    }

    /// Complete the sequence by replacing [end](Self::end) with an [`Instant`]
    pub fn complete(&mut self, end: Instant) -> Option<Instant> {
        self.end.replace(end)
    }
}

/// Manages events organized by their completion times.
#[derive(Debug, Clone)]
pub struct EventList<T> {
    completed_events: Vec<TimedSequence<T>>,
    active_event: Option<ActiveEvent<T>>,
}

impl<T> Default for EventList<T> {
    fn default() -> Self {
        Self {
            completed_events: Vec::new(),
            active_event: None,
        }
    }
}

impl<T> Deref for EventList<T> {
    type Target = Vec<TimedSequence<T>>;

    fn deref(&self) -> &Self::Target {
        &self.completed_events
    }
}

impl<T> EventList<T> {
    /// Add a new event to the list
    pub fn push(&mut self, value: T) {
        let event = ActiveEvent::from(value);

        // Replace active event and push possible old event onto the structure
        if let Some(previous_event) = self.active_event.replace(event) {
            let start_time = previous_event.start_time;
            let completed = previous_event.complete_now();

            match self.completed_events.last_mut() {
                // Append previous event
                Some(last) => {
                    last.values.push(completed);
                }
                // First event in the tree, create new sequence
                None => {
                    let sequence = TimedSequence::new(completed, start_time);
                    self.completed_events.push(sequence);
                }
            }
        }
    }

    /// The current [`ActiveEvent`]
    pub fn active_event(&self) -> &Option<ActiveEvent<T>> {
        &self.active_event
    }

    /// Completes the current [`ActiveEvent`] if it exists
    /// Creates a new [`TimedSequence`]
    pub fn complete(&mut self) -> bool {
        let now = Instant::now();
        if let Some(event) = self.active_event.take() {
            let start_time = event.start_time;
            let completed = event.complete_at(now);

            // Update end time of last sequence
            if let Some(last) = self.completed_events.last_mut() {
                last.complete(now);
            };

            // Insert new sequence
            let sequence = TimedSequence::new(completed, start_time);
            self.completed_events.push(sequence);

            true
        } else {
            false
        }
    }

    /// The total number of completed sequences
    pub fn total_completed_sequences(&self) -> usize {
        let mut len = self.completed_events.len();
        // Reduce by one incase last sequence in tree has not completed
        if self
            .completed_events
            .last()
            .is_some_and(|v| v.end.is_none())
        {
            len -= 1;
        };
        len
    }

    /// Total number of events in the tree, excluding any [`ActiveEvent`]
    pub fn total_events_len(&self) -> usize {
        self.completed_events.iter().map(|v| v.values.len()).sum()
    }
}
