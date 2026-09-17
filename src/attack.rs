//! Attack facts shared by alerts and practice. No display preferences, timing
//! estimates, speed writes or HUD decisions belong in this module.
use crate::cue::Target;

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

pub(crate) fn npc_variation(model: i32, npc: i32) -> Option<i32> {
    let entries = crate::incoming_attacks::NPC_VARIATIONS;
    let index = entries.binary_search_by_key(&npc, |entry| entry.0).ok()?;
    let variation = entries[index].1;
    (variation / 10 == model).then_some(variation)
}

/// Current or next phase in this captured animation, including unresolved
/// phases. Consumers enforce capture freshness and their own feature policy.
/// Never skip an unknown phase to return a later classified combo hit.
pub fn classify(target: &Target) -> Option<Phase> {
    let time = target.animation.time;
    if target.animation_error.is_some() || !time.is_finite() || time < 0.0 {
        return None;
    }
    let entries = crate::incoming_attacks::ATTACKS;
    let variation = target
        .npc_param
        .and_then(|npc| npc_variation(target.model, npc))
        .unwrap_or(-1);
    let key = (target.model, variation, target.animation.id);
    let first = entries.partition_point(|entry| (entry.0, entry.1, entry.2) < key);
    let last = entries.partition_point(|entry| (entry.0, entry.1, entry.2) <= key);
    let mut previous_end = 0.0;
    for (index, &(_, _, _, start, end, code)) in entries[first..last].iter().enumerate() {
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
