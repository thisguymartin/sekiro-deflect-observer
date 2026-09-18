//! Portable cue decisions. All timestamps use one injected monotonic epoch.
//! Activation-only estimates are not contact measurements or success predictions.
use crate::cue::{self, LiveCue, Response, Target};
use std::collections::BTreeSet;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    Hidden,
    Neutral,
    Preparation,
    Actionable,
    Expired,
    Incoming,
    AttackActive,
}
impl State {
    pub fn label(self) -> &'static str {
        match self {
            Self::Hidden => "hidden",
            Self::Neutral => "neutral",
            Self::Preparation => "preparation",
            Self::Actionable => "actionable",
            Self::Expired => "expired",
            Self::Incoming => "incoming",
            Self::AttackActive => "attack_active",
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct Decision {
    pub capture_id: u64,
    pub capture_source: &'static str,
    pub state: State,
    pub response: Response,
    pub occurrence: u64,
    pub phase: Option<u32>,
    /// Extracted incoming activation window in animation seconds, not contact.
    pub activation: Option<Interval>,
    pub press: Option<Interval>,
    pub contact: Option<Interval>,
    pub preferred: Option<f32>,
    pub calibrated: bool,
    pub animation_time: f32,
    pub rate: Option<f32>,
    pub source_age_ms: f32,
    pub pulse: f32,
    pub pulse_emitted: bool,
    pub progress: f32,
    pub reason: &'static str,
    pub reach: Option<cue::Reach>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    pub start: f32,
    pub end: f32,
}
impl Interval {
    fn contains(self, time: f32) -> bool {
        time >= self.start && time < self.end
    }
}
#[derive(Clone, Debug)]
pub struct Settings {
    pub preparation_ms: f32,
    /// Positive presentation + input latency shifts both boundaries EARLIER.
    pub latency_ms: f32,
    /// Legacy activation estimates in animation milliseconds; never widened.
    pub leads_ms: [f32; 3],
    pub enabled: [bool; 3],
    pub pulse_duration_ms: f32,
    pub pulse_intensity: f32,
    pub reduced_flash: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            preparation_ms: 650.0,
            latency_ms: 0.0,
            leads_ms: [150.0, 300.0, 300.0],
            enabled: [true; 3],
            pulse_duration_ms: 80.0,
            pulse_intensity: 0.6,
            reduced_flash: false,
        }
    }
}
impl Settings {
    fn valid(&self) -> bool {
        (350.0..=1500.0).contains(&self.preparation_ms)
            && (0.0..=150.0).contains(&self.latency_ms)
            && self
                .leads_ms
                .iter()
                .zip([150.0, 300.0, 300.0])
                .all(|(v, max)| (25.0..=max).contains(v))
            && (16.0..=120.0).contains(&self.pulse_duration_ms)
            && (0.0..=1.0).contains(&self.pulse_intensity)
    }
}
/// Identity/version/evidence checked at config load; exact hit checked again here.
#[derive(Clone, Copy, Debug)]
pub struct Calibration {
    pub model: i32,
    pub animation: i32,
    pub phase: u32,
    pub activation_s: f32,
    pub end_s: f32,
    pub response: Response,
    pub contact_min_ms: f32,
    pub contact_max_ms: f32,
    pub early_ms: f32,
    pub late_ms: f32,
    pub preferred_ms: Option<f32>,
}
impl Calibration {
    fn valid(self) -> bool {
        (-500.0..=500.0).contains(&self.contact_min_ms)
            && (-500.0..=500.0).contains(&self.contact_max_ms)
            && self.contact_min_ms <= self.contact_max_ms
            && (0.0..=300.0).contains(&self.late_ms)
            && (0.0..=300.0).contains(&self.early_ms)
            && self.early_ms > self.late_ms
            && self.contact_max_ms - self.contact_min_ms < self.early_ms - self.late_ms
            && self.response != Response::Unverified
            && self
                .preferred_ms
                .is_none_or(|p| p.is_finite() && p > self.late_ms && p <= self.early_ms)
    }
}
#[derive(Default)]
pub struct Engine {
    live: LiveCue,
    lock: LiveCue,
    previous: Option<(Owner, Duration, f32)>,
    previous_rate: Option<f32>,
    rate: Option<f32>,
    occurrence: u64,
    capture_id: u64,
    emitted: BTreeSet<u32>,
    last_decision: Option<(u32, f32, Duration)>,
    pulse_started: Option<(u32, Duration)>,
}
type Owner = (usize, usize, u32, i32, i32, Option<i32>);
impl Engine {
    fn break_continuity(&mut self) {
        self.occurrence = self.occurrence.wrapping_add(1);
        self.previous_rate = None;
        self.rate = None;
        self.emitted.clear();
        self.last_decision = None;
        self.pulse_started = None;
    }
    /// Clears flashes immediately; reacquisition needs three progressing captures.
    pub fn clear(&mut self) {
        self.clear_timing();
        self.lock.push(Duration::ZERO, Ok(None));
    }
    fn clear_timing(&mut self) {
        self.break_continuity();
        self.previous = None;
        self.live.push(Duration::ZERO, Ok(None));
    }
    pub fn observe(&mut self, at: Duration, target: Option<&Target>) {
        self.lock.push(at, Ok(target.cloned()));
        let Some(mut target) = target
            .filter(|t| {
                t.animation_error.is_none()
                    && t.animation.time.is_finite()
                    && t.animation.time >= 0.0
            })
            .cloned()
        else {
            self.clear_timing();
            return;
        };
        let owner = (
            target.player_instance,
            target.animation_module,
            target.handle,
            target.model,
            target.animation.id,
            target.npc_param,
        );
        let source = target.captured_at.unwrap_or(at);
        if source > at {
            self.clear();
            return;
        }
        if let Some((old_owner, old_at, old_time)) = self.previous {
            // Polling the same clock/track cannot manufacture another game update.
            // The reader's raw sequence is not an occurrence identity.
            if owner == old_owner
                && target.animation.time == old_time
                && (source == old_at || target.captured_at.is_none())
            {
                if target.captured_at.is_none() && source > old_at {
                    // A new polling read observed no progress: suppress now.
                    // Keep the old source for the next measured delta; never
                    // claim that repeatedly reading it made the clock fresh.
                    self.rate = None;
                    self.last_decision = None;
                    self.pulse_started = None;
                }
                target.captured_at = Some(old_at);
                self.live.push(at, Ok(Some(target)));
                return;
            }
            let delta = source.checked_sub(old_at).map(|d| d.as_secs_f32());
            self.rate = None;
            if owner == old_owner
                && delta.is_some_and(|d| (0.001..=0.05).contains(&d))
                && target.animation.time > old_time
            {
                let rate = (target.animation.time - old_time) / delta.unwrap();
                if (0.25..=4.0).contains(&rate) {
                    if self
                        .previous_rate
                        .is_some_and(|r| (rate - r).abs() <= r * 0.2)
                    {
                        self.rate = Some(rate)
                    } else {
                        self.last_decision = None;
                        self.pulse_started = None;
                    }
                    self.previous_rate = Some(rate);
                } else {
                    self.break_continuity();
                }
            } else if owner != old_owner
                || target.animation.time < old_time
                || delta.is_none_or(|d| d > 0.05 || d <= 0.0)
            {
                self.break_continuity();
            } else {
                // A newly captured unchanged clock is positive pause evidence.
                self.previous_rate = None;
                self.last_decision = None;
                self.pulse_started = None;
            }
        } else {
            self.break_continuity();
        }
        self.previous = Some((owner, source, target.animation.time));
        self.capture_id = target.metadata.capture_id;
        target.captured_at = Some(source);
        self.live.push(at, Ok(Some(target)));
    }
    pub fn decide(&mut self, now: Duration) -> Decision {
        self.decide_with(now, &Settings::default(), &[])
    }

