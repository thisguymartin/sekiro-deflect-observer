//! Completed animation-event batches and timeline crossings, not contact results.
use crate::cue::{decode_animation, Animation};
use crate::reader::ReadError;

pub const BATCH_BYTES: usize = 224;

/// Select within one completed engine batch. Never search preceding history.
/// Fixed-size input makes the game-thread capture allocation-free.
pub fn select_batch(raw: &[u8; BATCH_BYTES], model: i32) -> Result<Animation, ReadError> {
    let head = i32::from_le_bytes(raw[200..204].try_into().unwrap());
    let begin = i32::from_le_bytes(raw[204..208].try_into().unwrap());
    if !(0..10).contains(&head) || !(0..10).contains(&begin) {
        return Err(ReadError::InvalidAnimation);
    }
    let mut latest = Animation {
        id: -1,
        previous: 0.0,
        time: 0.0,
        sequence: 0,
    };
    let mut selected: Option<Animation> = None;
    for step in 0..(head - begin + 10) % 10 {
        let slot = ((begin + step) % 10) as usize * 20;
        let frame = decode_animation(raw[slot..slot + 20].try_into().unwrap())?;
        latest = frame;
        let key = (model, frame.id);
        if crate::attack_timings::ATTACKS
            .binary_search_by_key(&key, |a| (a.0, a.1))
            .is_ok()
            || crate::attack_timings::SPECIALS
                .binary_search_by_key(&key, |a| (a.0, a.1))
                .is_ok()
        {
            if selected.is_some_and(|a| a.id != frame.id) {
                return Err(ReadError::InvalidAnimation);
            }
            selected = Some(frame);
        }
    }
    Ok(selected.unwrap_or(latest))
}

#[derive(Clone, Debug)]
pub struct Event {
    pub handle: u32,
    pub model: i32,
    pub kind: &'static str,
    pub animation: i32,
    pub phase_time: f32,
}

#[derive(Default)]
pub struct Tracker {
    previous: Option<(u32, i32, Animation)>,
}
impl Tracker {
    /// A changed target, animation, sequence or backwards clock ends continuity.
    /// Crossing evidence describes TAE time, never damage or a successful deflect.
    pub fn update(&mut self, sample: Option<(u32, i32, Animation)>) -> Vec<Event> {
        let mut events = Vec::new();
        let continuous = match (self.previous, sample) {
            (Some((h, m, a)), Some((nh, nm, n))) => {
                h == nh && m == nm && a.id == n.id && a.sequence == n.sequence && n.time >= a.time
            }
            _ => false,
        };
        if !continuous {
            if let Some((handle, model, a)) = self.previous {
                if a.id >= 0 {
                    events.push(Event {
                        handle,
                        model,
                        kind: "track_ended_or_cancelled",
                        animation: a.id,
                        phase_time: a.time,
                    });
                }
            }
            if let Some((handle, model, a)) = sample {
                if a.id >= 0 {
                    events.push(Event {
                        handle,
                        model,
                        kind: "track_started",
                        animation: a.id,
                        phase_time: a.time,
                    });
                }
            }
        }
        if let Some((handle, model, frame)) = sample {
            // First observation may arrive mid-attack: do not invent prior events.
            if continuous {
                let from = self.previous.unwrap().2.time;
                let key = (model, frame.id);
                let entries = crate::attack_timings::ATTACKS;
                let first = entries.partition_point(|a| (a.0, a.1) < key);
                let last = entries.partition_point(|a| (a.0, a.1) <= key);
                for entry in &entries[first..last] {
                    for (time, kind) in [
                        (entry.2, "activation_time_crossed"),
                        (entry.3, "deactivation_time_crossed"),
                    ] {
                        if from < time && frame.time >= time {
                            events.push(Event {
                                handle,
                                model,
                                kind,
                                animation: frame.id,
                                phase_time: time,
                            });
                        }
                    }
                }
                events.sort_by(|a, b| a.phase_time.total_cmp(&b.phase_time));
            }
        }
        self.previous = sample;
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(id: i32, time: f32) -> Animation {
        Animation {
            id,
            previous: time - 0.016,
            time,
            sequence: 1,
        }
    }
    fn put(raw: &mut [u8; 224], slot: usize, a: Animation) {
        for (i, bytes) in [
            a.id.to_le_bytes(),
            a.previous.to_le_bytes(),
            a.time.to_le_bytes(),
            5.0_f32.to_le_bytes(),
            a.sequence.to_le_bytes(),
        ]
        .into_iter()
        .enumerate()
        {
            raw[slot * 20 + i * 4..slot * 20 + i * 4 + 4].copy_from_slice(&bytes);
        }
    }
    #[test]
    fn completed_batch_selects_attack_over_auxiliary_and_rejects_competing_tracks() {
        let mut raw = [0; 224];
        raw[200..204].copy_from_slice(&1_i32.to_le_bytes());
        raw[204..208].copy_from_slice(&9_i32.to_le_bytes());
        put(&mut raw, 9, frame(3000, 0.5));
        put(&mut raw, 0, frame(40000, 2.0));
        assert_eq!(select_batch(&raw, 1020).unwrap().id, 3000);
        put(&mut raw, 0, frame(3001, 0.5));
        assert_eq!(select_batch(&raw, 1020), Err(ReadError::InvalidAnimation));
        raw[204..208].copy_from_slice(&1_i32.to_le_bytes());
        assert_eq!(select_batch(&raw, 1020).unwrap().id, -1);
        raw[200..204].copy_from_slice(&10_i32.to_le_bytes());
        assert_eq!(select_batch(&raw, 1020), Err(ReadError::InvalidAnimation));
    }
    #[test]
    fn events_are_ordered_once_and_cancelled_tracks_do_not_continue() {
        let mut t = Tracker::default();
        assert_eq!(t.update(Some((1, 1020, frame(3006, 0.1)))).len(), 1);
        let events = t.update(Some((1, 1020, frame(3006, 2.0))));
        assert_eq!(events.len(), 4);
        assert!(events
            .windows(2)
            .all(|w| w[0].phase_time <= w[1].phase_time));
        assert!(t.update(Some((1, 1020, frame(3006, 2.0)))).is_empty());
        let events = t.update(Some((1, 1020, frame(8010, 0.1))));
        assert_eq!(events[0].kind, "track_ended_or_cancelled");
        assert_eq!(events.len(), 2);
        assert_eq!(t.update(None).len(), 1);
        assert!(t.update(None).is_empty());
    }
    #[test]
    fn restart_or_target_change_never_replays_old_crossings() {
        let mut t = Tracker::default();
        t.update(Some((1, 1020, frame(3000, 0.7))));
        assert!(t
            .update(Some((2, 1020, frame(3000, 0.75))))
            .iter()
            .all(|e| !e.kind.contains("crossed")));
        assert_eq!(t.update(Some((2, 1020, frame(3000, 0.1)))).len(), 2);
    }
}
