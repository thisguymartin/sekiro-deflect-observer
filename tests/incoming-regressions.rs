//! Real extracted move identities, synthetic clocks. No successful-hit claims.
use sekiro_deflect_observer::{
    cue::{Animation, Response, Target},
    reader::ReadError,
    timing::{Engine, State},
};
use std::time::Duration;

fn target(model: i32, animation: i32, time: f32) -> Target {
    Target {
        metadata: Default::default(),
        captured_at: Some(Duration::from_millis(1000)),
        player_instance: 0x30000,
        animation_module: 0xc0000,
        handle: 0x10004001,
        model,
        animation: Animation {
            id: animation,
            time,
            previous: time,
            sequence: 1,
        },
        position: [100.0, 50.0, 100.0],
        player_position: [0.0; 3],
        anchor: [0.0; 3],
        facing: [0.0, 0.0, 1.0],
        body_radius: 0.0,
        camera: None,
        animation_error: None,
    }
}

#[test]
fn first_attack_capture_warns_even_far_away_and_facing_away() {
    let mut engine = Engine::default();
    engine.observe(Duration::from_millis(1000), Some(&target(1020, 3003, 0.1)));
    let cue = engine.incoming(Duration::from_millis(1000), [true; 3], true);
    assert_eq!(cue.state, State::Incoming);
    assert_eq!(cue.response, Response::Parry);
    assert!(cue.rate.is_none()); // no three-frame rate/contact requirement
    assert!(cue.press.is_none() && cue.contact.is_none() && cue.preferred.is_none());
    assert!(!cue.pulse_emitted);
    assert_eq!(cue.pulse, 0.0);
}

#[test]
fn reference_marker_reaches_center_at_activation_and_resets_for_the_active_phase() {
    let mut engine = Engine::default();
    // c1550/3003 activates at 0.8 seconds and ends at 0.9 seconds.
    engine.observe(Duration::from_millis(1000), Some(&target(1550, 3003, 0.4)));
    let windup = engine.incoming(Duration::from_millis(1000), [true; 3], true);
    assert_eq!(windup.state, State::Incoming);
    assert!((windup.progress - 0.5).abs() < 0.001);
    let mut active = target(1550, 3003, 0.8);
    active.captured_at = Some(Duration::from_millis(1016));
    engine.observe(Duration::from_millis(1016), Some(&active));
    let cue = engine.incoming(Duration::from_millis(1016), [true; 3], true);
    assert_eq!(cue.state, State::AttackActive);
    assert!(cue.progress.abs() < 0.001);
    assert!(cue.contact.is_none() && cue.press.is_none());
    assert!(!cue.pulse_emitted);
}

#[test]
fn thrust_uses_mikiri_marker_and_falls_back_to_parry_when_skill_hint_disabled() {
    let mut engine = Engine::default();
    engine.observe(Duration::from_millis(1000), Some(&target(1550, 3003, 0.4)));
    let mikiri = engine.incoming(Duration::from_millis(1000), [true; 3], true);
    assert_eq!(mikiri.response, Response::Mikiri);
    assert_eq!(mikiri.state, State::Incoming);
    assert_eq!(
        engine
            .incoming(Duration::from_millis(1000), [true; 3], false)
            .response,
        Response::Parry
    );
    assert_eq!(
        engine
            .incoming(Duration::from_millis(1000), [false; 3], false)
            .state,
        State::Neutral
    );
}

#[test]
fn grabs_and_sweeps_keep_distinct_responses_without_reach_gating() {
    for (model, animation, response) in [
        (5020, 100003005, Response::Dodge),
        (5100, 100003009, Response::Jump),
    ] {
        let mut engine = Engine::default();
        engine.observe(
            Duration::from_millis(1000),
            Some(&target(model, animation, 0.1)),
        );
        let cue = engine.incoming(Duration::from_millis(1000), [true; 3], true);
        assert_eq!(cue.response, response);
        assert_eq!(cue.state, State::Incoming);
    }
}

#[test]
fn unresolved_sword_spear_variant_is_explicitly_unknown() {
    let mut engine = Engine::default();
    engine.observe(Duration::from_millis(1000), Some(&target(1020, 3004, 0.5)));
    let cue = engine.incoming(Duration::from_millis(1000), [true; 3], true);
    assert_eq!(cue.state, State::Incoming);
    assert_eq!(cue.response, Response::Unverified);
    assert_eq!(cue.reason, "incoming_response_unknown");
}

