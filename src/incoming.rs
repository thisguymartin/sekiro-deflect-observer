//! Incoming move type from current animation and resolved local attack parameters.
//! No distance gate, contact prediction, latency correction or press window.
use crate::cue::{Response, Target};
use crate::timing::{Decision, State};
use std::time::Duration;

pub(crate) fn npc_variation(model: i32, npc: i32) -> Option<i32> {
    let entries = crate::incoming_attacks::NPC_VARIATIONS;
    let index = entries.binary_search_by_key(&npc, |entry| entry.0).ok()?;
    let variation = entries[index].1;
    (variation / 10 == model).then_some(variation)
}

pub(crate) fn decide(
    target: &Target,
    now: Duration,
    enabled: [bool; 3],
    mikiri: bool,
    mut decision: Decision,
) -> Decision {
    let time = target.animation.time;
    decision.animation_time = time;
    decision.source_age_ms = target
        .captured_at
        .map_or(0.0, |at| now.saturating_sub(at).as_secs_f32() * 1000.0);
    decision.reason = "locked_no_incoming_phase";
    let entries = crate::incoming_attacks::ATTACKS;
    let variation = target
        .npc_param
        .and_then(|npc| npc_variation(target.model, npc))
        .unwrap_or(-1);
    let key = (target.model, variation, target.animation.id);
    let first = entries.partition_point(|entry| (entry.0, entry.1, entry.2) < key);
    let last = entries.partition_point(|entry| (entry.0, entry.1, entry.2) <= key);
    let mut previous_end = 0.0_f32;
    for (phase, &(_, _, _, start, end, code)) in entries[first..last].iter().enumerate() {
        if time >= end {
            previous_end = end;
            continue;
        }
        let response = match code {
            1 => Response::Parry,
            2 => Response::Dodge,
            3 => Response::Jump,
            4 if mikiri => Response::Mikiri,
            4 => Response::Parry, // classified thrust also allows deflection
            5 => Response::Avoid,
            _ => Response::Unverified,
        };
        let show = match response {
            Response::Parry => enabled[0],
            Response::Dodge => enabled[1],
            Response::Jump => enabled[2],
            Response::Mikiri => mikiri,
            Response::Avoid | Response::Unverified => enabled.iter().any(|value| *value),
        };
        // Never skip a disabled/unknown combo hit to show a later response.
        if !show {
            return decision;
        }
        decision.state = if time < start {
            State::Incoming
        } else {
            State::AttackActive
        };
        decision.response = response;
        decision.phase = Some(phase as u32);
        decision.activation = Some(crate::timing::Interval { start, end });
        // Wind-up reaches the center gate at activation. Active progress is a
        // separate phase fraction, never a measured collision or input result.
        decision.progress = if time < start {
            ((time - previous_end) / (start - previous_end)).clamp(0.0, 1.0)
        } else {
            ((time - start) / (end - start)).clamp(0.0, 1.0)
        };
        decision.reason = if response == Response::Unverified {
            "incoming_response_unknown"
        } else if time < start {
            "incoming_windup"
        } else {
            "incoming_attack_active"
        };
        return decision;
    }
    decision
}
