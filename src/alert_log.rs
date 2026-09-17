//! Sparse draw-submission ledger: every decision transition plus a heartbeat.
//! Complements the bounded full-frame log; does not assert visible presentation.
use crate::{cue::Response, timing::State};
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Key {
    pub handle: u32,
    pub model: i32,
    pub npc_param: Option<i32>,
    pub animation: i32,
    pub occurrence: u64,
    pub phase: Option<u32>,
    pub state: State,
    pub response: Response,
    pub status: &'static str,
    pub submitted: bool,
    pub generation: u64,
}

#[derive(Default)]
pub struct Sampler {
    previous: Option<Key>,
    at: Duration,
}

impl Sampler {
    pub fn record(&mut self, at: Duration, key: Key) -> bool {
        if self.previous.as_ref() != Some(&key)
            || at < self.at
            || at.saturating_sub(self.at) >= Duration::from_secs(1)
        {
            self.previous = Some(key);
            self.at = at;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> Key {
        Key {
            handle: 1,
            model: 1010,
            npc_param: Some(10100200),
            animation: 200003008,
            occurrence: 1,
            phase: Some(0),
            state: State::Incoming,
            response: Response::Mikiri,
            status: "incoming_windup",
            submitted: true,
            generation: 1,
        }
    }

    #[test]
    fn keeps_short_attacks_cancellations_variant_changes_and_later_session_heartbeats() {
        let mut sampler = Sampler::default();
        assert!(sampler.record(Duration::ZERO, key()));
        assert!(!sampler.record(Duration::from_millis(16), key()));
        let mut active = key();
        active.state = State::AttackActive;
        assert!(sampler.record(Duration::from_millis(32), active.clone()));
        active.phase = Some(1);
        assert!(sampler.record(Duration::from_millis(48), active.clone()));
        active.npc_param = Some(10100000);
        assert!(sampler.record(Duration::from_millis(64), active.clone()));
        active.submitted = false;
        active.state = State::Hidden;
        assert!(sampler.record(Duration::from_millis(80), active.clone()));
        assert!(sampler.record(Duration::from_secs(60 * 60 * 4), active.clone()));
        assert!(!sampler.record(
            Duration::from_secs(60 * 60 * 4) + Duration::from_millis(900),
            active.clone()
        ));
        assert!(sampler.record(Duration::from_secs(60 * 60 * 4 + 1), active));
    }
}
