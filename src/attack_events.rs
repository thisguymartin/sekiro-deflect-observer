//! Completed animation-event batches and timeline crossings, not contact results.
use crate::cue::{decode_animation, Animation};
use crate::reader::ReadError;

pub const BATCH_BYTES: usize = 224;

/// Select within one completed engine batch. Never search preceding history.
/// Fixed-size input makes the game-thread capture allocation-free.
pub fn select_batch(
    raw: &[u8; BATCH_BYTES],
    model: i32,
    npc_param: Option<i32>,
) -> Result<Animation, ReadError> {
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
    let catalog = crate::attack::Catalog::new(model, npc_param);
    let mut selected: Option<(crate::attack::AnimationPriority, Animation)> = None;
    let mut ambiguous = false;
    for step in 0..(head - begin + 10) % 10 {
        let slot = ((begin + step) % 10) as usize * 20;
        let frame = decode_animation(raw[slot..slot + 20].try_into().unwrap())?;
        latest = frame;
        let priority = catalog.priority(frame.id);
        if priority == crate::attack::AnimationPriority::Unmapped {
            continue;
        }
        match selected {
            None => {
                selected = Some((priority, frame));
                ambiguous = false;
            }
            Some((current, _)) if priority > current => {
                selected = Some((priority, frame));
                ambiguous = false;
            }
            Some((current, prior)) if priority == current => {
                ambiguous |= prior.id != frame.id;
                selected = Some((priority, frame));
            }
            Some(_) => {}
        }
    }
    if ambiguous {
        Err(ReadError::InvalidAnimation)
    } else {
        Ok(selected.map_or(latest, |(_, frame)| frame))
    }
}

