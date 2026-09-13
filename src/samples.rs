//! Bounded diagnostic history in a single process/session's monotonic clock.
use crate::reader::{Observation, ReadError};
use std::collections::VecDeque;
use std::time::Duration;

pub const FRESHNESS: Duration = Duration::from_millis(100);
pub const HISTORY_CAPACITY: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Present,
    Absent,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct Sample {
    pub started: Duration,
    pub finished: Duration,
    pub result: Result<Observation, ReadError>,
}

impl Sample {
    pub fn state(&self) -> State {
        match &self.result {
            Ok(value) if value.candidate_present() => State::Present,
            Ok(_) => State::Absent,
            Err(_) => State::Unknown,
        }
    }
}

#[derive(Default)]
pub struct History {
    pub latest: Option<Sample>,
    /// Only transitions are retained; these are diagnostic observations, not
    /// completed deflect windows or input counts.
    pub transitions: VecDeque<(Duration, State)>,
}

impl History {
    pub fn last_present_duration(&self, now: Duration) -> Option<Duration> {
        let mut latest = None;
        for (index, (started, state)) in self.transitions.iter().enumerate() {
            if *state != State::Present {
                continue;
            }
            let end = self.transitions.get(index + 1).map(|(at, _)| *at);
            latest = Some(end.map_or_else(
                || now.saturating_sub(*started),
                |at| at.saturating_sub(*started),
            ));
        }
        latest
    }

    fn transition(&mut self, at: Duration, state: State) {
        if self.transitions.back().is_none_or(|(_, old)| *old != state) {
            if self.transitions.len() == HISTORY_CAPACITY {
                self.transitions.pop_front();
            }
            self.transitions.push_back((at, state));
        }
    }

    pub fn push(&mut self, sample: Sample) -> bool {
        if sample.finished < sample.started {
            return false;
        }
        if let Some(previous) = &self.latest {
            if sample.started <= previous.finished {
                return false;
            }
            let expires = previous.started + FRESHNESS;
            let player_changed = match (&previous.result, &sample.result) {
                (Ok(a), Ok(b)) => a.player != b.player,
                _ => false,
            };
            if sample.started >= expires {
                self.transition(expires, State::Unknown);
            } else if player_changed {
                self.transition(sample.started, State::Unknown);
            }
        }
        self.transition(sample.finished, sample.state());
        self.latest = Some(sample);
        true
    }

    pub fn state(&self, now: Duration) -> State {
        self.latest.as_ref().map_or(State::Unknown, |sample| {
            if now < sample.finished || now.saturating_sub(sample.started) >= FRESHNESS {
                State::Unknown
            } else {
                sample.state()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample(ms: u64, result: Result<Observation, ReadError>) -> Sample {
        Sample {
            started: Duration::from_millis(ms),
            finished: Duration::from_millis(ms + 1),
            result,
        }
    }
    fn active() -> Result<Observation, ReadError> {
        Ok(Observation {
            player: 0x30000,
            effects: vec![105010],
        })
    }
    #[test]
    fn expires_at_limit_and_failure_replaces_known_state() {
        let mut h = History::default();
        h.push(sample(0, active()));
        assert_eq!(h.state(Duration::from_millis(99)), State::Present);
        assert_eq!(h.state(Duration::from_millis(100)), State::Unknown);
        assert_eq!(h.state(Duration::from_millis(101)), State::Unknown);
        h.push(sample(102, Err(ReadError::Unreadable)));
        assert_eq!(h.state(Duration::from_millis(103)), State::Unknown);
    }
    #[test]
    fn gaps_are_preserved_and_disordered_samples_rejected() {
        let mut h = History::default();
        assert!(h.push(sample(1, active())));
        assert!(!h.push(sample(1, active())));
        assert!(h.push(sample(201, active())));
        assert_eq!(
            h.transitions.iter().map(|(_, s)| *s).collect::<Vec<_>>(),
            vec![State::Present, State::Unknown, State::Present]
        );
    }
    #[test]
    fn history_is_bounded_and_new_sessions_are_empty() {
        let mut h = History::default();
        for i in 0..1000 {
            h.push(sample(
                i * 8,
                if i % 2 == 0 {
                    active()
                } else {
                    Err(ReadError::MissingPlayer)
                },
            ));
        }
        assert_eq!(h.transitions.len(), HISTORY_CAPACITY);
        assert_eq!(History::default().state(Duration::ZERO), State::Unknown);
    }

    #[test]
    fn changing_player_inserts_unknown_boundary() {
        let mut h = History::default();
        h.push(sample(1, active()));
        let mut next = active().unwrap();
        next.player += 0x1000;
        h.push(sample(9, Ok(next)));
        assert_eq!(
            h.transitions.iter().map(|(_, s)| *s).collect::<Vec<_>>(),
            vec![State::Present, State::Unknown, State::Present]
        );
    }

    #[test]
    fn reports_completed_present_duration() {
        let mut h = History::default();
        h.push(sample(1, active()));
        h.push(sample(
            50,
            Ok(Observation {
                player: 0x30000,
                effects: vec![],
            }),
        ));
        assert_eq!(
            h.last_present_duration(Duration::from_millis(200)),
            Some(Duration::from_millis(49))
        );
    }
}