    /// Classify the current attack without predicting reach, contact or input time.
    pub fn incoming(&self, now: Duration, enabled: [bool; 3], mikiri: bool) -> Decision {
        let Some(lock) = self.lock.current_lock(now) else {
            return Decision {
                reason: "missing_or_stale_lock",
                ..Default::default()
            };
        };
        let neutral = Decision {
            state: State::Neutral,
            capture_id: lock.metadata.capture_id,
            capture_source: lock.metadata.source,
            occurrence: self.occurrence,
            reason: "locked_animation_unavailable",
            ..Default::default()
        };
        let Some(target) = self.live.current(now) else {
            return neutral;
        };
        crate::incoming::decide(target, now, enabled, mikiri, neutral)
    }
    pub fn decide_with(
        &mut self,
        now: Duration,
        settings: &Settings,
        profiles: &[Calibration],
    ) -> Decision {
        let Some(target) = self.live.current(now) else {
            self.last_decision = None;
            self.pulse_started = None;
            if let Some(target) = self.lock.current_lock(now) {
                return Decision {
                    state: State::Neutral,
                    capture_id: target.metadata.capture_id,
                    capture_source: target.metadata.source,
                    occurrence: self.occurrence,
                    source_age_ms: target
                        .captured_at
                        .map_or(0.0, |at| now.saturating_sub(at).as_secs_f32() * 1000.0),
                    reason: if target.animation_error.is_some() {
                        "locked_animation_unavailable"
                    } else {
                        "locked_animation_stale"
                    },
                    ..Default::default()
                };
            }
            return Decision {
                reason: "missing_or_stale_observation",
                ..Default::default()
            };
        };
        let age = now - target.captured_at.unwrap();
        let reach = cue::reach(target);
        let mut decision = Decision {
            capture_id: self.capture_id,
            capture_source: target.metadata.source,
            state: State::Neutral,
            occurrence: self.occurrence,
            animation_time: target.animation.time,
            rate: self.rate,
            source_age_ms: age.as_secs_f32() * 1000.0,
            reason: "clock_unstable",
            reach: Some(reach),
            ..Default::default()
        };
        if !settings.valid()
            || self.rate.is_none()
            || age > Duration::from_millis(25)
            || reach.reason == "reach_invalid_geometry"
        {
            decision.reason = if !settings.valid() {
                "invalid_settings"
            } else if reach.reason == "reach_invalid_geometry" {
                reach.reason
            } else {
                "clock_unstable_or_old"
            };
            self.last_decision = None;
            self.pulse_started = None;
            return decision;
        }
        let rate = self.rate.unwrap();
        let time = target.animation.time + rate * age.as_secs_f32();
        decision.animation_time = time;
        let entries = crate::attack_timings::ATTACKS;
        let key = (target.model, target.animation.id);
        let first = entries.partition_point(|a| (a.0, a.1) < key);
        let last = entries.partition_point(|a| (a.0, a.1) <= key);
        let mut selected: Option<(u8, Decision)> = None;
        for (phase, &(_, _, start, end, classified)) in
            entries[first..last].iter().enumerate().take(256)
        {
            let response = cue::phase_response(target, start, classified);
            let index = match response {
                Response::Parry => 0,
                Response::Dodge => 1,
                Response::Jump => 2,
                Response::Unverified | Response::Mikiri | Response::Avoid => {
                    // Neutral attention only: the archive contains an attack
                    // phase, but it does not establish a safe defensive response.
                    // No press/contact estimate, progress countdown or pulse.
                    if settings.enabled.iter().any(|v| *v)
                        && time >= start - rate * settings.preparation_ms / 1000.0
                        && time < end
                        && selected.as_ref().is_none_or(|(p, _)| *p > 2)
                    {
                        selected = Some((
                            2,
                            Decision {
                                phase: Some(phase as u32),
                                reason: "unverified_attack",
                                ..decision.clone()
                            },
                        ));
                    }
                    continue;
                }
            };
            if !settings.enabled[index] {
                continue;
            }
            let profile = profiles.iter().find(|p| {
                p.model == target.model
                    && p.animation == target.animation.id
                    && p.phase == phase as u32
                    && p.activation_s == start
                    && p.end_s == end
                    && p.response == response
                    && p.valid()
            });
            let shift = rate * settings.latency_ms / 1000.0;
            let (contact, press, preferred) = if let Some(p) = profile {
                let contact = Interval {
                    start: start + p.contact_min_ms / 1000.0,
                    end: start + p.contact_max_ms / 1000.0,
                };
                let press = Interval {
                    start: contact.end - rate * p.early_ms / 1000.0 - shift,
                    end: contact.start - rate * p.late_ms / 1000.0 - shift,
                };
                let preferred = p
                    .preferred_ms
                    .map(|p| (contact.start + contact.end) / 2.0 - rate * p / 1000.0 - shift)
                    .filter(|p| press.contains(*p));
                (contact, press, preferred)
            } else {
                (
                    Interval { start, end: start },
                    Interval {
                        start: start - settings.leads_ms[index] / 1000.0 - shift,
                        end: start - shift,
                    },
                    None,
                )
            };
            if press.start >= press.end {
                continue;
            }
            let prepare_start = press.start - rate * settings.preparation_ms / 1000.0;
            let in_reach = reach.reason == "within_coarse_reach";
            let (priority, state) = if press.contains(time) && in_reach {
                (0, State::Actionable)
            } else if time >= prepare_start && time < press.end {
                (1, State::Preparation)
            } else if time >= press.end && time < end {
                (3, State::Expired)
            } else {
                continue;
            };
            if selected.as_ref().is_some_and(|(p, _)| *p <= priority) {
                continue;
            }
            selected = Some((
                priority,
                Decision {
                    state,
                    response,
                    phase: Some(phase as u32),
                    press: Some(press),
                    contact: Some(contact),
                    preferred,
                    calibrated: profile.is_some(),
                    progress: ((time - prepare_start) / (press.end - prepare_start))
                        .clamp(0.0, 1.0),
                    reason: if !in_reach {
                        reach.reason
                    } else if profile.is_some() {
                        "calibrated_profile"
                    } else {
                        "activation_estimate"
                    },
                    ..decision.clone()
                },
            ));
        }
        let Some((_, mut decision)) = selected else {
            self.last_decision = None;
            self.pulse_started = None;
            decision.reason = "unknown_or_disabled_phase";
            return decision;
        };
        let phase = decision.phase.unwrap();
        if decision.state == State::Actionable {
            if let Some(preferred) = decision.preferred {
                let crossing = self.last_decision.is_some_and(|(p, t, at)| {
                    p == phase
                        && t < preferred
                        && time >= preferred
                        && now.checked_sub(at).is_some_and(|d| d < cue::FRESHNESS)
                });
                if crossing && self.emitted.insert(phase) {
                    self.pulse_started = Some((phase, now));
                    decision.pulse_emitted = true;
                }
            }
            if let Some((p, at)) = self.pulse_started.filter(|(p, _)| *p == phase) {
                let _ = p;
                let elapsed = now.saturating_sub(at).as_secs_f32() * 1000.0;
                let intensity = if settings.reduced_flash {
                    0.0
                } else {
                    settings.pulse_intensity
                };
                decision.pulse =
                    intensity * (1.0 - elapsed / settings.pulse_duration_ms).clamp(0.0, 1.0);
            }
        } else {
            self.pulse_started = None
        }
        self.last_decision = Some((phase, time, now));
        decision
    }
}