#[derive(Clone, Debug)]
pub struct Event {
    pub player_instance: usize,
    pub animation_module: usize,
    pub owner_generation: u64,
    pub handle: u32,
    pub model: i32,
    pub kind: &'static str,
    pub animation: i32,
    pub phase_time: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Observation {
    pub player_instance: usize,
    pub animation_module: usize,
    pub owner_generation: u64,
    pub handle: u32,
    pub model: i32,
    pub animation: Animation,
}

#[derive(Default)]
pub struct Tracker {
    previous: Option<Observation>,
}
impl Tracker {
    /// A changed owner, target, animation or backwards clock ends continuity.
    /// Raw ring sequence changes are not validated attack-occurrence evidence.
    /// Crossing evidence describes TAE time, never damage or a successful deflect.
    pub fn update(&mut self, sample: Option<Observation>) -> Vec<Event> {
        let mut events = Vec::new();
        let continuous = match (self.previous, sample) {
            (Some(previous), Some(next)) => {
                previous.player_instance == next.player_instance
                    && previous.animation_module == next.animation_module
                    && previous.owner_generation == next.owner_generation
                    && previous.handle == next.handle
                    && previous.model == next.model
                    && previous.animation.id == next.animation.id
                    && next.animation.time >= previous.animation.time
            }
            _ => false,
        };
        if !continuous {
            if let Some(previous) = self.previous {
                if previous.animation.id >= 0 {
                    events.push(Event {
                        player_instance: previous.player_instance,
                        animation_module: previous.animation_module,
                        owner_generation: previous.owner_generation,
                        handle: previous.handle,
                        model: previous.model,
                        kind: "track_ended_or_cancelled",
                        animation: previous.animation.id,
                        phase_time: previous.animation.time,
                    });
                }
            }
            if let Some(next) = sample {
                if next.animation.id >= 0 {
                    events.push(Event {
                        player_instance: next.player_instance,
                        animation_module: next.animation_module,
                        owner_generation: next.owner_generation,
                        handle: next.handle,
                        model: next.model,
                        kind: "track_started",
                        animation: next.animation.id,
                        phase_time: next.animation.time,
                    });
                }
            }
        }
        if let Some(observation) = sample {
            // First observation may arrive mid-attack: do not invent prior events.
            if continuous {
                let from = self.previous.unwrap().animation.time;
                let frame = observation.animation;
                let key = (observation.model, frame.id);
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
                                player_instance: observation.player_instance,
                                animation_module: observation.animation_module,
                                owner_generation: observation.owner_generation,
                                handle: observation.handle,
                                model: observation.model,
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
    fn observation(handle: u32, model: i32, animation: Animation) -> Observation {
        Observation {
            player_instance: 0x20000,
            animation_module: 0x30000,
            owner_generation: 1,
            handle,
            model,
            animation,
        }
    }
    #[test]
    fn completed_batch_selects_attack_over_auxiliary_and_rejects_competing_tracks() {
        let mut raw = [0; 224];
        raw[200..204].copy_from_slice(&1_i32.to_le_bytes());
        raw[204..208].copy_from_slice(&9_i32.to_le_bytes());
        put(&mut raw, 9, frame(3000, 0.5));
        put(&mut raw, 0, frame(40000, 2.0));
        assert_eq!(select_batch(&raw, 1020, None).unwrap().id, 3000);
        put(&mut raw, 0, frame(3001, 0.5));
        assert_eq!(
            select_batch(&raw, 1020, None),
            Err(ReadError::InvalidAnimation)
        );
        raw[204..208].copy_from_slice(&1_i32.to_le_bytes());
        assert_eq!(select_batch(&raw, 1020, None).unwrap().id, -1);
        raw[200..204].copy_from_slice(&10_i32.to_le_bytes());
        assert_eq!(
            select_batch(&raw, 1020, None),
            Err(ReadError::InvalidAnimation)
        );
    }

    #[test]
    fn incoming_attack_wins_over_legacy_only_object_contact() {
        let mut raw = [0; 224];
        raw[200..204].copy_from_slice(&1_i32.to_le_bytes());
        raw[204..208].copy_from_slice(&9_i32.to_le_bytes());
        put(&mut raw, 9, frame(5010, 0.1));
        put(&mut raw, 0, frame(3000, 0.5));

        assert_eq!(select_batch(&raw, 5080, None).unwrap().id, 3000);
    }
    #[test]
    fn events_are_ordered_once_and_cancelled_tracks_do_not_continue() {
        let mut t = Tracker::default();
        assert_eq!(
            t.update(Some(observation(1, 1020, frame(3006, 0.1)))).len(),
            1
        );
        let events = t.update(Some(observation(1, 1020, frame(3006, 2.0))));
        assert_eq!(events.len(), 4);
        assert!(events
            .windows(2)
            .all(|w| w[0].phase_time <= w[1].phase_time));
        assert!(t
            .update(Some(observation(1, 1020, frame(3006, 2.0))))
            .is_empty());
        let events = t.update(Some(observation(1, 1020, frame(8010, 0.1))));
        assert_eq!(events[0].kind, "track_ended_or_cancelled");
        assert_eq!(events.len(), 2);
        assert_eq!(t.update(None).len(), 1);
        assert!(t.update(None).is_empty());
    }
    #[test]
    fn raw_sequence_changes_do_not_invent_attack_occurrences() {
        let mut tracker = Tracker::default();
        tracker.update(Some(observation(1, 1010, frame(3000, 0.65))));
        let mut next = frame(3000, 0.68);
        next.sequence += 1;
        let events = tracker.update(Some(observation(1, 1010, next)));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, "activation_time_crossed");
    }
    #[test]
    fn restart_or_target_change_never_replays_old_crossings() {
        let mut t = Tracker::default();
        t.update(Some(observation(1, 1020, frame(3000, 0.7))));
        assert!(t
            .update(Some(observation(2, 1020, frame(3000, 0.75))))
            .iter()
            .all(|e| !e.kind.contains("crossed")));
        assert_eq!(
            t.update(Some(observation(2, 1020, frame(3000, 0.1)))).len(),
            2
        );
    }

    #[test]
    fn owner_or_generation_change_ends_continuity_before_crossings() {
        let first = Observation {
            player_instance: 0x20000,
            animation_module: 0x30000,
            owner_generation: 7,
            handle: 1,
            model: 1010,
            animation: frame(3000, 0.65),
        };
        let mut tracker = Tracker::default();
        tracker.update(Some(first));

        for changed in [
            Observation {
                player_instance: 0x21000,
                animation: frame(3000, 0.68),
                ..first
            },
            Observation {
                animation_module: 0x31000,
                animation: frame(3000, 0.68),
                ..first
            },
            Observation {
                owner_generation: 8,
                animation: frame(3000, 0.68),
                ..first
            },
        ] {
            let events = tracker.update(Some(changed));
            assert_eq!(events.len(), 2);
            assert_eq!(events[0].kind, "track_ended_or_cancelled");
            assert_eq!(events[1].kind, "track_started");
            assert!(events.iter().all(|event| !event.kind.contains("crossed")));
            assert_eq!(events[0].player_instance, first.player_instance);
            assert_eq!(events[0].animation_module, first.animation_module);
            assert_eq!(events[0].owner_generation, first.owner_generation);
            assert_eq!(events[1].player_instance, changed.player_instance);
            assert_eq!(events[1].animation_module, changed.animation_module);
            assert_eq!(events[1].owner_generation, changed.owner_generation);
            tracker.update(Some(first));
        }
    }
}