#[test]
fn active_attack_is_not_a_press_window_and_does_not_extend_into_recovery() {
    let mut engine = Engine::default();
    engine.observe(Duration::from_millis(1000), Some(&target(1550, 3003, 0.85)));
    let cue = engine.incoming(Duration::from_millis(1000), [true; 3], true);
    assert_eq!(cue.state, State::AttackActive);
    assert_eq!(cue.response, Response::Mikiri);
    assert!(cue.press.is_none() && cue.contact.is_none());
    let mut recovery = target(1550, 3003, 0.91);
    recovery.captured_at = Some(Duration::from_millis(1016));
    engine.observe(Duration::from_millis(1016), Some(&recovery));
    assert_eq!(
        engine
            .incoming(Duration::from_millis(1016), [true; 3], true)
            .state,
        State::Neutral
    );
}

#[test]
fn lost_failed_stale_or_switched_observations_never_reuse_an_attack() {
    let mut engine = Engine::default();
    engine.observe(Duration::from_millis(1000), Some(&target(1550, 3003, 0.4)));
    assert_eq!(
        engine
            .incoming(Duration::from_millis(1050), [true; 3], true)
            .state,
        State::Hidden
    );
    let mut failed = target(1550, 3003, 0.4);
    failed.animation_error = Some(ReadError::InvalidAnimation);
    engine.observe(Duration::from_millis(1050), Some(&failed));
    assert_eq!(
        engine
            .incoming(Duration::from_millis(1050), [true; 3], true)
            .state,
        State::Neutral
    );
    let mut switched = target(1020, 3003, 0.1);
    switched.handle += 1;
    switched.captured_at = Some(Duration::from_millis(1051));
    engine.observe(Duration::from_millis(1051), Some(&switched));
    assert_eq!(
        engine
            .incoming(Duration::from_millis(1051), [true; 3], true)
            .response,
        Response::Parry
    );
    engine.observe(Duration::from_millis(1052), None);
    assert_eq!(
        engine
            .incoming(Duration::from_millis(1052), [true; 3], true)
            .state,
        State::Hidden
    );
    engine.observe(Duration::from_millis(1060), Some(&switched));
    engine.clear();
    assert_eq!(
        engine
            .incoming(Duration::from_millis(1060), [true; 3], true)
            .state,
        State::Hidden
    );
}

#[test]
fn no_parry_label_is_reserved_for_resolved_deflection_disabled_rows() {
    let mut engine = Engine::default();
    engine.observe(Duration::from_millis(1000), Some(&target(5000, 3005, 0.3)));
    let cue = engine.incoming(Duration::from_millis(1000), [true; 3], true);
    assert_eq!(cue.state, State::Incoming);
    assert_eq!(cue.response, Response::Avoid);
    assert!(cue.press.is_none());
}

#[test]
fn incoming_mode_is_the_default_and_mikiri_can_be_disabled_independently() {
    let config = sekiro_deflect_observer::config::Config::default();
    assert!(config.incoming_cues);
    let parsed = sekiro_deflect_observer::config::Config::parse("mikiri = false").unwrap();
    assert!(parsed.incoming_cues);
    assert!(!parsed.mikiri);
    let legacy = sekiro_deflect_observer::config::Config::parse("incoming_cues = false").unwrap();
    assert!(!legacy.incoming_cues);
}

#[test]
fn combo_changes_response_per_hit_and_never_skips_a_disabled_first_hit() {
    let mut engine = Engine::default();
    engine.observe(Duration::from_millis(1000), Some(&target(1020, 3005, 0.1)));
    let first = engine.incoming(Duration::from_millis(1000), [true; 3], true);
    assert_eq!((first.phase, first.response), (Some(0), Response::Parry));
    assert_eq!(
        engine
            .incoming(Duration::from_millis(1000), [false, true, true], true)
            .state,
        State::Neutral
    );
    let mut second = target(1020, 3005, 0.55);
    second.captured_at = Some(Duration::from_millis(1016));
    engine.observe(Duration::from_millis(1016), Some(&second));
    let next = engine.incoming(Duration::from_millis(1016), [true; 3], true);
    assert_eq!((next.phase, next.response), (Some(1), Response::Mikiri));
    assert_eq!(next.state, State::Incoming);
    assert!(next.press.is_none());
}
