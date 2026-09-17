//! Alert presentation of raw attack facts; preferences only affect this output.
//! No distance gate, contact prediction, latency correction or press window.
use crate::attack::{self, Kind};
use crate::cue::{Response, Target};
use crate::timing::{Decision, State};
use std::time::Duration;

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
    if let Some(phase) = attack::classify(target) {
        let (start, end) = (phase.start, phase.end);
        let response = match phase.kind {
            Kind::Parryable => Response::Parry,
            Kind::Grab => Response::Dodge,
            Kind::Sweep => Response::Jump,
            Kind::Thrust if mikiri => Response::Mikiri,
            Kind::Thrust => Response::Parry, // classified thrust also allows deflection
            Kind::Unparryable => Response::Avoid,
            Kind::Unknown => Response::Unverified,
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
        decision.phase = Some(phase.index);
        decision.activation = Some(crate::timing::Interval { start, end });
        // Wind-up reaches the center gate at activation. Active progress is a
        // separate phase fraction, never a measured collision or input result.
        decision.progress = if time < start {
            ((time - phase.previous_end) / (start - phase.previous_end)).clamp(0.0, 1.0)
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
