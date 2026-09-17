//! Synthetic observations only: these tests do not assert successful deflection.
// Keep exact generated boundary literals visible for evidence matching.
#![allow(clippy::excessive_precision)]
use sekiro_deflect_observer::{
    cue::{Animation, Camera, Response, Target},
    timing::{Calibration, Engine, Settings, State},
};
use std::time::Duration;

fn target(time: f32) -> Target {
    Target {
        metadata: Default::default(),
        captured_at: None,
        player_instance: 0x30000,
        animation_module: 0xc0000,
        handle: 0x10004001,
        model: 1010,
        animation: Animation {
            id: 3000,
            previous: time - 0.016,
            time,
            sequence: 1,
        },
        position: [0.0, 0.0, 2.0],
        anchor: [0.0, 1.55, 0.0],
        facing: [0.0, 0.0, -1.0],
        player_position: [0.0; 3],
        body_radius: 0.0,
        camera: Some(Camera {
            right: [1.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            forward: [0.0, 0.0, 1.0],
            position: [0.0; 3],
            fov: 1.0,
            aspect: 16.0 / 9.0,
            near: 0.08,
            far: 1000.0,
        }),
        animation_error: None,
    }
}

fn seeded(animation: f32) -> Engine {
    let mut engine = Engine::default();
    for (ms, time) in [
        (968, animation - 0.032),
        (984, animation - 0.016),
        (1000, animation),
    ] {
        engine.observe(Duration::from_millis(ms), Some(&target(time)));
    }
    engine
}

fn profile() -> Calibration {
    Calibration {
        model: 1010,
        animation: 3000,
        phase: 0,
        activation_s: 0.666666687,
        end_s: 0.800000012,
        response: Response::Parry,
        contact_min_ms: 0.0,
        contact_max_ms: 0.0,
        early_ms: 150.0,
        late_ms: 0.0,
        preferred_ms: Some(100.0),
    }
}

#[test]
fn preparation_action_and_expiry_are_distinct() {
    for (time, wanted) in [
        (0.4, State::Preparation),
        (0.55, State::Actionable),
        (0.7, State::Expired),
    ] {
        assert_eq!(
            seeded(time).decide(Duration::from_millis(1000)).state,
            wanted
        );
    }
}

#[test]
fn positive_latency_moves_both_boundaries_earlier_without_widening() {
    let base = seeded(0.55).decide_with(Duration::from_millis(1000), &Settings::default(), &[]);
    let shifted = seeded(0.55).decide_with(
        Duration::from_millis(1000),
        &Settings {
            latency_ms: 30.0,
            ..Default::default()
        },
        &[],
    );
    let (base, shifted) = (base.press.unwrap(), shifted.press.unwrap());
    assert!((base.start - shifted.start - 0.030).abs() < 0.00001);
    assert!((base.end - shifted.end - 0.030).abs() < 0.00001);
    assert!((shifted.end - shifted.start - 0.150).abs() < 0.00001);
}

#[test]
fn preferred_pulse_is_once_per_hit_and_ends_with_the_interval() {
    let mut engine = seeded(0.55);
    let options = Settings::default();
    let calibration = [profile()];
    let before = engine.decide_with(Duration::from_millis(1000), &options, &calibration);
    assert!(!before.pulse_emitted);
    let first = engine.decide_with(Duration::from_millis(1020), &options, &calibration);
    assert!(first.pulse_emitted);
    assert!(first.pulse > 0.0);
    assert!(
        !engine
            .decide_with(Duration::from_millis(1021), &options, &calibration)
            .pulse_emitted
    );
    engine.observe(Duration::from_millis(1032), Some(&target(0.582)));
    assert!(
        !engine
            .decide_with(Duration::from_millis(1032), &options, &calibration)
            .pulse_emitted
    );
    engine.observe(Duration::from_millis(1048), Some(&target(0.598)));
    engine.observe(Duration::from_millis(1064), Some(&target(0.614)));
    engine.observe(Duration::from_millis(1080), Some(&target(0.630)));
    engine.observe(Duration::from_millis(1096), Some(&target(0.646)));
    let expired = engine.decide_with(Duration::from_millis(1120), &options, &calibration);
    assert_eq!(expired.state, State::Expired);
    assert_eq!(expired.pulse, 0.0);
}

#[test]
fn uncalibrated_or_missed_preferred_time_never_fabricates_a_pulse() {
    let mut engine = seeded(0.55);
    engine.decide(Duration::from_millis(1000));
    assert!(!engine.decide(Duration::from_millis(1020)).pulse_emitted);
    let mut late = seeded(0.60);
    assert!(
        !late
            .decide_with(
                Duration::from_millis(1000),
                &Settings::default(),
                &[profile()]
            )
            .pulse_emitted
    );
}

#[test]
fn render_delay_cannot_extend_a_press_past_its_end() {
    let mut engine = Engine::default();
    for (ms, animation) in [(968, 0.618), (984, 0.634), (1000, 0.650)] {
        engine.observe(Duration::from_millis(ms), Some(&target(animation)));
    }
    // The table's activation is 0.666666687. At 1x, render sees 0.670.
    assert_ne!(
        engine.decide(Duration::from_millis(1020)).state,
        State::Actionable
    );
}

#[test]
fn pauses_speed_changes_rewinds_and_bad_observations_cancel_action() {
    let mut engine = seeded(0.55);
    assert_eq!(
        engine.decide(Duration::from_millis(1000)).state,
        State::Actionable
    );
    let mut paused = target(0.55);
    paused.captured_at = Some(Duration::from_millis(1016));
    engine.observe(Duration::from_millis(1016), Some(&paused));
    assert_eq!(
        engine.decide(Duration::from_millis(1016)).state,
        State::Neutral
    );
    engine.observe(Duration::from_millis(1032), Some(&target(0.582)));
    assert_eq!(
        engine.decide(Duration::from_millis(1032)).state,
        State::Neutral
    );
    engine.observe(Duration::from_millis(1048), Some(&target(0.614)));
    assert_eq!(
        engine
            .decide(Duration::from_millis(1048))
            .rate
            .map(|r| (r * 10.0).round()),
        Some(20.0)
    );
    let occurrence = engine.decide(Duration::from_millis(1048)).occurrence;
    engine.observe(Duration::from_millis(1064), Some(&target(0.1)));
    let rewound = engine.decide(Duration::from_millis(1064));
    assert_eq!(rewound.state, State::Neutral);
    assert_ne!(rewound.occurrence, occurrence);
    engine.observe(Duration::from_millis(1070), None);
    assert_eq!(
        engine.decide(Duration::from_millis(1070)).state,
        State::Hidden
    );
}

#[test]
fn duplicate_poll_and_sequence_changes_do_not_refresh_animation_capture() {
    let mut engine = seeded(0.55);
    for ms in [1008, 1016, 1024, 1032, 1040, 1048] {
        let mut same = target(0.55);
        same.animation.sequence = ms as i32;
        engine.observe(Duration::from_millis(ms), Some(&same));
    }
    assert_ne!(
        engine.decide(Duration::from_millis(1048)).state,
        State::Actionable
    );
    let locked = engine.decide(Duration::from_millis(1050));
    assert_eq!(locked.state, State::Neutral);
    assert_eq!(locked.reason, "locked_animation_stale");
    assert!(locked.press.is_none());
    assert_eq!(
        engine.decide(Duration::from_millis(1098)).state,
        State::Hidden
    );
}

#[test]
fn lock_switch_and_reacquisition_require_fresh_progress_and_cannot_replay_flash() {
    let mut engine = seeded(0.55);
    engine.decide_with(
        Duration::from_millis(1000),
        &Settings::default(),
        &[profile()],
    );
    assert!(
        engine
            .decide_with(
                Duration::from_millis(1020),
                &Settings::default(),
                &[profile()]
            )
            .pulse_emitted
    );
    engine.observe(Duration::from_millis(1021), None);
    assert_eq!(
        engine.decide(Duration::from_millis(1021)).state,
        State::Hidden
    );
    let mut switched = target(0.58);
    switched.handle += 1;
    engine.observe(Duration::from_millis(1022), Some(&switched));
    assert_eq!(
        engine.decide(Duration::from_millis(1022)).state,
        State::Neutral
    );
    assert!(
        !engine
            .decide_with(
                Duration::from_millis(1022),
                &Settings::default(),
                &[profile()]
            )
            .pulse_emitted
    );
}

#[test]
fn two_combo_hits_and_repeated_animation_have_independent_pulses() {
    let mut engine = Engine::default();
    let mut hits = Vec::new();
    let profiles = [
        Calibration {
            model: 1020,
            animation: 3006,
            phase: 0,
            activation_s: 0.733333349,
            end_s: 0.933333337,
            ..profile()
        },
        Calibration {
            model: 1020,
            animation: 3006,
            phase: 1,
            activation_s: 1.633333325,
            end_s: 1.833333373,
            ..profile()
        },
    ];
    for occurrence in 0..2 {
        for tick in 0..115 {
            let time = tick as f32 * 0.016;
            let mut sample = target(time);
            sample.model = 1020;
            sample.animation.id = 3006;
            let at = Duration::from_millis(1000 + occurrence * 1840 + tick * 16);
            engine.observe(at, Some(&sample));
            let decision = engine.decide_with(at, &Settings::default(), &profiles);
            if decision.pulse_emitted {
                hits.push((decision.occurrence, decision.phase.unwrap()))
            }
        }
    }
    assert_eq!(hits.len(), 4);
    assert_eq!(
        hits.iter().map(|h| h.1).collect::<Vec<_>>(),
        vec![0, 1, 0, 1]
    );
    assert_ne!(hits[0].0, hits[2].0);
}

#[test]
fn boundaries_stalls_invalid_settings_and_response_exclusions_fail_closed() {
    let start = 0.666666687 - 0.150;
    assert_eq!(
        seeded(start).decide(Duration::from_millis(1000)).state,
        State::Actionable
    );
    assert_eq!(
        seeded(0.666666687)
            .decide(Duration::from_millis(1000))
            .state,
        State::Expired
    );
    assert_eq!(
        seeded(0.55).decide(Duration::from_millis(1050)).state,
        State::Hidden
    );
    assert_eq!(
        seeded(0.55).decide(Duration::from_millis(999)).state,
        State::Hidden
    );
    assert_eq!(
        seeded(0.55)
            .decide_with(
                Duration::from_millis(1000),
                &Settings {
                    latency_ms: f32::NAN,
                    ..Default::default()
                },
                &[]
            )
            .state,
        State::Neutral
    );
    assert_eq!(
        seeded(0.55)
            .decide_with(
                Duration::from_millis(1000),
                &Settings {
                    enabled: [false; 3],
                    ..Default::default()
                },
                &[]
            )
            .state,
        State::Neutral
    );
    for mutate in [0, 1, 2, 3] {
        let mut engine = Engine::default();
        for (ms, t) in [(968, 0.518), (984, 0.534), (1000, 0.55)] {
            let mut sample = target(t);
            match mutate {
                0 => sample.position[2] = 100.0,
                1 => sample.facing[2] = 1.0,
                2 => sample.animation.id = 3005,
                _ => {
                    sample.animation_error =
                        Some(sekiro_deflect_observer::reader::ReadError::InvalidAnimation)
                }
            }
            engine.observe(Duration::from_millis(ms), Some(&sample));
        }
        assert_ne!(
            engine.decide(Duration::from_millis(1000)).state,
            State::Actionable
        );
    }
    let mut skipped = seeded(0.5);
    skipped.decide_with(
        Duration::from_millis(1000),
        &Settings::default(),
        &[profile()],
    );
    assert!(
        !skipped
            .decide_with(
                Duration::from_millis(1300),
                &Settings::default(),
                &[profile()]
            )
            .pulse_emitted
    );
}

#[test]
fn newly_polled_stopped_clock_suppresses_action_in_that_decision() {
    let mut engine = seeded(0.55);
    assert_eq!(
        engine.decide(Duration::from_millis(1000)).state,
        State::Actionable
    );
    engine.observe(Duration::from_millis(1008), Some(&target(0.55)));
    assert_ne!(
        engine.decide(Duration::from_millis(1008)).state,
        State::Actionable
    );
}

#[test]
fn latency_shift_scales_with_playback_rate_and_all_response_cases_stay_conservative() {
    let mut engine = Engine::default();
    for (ms, time) in [(968, 0.486), (984, 0.518), (1000, 0.55)] {
        engine.observe(Duration::from_millis(ms), Some(&target(time)));
    }
    let base = engine.decide(Duration::from_millis(1000));
    let shifted = engine.decide_with(
        Duration::from_millis(1000),
        &Settings {
            latency_ms: 30.0,
            ..Default::default()
        },
        &[],
    );
    assert!((base.press.unwrap().start - shifted.press.unwrap().start - 0.060).abs() < 0.00001);
    for (model, id, time, response) in [
        (1010, 3000, 0.55, Response::Parry),
        (5020, 100003005, 0.45, Response::Dodge),
        (5100, 100003009, 1.05, Response::Jump),
        (1010, 3005, 0.55, Response::Unverified),
    ] {
        let mut engine = Engine::default();
        for (ms, offset) in [(968, 0.032), (984, 0.016), (1000, 0.0)] {
            let mut sample = target(time - offset);
            sample.model = model;
            sample.animation.id = id;
            engine.observe(Duration::from_millis(ms), Some(&sample));
        }
        let decision = engine.decide(Duration::from_millis(1000));
        assert_eq!(decision.response, response);
        assert_eq!(
            decision.state == State::Actionable,
            response != Response::Unverified
        );
    }
}

#[test]
fn worker_progress_during_a_wholly_skipped_render_interval_cannot_issue_a_late_pulse() {
    let mut engine = seeded(0.4);
    engine.decide_with(
        Duration::from_millis(1000),
        &Settings::default(),
        &[profile()],
    );
    for tick in 1..=20 {
        engine.observe(
            Duration::from_millis(1000 + tick * 16),
            Some(&target(0.4 + tick as f32 * 0.016)),
        );
    }
    let late = engine.decide_with(
        Duration::from_millis(1320),
        &Settings::default(),
        &[profile()],
    );
    assert_eq!(late.state, State::Expired);
    assert!(!late.pulse_emitted);
    assert_eq!(late.pulse, 0.0);
}

#[test]
fn original_capture_identity_survives_redelivery_and_reduced_flash_has_no_brightness_pulse() {
    let mut engine = Engine::default();
    for (id, ms, time) in [(1, 968, 0.518), (2, 984, 0.534), (3, 1000, 0.55)] {
        let mut sample = target(time);
        sample.captured_at = Some(Duration::from_millis(ms));
        sample.metadata.capture_id = id;
        sample.metadata.source = "event_batch";
        engine.observe(Duration::from_millis(ms), Some(&sample));
    }
    let mut repeated = target(0.55);
    repeated.captured_at = Some(Duration::from_millis(1000));
    repeated.metadata.capture_id = 3;
    repeated.metadata.source = "event_batch";
    engine.observe(Duration::from_millis(1008), Some(&repeated));
    let options = Settings {
        reduced_flash: true,
        ..Default::default()
    };
    let before = engine.decide_with(Duration::from_millis(1008), &options, &[profile()]);
    assert_eq!(before.capture_id, 3);
    assert_eq!(before.capture_source, "event_batch");
    assert!((before.source_age_ms - 8.0).abs() < 0.001);
    let crossing = engine.decide_with(Duration::from_millis(1020), &options, &[profile()]);
    assert_eq!(crossing.state, State::Actionable);
    assert_eq!(crossing.pulse, 0.0);
}

#[test]
fn mapped_windup_warns_before_reach_but_never_prompts_an_out_of_reach_press() {
    let mut engine = Engine::default();
    for (ms, time) in [(968, 0.368), (984, 0.384), (1000, 0.4)] {
        let mut sample = target(time);
        sample.model = 1550;
        sample.animation.id = 3001;
        sample.position[2] = 5.0;
        engine.observe(Duration::from_millis(ms), Some(&sample));
    }
    let ready = engine.decide(Duration::from_millis(1000));
    assert_eq!(ready.state, State::Preparation);
    assert_eq!(ready.phase, Some(0));
    for tick in 1..25 {
        let mut sample = target(0.4 + tick as f32 * 0.016);
        sample.model = 1550;
        sample.animation.id = 3001;
        sample.position[2] = 5.0;
        let at = Duration::from_millis(1000 + tick * 16);
        engine.observe(at, Some(&sample));
        let d = engine.decide(at);
        assert_ne!(d.state, State::Actionable);
        assert!(!d.pulse_emitted);
        assert_eq!(d.pulse, 0.0);
    }
    engine.observe(Duration::from_millis(1400), None);
    assert_eq!(
        engine.decide(Duration::from_millis(1400)).state,
        State::Hidden
    );
}

#[test]
fn recorded_unverified_thrust_has_neutral_attention_without_inventing_a_response() {
    let mut engine = Engine::default();
    for tick in 0..70 {
        let mut sample = target(tick as f32 * 0.016);
        sample.model = 1550;
        sample.animation.id = 3003; // mixed melee/dummy/projectile routing
        let at = Duration::from_millis(1000 + tick * 16);
        engine.observe(at, Some(&sample));
        let d = engine.decide(at);
        assert_ne!(d.state, State::Actionable);
        assert_eq!(d.response, Response::Unverified);
        assert!(d.press.is_none());
        assert!(!d.pulse_emitted);
        if tick == 30 {
            assert_eq!(d.reason, "unverified_attack");
            assert_eq!(d.phase, Some(0));
        }
    }
}

#[test]
fn reach_failures_remain_non_actionable_and_identify_the_blocking_observation() {
    for (case, expected) in [
        (0, "reach_distance"),
        (1, "reach_away_facing"),
        (2, "reach_height"),
        (3, "reach_invalid_facing"),
        (4, "reach_invalid_geometry"),
    ] {
        let mut engine = Engine::default();
        for (ms, time) in [(968, 0.518), (984, 0.534), (1000, 0.55)] {
            let mut sample = target(time);
            match case {
                0 => sample.position[2] = 5.0,
                1 => sample.facing[2] = 1.0,
                2 => sample.position[1] = 2.0,
                3 => sample.facing = [0.0; 3],
                _ => sample.body_radius = f32::NAN,
            }
            engine.observe(Duration::from_millis(ms), Some(&sample));
        }
        let d = engine.decide(Duration::from_millis(1000));
        assert_ne!(d.state, State::Actionable);
        assert_eq!(d.reach.unwrap().reason, expected);
        assert!(!d.pulse_emitted);
        assert_eq!(d.pulse, 0.0);
    }
}

#[test]
fn animation_failure_keeps_fresh_lock_visible_and_cancels_every_press() {
    let mut engine = seeded(0.55);
    assert_eq!(
        engine.decide(Duration::from_millis(1000)).state,
        State::Actionable
    );
    let mut failed = target(0.55);
    failed.animation_error = Some(sekiro_deflect_observer::reader::ReadError::InvalidAnimation);
    failed.captured_at = Some(Duration::from_millis(900));
    let mut visible_lock = sekiro_deflect_observer::cue::LiveCue::default();
    visible_lock.push(Duration::from_millis(1008), Ok(Some(failed.clone())));
    assert!(visible_lock.current(Duration::from_millis(1008)).is_none());
    assert!(visible_lock
        .current_lock(Duration::from_millis(1008))
        .is_some());
    engine.observe(Duration::from_millis(1008), Some(&failed));
    for ms in [1008, 1030, 1057] {
        let d = engine.decide_with(
            Duration::from_millis(ms),
            &Settings::default(),
            &[profile()],
        );
        assert_eq!(d.state, State::Neutral);
        assert_eq!(d.reason, "locked_animation_unavailable");
        assert_eq!(d.response, Response::Unverified);
        assert!(d.press.is_none() && d.contact.is_none() && d.phase.is_none());
        assert_eq!(d.pulse, 0.0);
        assert!(!d.pulse_emitted);
    }
    assert_eq!(
        engine.decide(Duration::from_millis(1058)).state,
        State::Hidden
    );
    assert!(visible_lock
        .current_lock(Duration::from_millis(1058))
        .is_none());

    engine.observe(Duration::from_millis(1060), Some(&target(0.56)));
    assert_eq!(
        engine.decide(Duration::from_millis(1060)).state,
        State::Neutral
    );
    engine.observe(Duration::from_millis(1076), Some(&target(0.576)));
    assert_eq!(
        engine.decide(Duration::from_millis(1076)).state,
        State::Neutral
    );
    engine.observe(Duration::from_millis(1092), Some(&target(0.592)));
    assert_eq!(
        engine.decide(Duration::from_millis(1092)).state,
        State::Actionable
    );
    engine.clear();
    assert_eq!(
        engine.decide(Duration::from_millis(1093)).state,
        State::Hidden
    );
    engine.observe(Duration::from_millis(1100), Some(&failed));
    engine.observe(Duration::from_millis(1101), None);
    assert_eq!(
        engine.decide(Duration::from_millis(1101)).state,
        State::Hidden
    );
}
