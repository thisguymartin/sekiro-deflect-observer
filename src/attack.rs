//! Attack facts shared by alerts and practice. No display preferences, timing
//! estimates, speed writes or HUD decisions belong in this module.
use crate::cue::Target;

type IncomingRow = (i32, i32, i32, f32, f32, u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Parryable,
    Grab,
    Sweep,
    Thrust,
    Unparryable,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Phase {
    pub index: u32,
    pub kind: Kind,
    pub start: f32,
    pub end: f32,
    pub previous_end: f32,
    pub animation_time: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum AnimationPriority {
    Unmapped,
    Legacy,
    Incoming,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Catalog {
    model: i32,
    variation: i32,
}

pub(crate) fn npc_variation(model: i32, npc: i32) -> Option<i32> {
    let entries = crate::incoming_attacks::NPC_VARIATIONS;
    let index = entries.binary_search_by_key(&npc, |entry| entry.0).ok()?;
    let variation = entries[index].1;
    (variation / 10 == model).then_some(variation)
}

impl Catalog {
    pub(crate) fn new(model: i32, npc_param: Option<i32>) -> Self {
        Self {
            model,
            variation: npc_param
                .and_then(|npc| npc_variation(model, npc))
                .unwrap_or(-1),
        }
    }

    fn incoming(self, animation: i32) -> &'static [IncomingRow] {
        let entries = crate::incoming_attacks::ATTACKS;
        let key = (self.model, self.variation, animation);
        let first = entries.partition_point(|entry| (entry.0, entry.1, entry.2) < key);
        let last = entries.partition_point(|entry| (entry.0, entry.1, entry.2) <= key);
        &entries[first..last]
    }

    pub(crate) fn priority(self, animation: i32) -> AnimationPriority {
        if !self.incoming(animation).is_empty() {
            return AnimationPriority::Incoming;
        }
        let key = (self.model, animation);
        if crate::attack_timings::ATTACKS
            .binary_search_by_key(&key, |entry| (entry.0, entry.1))
            .is_ok()
            || crate::attack_timings::SPECIALS
                .binary_search_by_key(&key, |entry| (entry.0, entry.1))
                .is_ok()
        {
            AnimationPriority::Legacy
        } else {
            AnimationPriority::Unmapped
        }
    }
}

/// Current or next phase in this captured animation, including unresolved
/// phases. Consumers enforce capture freshness and their own feature policy.
/// Never skip an unknown phase to return a later classified combo hit.
pub fn classify(target: &Target) -> Option<Phase> {
    let time = target.animation.time;
    if target.animation_error.is_some() || !time.is_finite() || time < 0.0 {
        return None;
    }
    let entries = Catalog::new(target.model, target.npc_param).incoming(target.animation.id);
    let mut previous_end = 0.0;
    for (index, &(_, _, _, start, end, code)) in entries.iter().enumerate() {
        if !start.is_finite() || !end.is_finite() || start < 0.0 || end <= start {
            return None;
        }
        if time >= end {
            previous_end = end;
            continue;
        }
        let kind = match code {
            1 => Kind::Parryable,
            2 => Kind::Grab,
            3 => Kind::Sweep,
            4 => Kind::Thrust,
            5 => Kind::Unparryable,
            _ => Kind::Unknown,
        };
        return Some(Phase {
            index: index as u32,
            kind,
            start,
            end,
            previous_end,
            animation_time: time,
        });
    }
    None
}
