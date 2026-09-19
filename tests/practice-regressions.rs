//! Synthetic ownership/speed memory. These do not validate in-game animation behavior.
use sekiro_deflect_observer::{
    attack::{self, Kind},
    config::Config,
    cue::{Animation, Target},
    practice::{eligible, Controller, SpeedMemory, Status},
    reader::{Memory, ReadError, RESEARCH_HASH},
    timing::{Engine, Settings, State},
};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    time::Duration,
};

const BASE: usize = 0x140000000;
const HANDLE: u32 = 0x10004001;
const SPEED: usize = 0x120000 + 0xd00;

#[derive(Default)]
struct Fixture {
    bytes: RefCell<BTreeMap<usize, u8>>,
    writes: RefCell<Vec<(usize, f32)>>,
    reads: Cell<usize>,
    unavailable: Cell<bool>,
    fail_write: Cell<bool>,
}
impl Fixture {
    fn put(&self, at: usize, bytes: &[u8]) {
        self.bytes
            .borrow_mut()
            .extend(bytes.iter().enumerate().map(|(i, b)| (at + i, *b)));
    }
    fn new(original: f32) -> Self {
        let fixture = Self::default();
        for (at, value) in [
            (BASE + 0x3d7a1e0, 0x20000u64),
            (0x20000 + 0x88, 0x30000),
            (0x20000 + 0x10, 0x70000),
            (0x20000 + 0x520, 0x80000),
            (0x80000 + 8, 0x90000),
            (0x90000 + 0x38, 0xa0000),
            (0xa0000 + 0x1ff8, 0xb0000),
            (0xb0000 + 0x10, 0xc0000),
            (0xb0000 + 0x18, 0xe0000),
            (0xb0000 + 0x28, 0x120000),
            (0xa0000 + 0x30, 0x130000),
            (0x30000 + 0x1ff8, 0xf0000),
            (0xf0000 + 0x18, 0x110000),
            (0xf0000 + 0x28, 0x140000),
        ] {
            fixture.put(at, &value.to_le_bytes());
        }
        for (at, value) in [
            (0x70000 + 0x18, 0i32),
            (0x80000, 2),
            (0xa0000 + 8, HANDLE as i32),
            (0xa0000 + 0x68, 10200000),
            (0x130000 + 0x628, 10200000),
            (0xe0000 + 0x130, 100),
            (0x110000 + 0x130, 100),
        ] {
            fixture.put(at, &value.to_le_bytes());
        }
        fixture.put(SPEED, &original.to_le_bytes());
        fixture
    }
    fn speed(&self) -> f32 {
        let mut bytes = [0; 4];
        self.read(SPEED, &mut bytes).unwrap();
        f32::from_le_bytes(bytes)
    }
}
impl Memory for Fixture {
    fn read(&self, at: usize, bytes: &mut [u8]) -> Result<(), ReadError> {
        self.reads.set(self.reads.get() + 1);
        if self.unavailable.get() {
            return Err(ReadError::Unreadable);
        }
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = *self
                .bytes
                .borrow()
                .get(&(at + i))
                .ok_or(ReadError::Unreadable)?;
        }
        Ok(())
    }
}
impl SpeedMemory for Fixture {
    fn write_speed(&self, at: usize, value: f32) -> Result<(), ReadError> {
        if self.fail_write.get() {
            return Err(ReadError::Unreadable);
        }
        self.writes.borrow_mut().push((at, value));
        self.put(at, &value.to_le_bytes());
        Ok(())
    }
}
fn target() -> Target {
    Target {
        metadata: Default::default(),
        captured_at: Some(Duration::from_secs(1)),
        player_instance: 0x30000,
        animation_module: 0xc0000,
        handle: HANDLE,
        model: 1020,
        npc_param: Some(10200000),
        animation: Animation {
            id: 3003,
            time: 0.1,
            previous: 0.08,
            sequence: 1,
        },
        position: [0.0; 3],
        anchor: [0.0; 3],
        facing: [0.0; 3],
        player_position: [0.0; 3],
        body_radius: 0.0,
        camera: None,
        animation_error: None,
    }
}
fn tick(
    controller: &mut Controller,
    fixture: &Fixture,
    enabled: bool,
    target: Option<&Target>,
) -> Status {
    controller
        .update(fixture, BASE, RESEARCH_HASH, enabled, target, 0.8)
        .status
}

#[test]
fn off_and_unsupported_build_never_read_or_write_speed() {
    let fixture = Fixture::new(1.0);
    let mut controller = Controller::default();
    assert_eq!(
        controller
            .update(&fixture, BASE, "different-exe", true, Some(&target()), 0.8)
            .status,
        Status::Unsupported
    );
    assert_eq!(
        tick(&mut controller, &fixture, false, Some(&target())),
        Status::Off
    );
    assert_eq!(fixture.reads.get(), 0);
    assert!(fixture.writes.borrow().is_empty());
}

#[test]
fn scales_existing_speed_once_and_restores_exact_original() {
    let fixture = Fixture::new(1.25);
    let mut controller = Controller::default();
    for _ in 0..20 {
        assert_eq!(
            tick(&mut controller, &fixture, true, Some(&target())),
            Status::Active
        );
    }
    assert_eq!(fixture.speed(), 1.0);
    assert_eq!(fixture.writes.borrow().as_slice(), &[(SPEED, 1.0)]);
    assert_eq!(controller.snapshot().percent, 80);
    assert_eq!(tick(&mut controller, &fixture, false, None), Status::Off);
    assert_eq!(fixture.speed(), 1.25);
    assert!(!controller.pending());
}

#[test]
fn ending_attack_or_losing_context_restores_without_disarming() {
    let fixture = Fixture::new(1.0);
    let mut controller = Controller::default();
    tick(&mut controller, &fixture, true, Some(&target()));
    assert_eq!(tick(&mut controller, &fixture, true, None), Status::Ready);
    assert_eq!(fixture.speed(), 1.0);
    assert_eq!(
        tick(&mut controller, &fixture, true, Some(&target())),
        Status::Active
    );
    assert_eq!(fixture.speed(), 0.8);
}

#[test]
fn configuration_change_restores_before_rescaling() {
    let fixture = Fixture::new(1.25);
    let mut controller = Controller::default();
    tick(&mut controller, &fixture, true, Some(&target()));
    assert_eq!(
        controller
            .update(&fixture, BASE, RESEARCH_HASH, true, Some(&target()), 0.6)
            .status,
        Status::Active
    );
    assert_eq!(
        fixture.writes.borrow().as_slice(),
        &[(SPEED, 1.0), (SPEED, 1.25), (SPEED, 0.75)]
    );
}

#[test]
fn descending_presets_restore_baseline_without_stacking() {
    let fixture = Fixture::new(1.25);
    let mut controller = Controller::default();
    for factor in [0.9, 0.8, 0.7, 0.6, 0.9] {
        let snapshot =
            controller.update(&fixture, BASE, RESEARCH_HASH, true, Some(&target()), factor);
        assert_eq!(snapshot.status, Status::Active);
        assert_eq!(snapshot.percent, (factor * 100.0).round() as u32);
        assert_eq!(fixture.speed(), 1.25 * factor);
        controller.update(&fixture, BASE, RESEARCH_HASH, true, Some(&target()), factor);
        assert_eq!(fixture.speed(), 1.25 * factor);
    }
    tick(&mut controller, &fixture, false, None);
    assert_eq!(
        fixture.writes.borrow().as_slice(),
        &[
            (SPEED, 1.125),
            (SPEED, 1.25),
            (SPEED, 1.0),
            (SPEED, 1.25),
            (SPEED, 0.875),
            (SPEED, 1.25),
            (SPEED, 0.75),
            (SPEED, 1.25),
            (SPEED, 1.125),
            (SPEED, 1.25),
        ]
    );
    for factor in [0.9, 0.8, 0.7, 0.6] {
        controller.update(
            &fixture,
            BASE,
            RESEARCH_HASH,
            false,
            Some(&target()),
            factor,
        );
    }
    assert_eq!(fixture.writes.borrow().len(), 10);
    assert_eq!(fixture.speed(), 1.25);
}

#[test]
fn lost_owner_or_reused_actor_never_restores_into_replacement() {
    for offset in [8, 0x68] {
        let fixture = Fixture::new(1.0);
        let mut controller = Controller::default();
        tick(&mut controller, &fixture, true, Some(&target()));
        fixture.put(0xa0000 + offset, &123i32.to_le_bytes());
        tick(&mut controller, &fixture, false, None);
        assert!(!controller.pending());
        assert_eq!(fixture.writes.borrow().len(), 1);
    }
}

#[test]
fn death_restores_but_never_starts_slowdown() {
    for at in [0xe0000 + 0x130, 0x110000 + 0x130] {
        let fixture = Fixture::new(1.0);
        let mut controller = Controller::default();
        tick(&mut controller, &fixture, true, Some(&target()));
        fixture.put(at, &0i32.to_le_bytes());
        assert_eq!(
            tick(&mut controller, &fixture, true, Some(&target())),
            Status::Unavailable
        );
        assert_eq!(fixture.speed(), 1.0);
        assert!(!controller.pending());
    }
}

#[test]
fn read_failure_preserves_cleanup_and_blocks_new_writes() {
    let fixture = Fixture::new(1.0);
    let mut controller = Controller::default();
    tick(&mut controller, &fixture, true, Some(&target()));
    fixture.unavailable.set(true);
    assert_eq!(
        tick(&mut controller, &fixture, false, None),
        Status::RestorePending
    );
    assert!(controller.pending());
    assert_eq!(fixture.writes.borrow().len(), 1);
    fixture.unavailable.set(false);
    assert_eq!(tick(&mut controller, &fixture, false, None), Status::Off);
    assert_eq!(fixture.speed(), 1.0);
}

#[test]
fn failed_restore_is_retried_even_if_attack_returns() {
    let fixture = Fixture::new(1.0);
    let mut controller = Controller::default();
    tick(&mut controller, &fixture, true, Some(&target()));
    fixture.fail_write.set(true);
    assert_eq!(
        tick(&mut controller, &fixture, true, None),
        Status::RestorePending
    );
    assert_eq!(
        tick(&mut controller, &fixture, true, Some(&target())),
        Status::RestorePending
    );
    fixture.fail_write.set(false);
    assert_eq!(tick(&mut controller, &fixture, false, None), Status::Off);
    assert_eq!(fixture.speed(), 1.0);
}

#[test]
fn external_speed_writer_is_respected_until_explicit_rearm() {
    let fixture = Fixture::new(1.0);
    let mut controller = Controller::default();
    tick(&mut controller, &fixture, true, Some(&target()));
    fixture.put(SPEED, &1.5f32.to_le_bytes());
    assert_eq!(
        tick(&mut controller, &fixture, true, Some(&target())),
        Status::Conflict
    );
    assert_eq!(
        tick(&mut controller, &fixture, true, None),
        Status::Conflict
    );
    assert_eq!(fixture.speed(), 1.5);
    assert_eq!(fixture.writes.borrow().len(), 1);
    tick(&mut controller, &fixture, false, None);
    assert_eq!(
        tick(&mut controller, &fixture, true, Some(&target())),
        Status::Active
    );
    assert!((fixture.speed() - 1.2).abs() < 0.00001);
}

#[test]
fn cannot_write_wolf_or_a_mismatched_npc_or_animation_owner() {
    let fixture = Fixture::new(1.0);
    let mut controller = Controller::default();
    let mut t = target();
    t.npc_param = Some(999);
    assert_eq!(
        tick(&mut controller, &fixture, true, Some(&t)),
        Status::Unavailable
    );
    let mut t = target();
    t.animation_module += 8;
    assert_eq!(
        tick(&mut controller, &fixture, true, Some(&t)),
        Status::Unavailable
    );
    fixture.put(0x90000 + 0x38, &0x30000u64.to_le_bytes());
    assert_eq!(
        tick(&mut controller, &fixture, true, Some(&target())),
        Status::Unavailable
    );
    assert!(fixture.writes.borrow().is_empty());
}

#[test]
fn enemy_modules_cannot_alias_wolfs_speed_control() {
    let fixture = Fixture::new(1.0);
    fixture.put(0xf0000 + 0x28, &0x120000u64.to_le_bytes());
    assert_eq!(
        tick(&mut Controller::default(), &fixture, true, Some(&target())),
        Status::Unavailable
    );
    assert!(fixture.writes.borrow().is_empty());
}

#[test]
fn switching_enemies_restores_previous_before_writing_new_owner() {
    let fixture = Fixture::new(1.25);
    let delta = 0x1000000;
    let old = fixture.bytes.borrow().clone();
    for range in [
        0xa0000..0xa3000,
        0xb0000..0xb0100,
        0xe0000..0xe0200,
        0x120000..0x121000,
        0x130000..0x131000,
    ] {
        for (&at, &byte) in old.range(range) {
            fixture.put(at + delta, &[byte]);
        }
    }
    for (at, value) in [
        (0xa0000 + 0x1ff8, 0xb0000),
        (0xa0000 + 0x30, 0x130000),
        (0xb0000 + 0x10, 0xc0000),
        (0xb0000 + 0x18, 0xe0000),
        (0xb0000 + 0x28, 0x120000),
    ] {
        fixture.put(at + delta, &((value + delta) as u64).to_le_bytes());
    }
    fixture.put(0x80000, &3i32.to_le_bytes());
    fixture.put(
        0x90000 + 2 * 0x38,
        &((0xa0000 + delta) as u64).to_le_bytes(),
    );
    fixture.put(0xa0000 + delta + 8, &(HANDLE + 1).to_le_bytes());
    fixture.put(SPEED + delta, &1.0f32.to_le_bytes());
    let mut other = target();
    other.handle += 1;
    other.animation_module += delta;
    let mut controller = Controller::default();
    tick(&mut controller, &fixture, true, Some(&target()));
    assert_eq!(
        tick(&mut controller, &fixture, true, Some(&other)),
        Status::Active
    );
    assert_eq!(
        fixture.writes.borrow().as_slice(),
        &[(SPEED, 1.0), (SPEED, 1.25), (SPEED + delta, 0.8)]
    );
    assert!(!controller.snapshot().active_for(&target()));
    assert!(controller.snapshot().active_for(&other));
    tick(&mut controller, &fixture, false, None);
    assert_eq!(fixture.writes.borrow().last(), Some(&(SPEED + delta, 1.0)));
}

#[test]
fn invalid_rates_and_baselines_do_not_write() {
    for factor in [0.0, 0.49, 1.0, 1.01, f32::NAN, f32::INFINITY] {
        let fixture = Fixture::new(1.0);
        Controller::default().update(&fixture, BASE, RESEARCH_HASH, true, Some(&target()), factor);
        assert!(fixture.writes.borrow().is_empty());
    }
    for baseline in [0.0, -1.0, 0.24, 4.01, f32::NAN, f32::INFINITY] {
        let fixture = Fixture::new(baseline);
        assert_eq!(
            tick(&mut Controller::default(), &fixture, true, Some(&target())),
            Status::Unavailable
        );
        assert!(fixture.writes.borrow().is_empty());
    }
}

#[test]
fn unknown_grab_stale_and_ended_attacks_are_ineligible() {
    let mut target = target();
    let now = Duration::from_secs(1);
    target.npc_param = None;
    assert!(eligible(&target, now));
    for (model, animation, kind) in [
        (1020, 3004, Kind::Unknown),
        (5020, 100003005, Kind::Grab),
        (5000, 3005, Kind::Unparryable),
    ] {
        let mut excluded = target.clone();
        excluded.model = model;
        excluded.animation.id = animation;
        assert_eq!(attack::classify(&excluded).unwrap().kind, kind);
        assert!(!eligible(&excluded, now));
    }
    assert!(!eligible(&target, now + Duration::from_millis(50)));
    assert!(!eligible(&target, Duration::ZERO));
    let mut ended = target.clone();
    ended.animation.time = 100.0;
    assert!(!eligible(&ended, now));
    target.animation_error = Some(ReadError::InvalidAnimation);
    assert!(!eligible(&target, now));
}

#[test]
fn hud_uses_slower_measured_animation_without_inventing_a_press_window() {
    let mut target = target();
    target.model = 1550;
    target.animation.id = 3003;
    let mut engine = Engine::default();
    let now = Duration::from_secs(1);
    target.animation.time = 0.4;
    engine.observe(now, Some(&target));
    let before = engine.incoming(now, [true; 3], true);
    target.animation.time += 0.1 * 0.8;
    target.captured_at = Some(now + Duration::from_millis(100));
    engine.observe(now + Duration::from_millis(100), Some(&target));
    let after = engine.incoming(now + Duration::from_millis(100), [true; 3], true);
    assert!((before.progress - 0.5).abs() < 0.001);
    assert!((after.progress - 0.6).abs() < 0.001);
    assert!(after.press.is_none() && after.contact.is_none());
    assert!(eligible(&target, now + Duration::from_millis(100)));
}

#[test]
fn alert_preferences_and_display_mode_cannot_change_an_active_speed_lease() {
    let now = Duration::from_secs(1);
    for (model, animation, kind) in [
        (1020, 3003, Kind::Parryable),
        (1550, 3003, Kind::Thrust),
        (5100, 100003009, Kind::Sweep),
    ] {
        let fixture = Fixture::new(1.0);
        fixture.put(0xa0000 + 0x68, &(model * 10000i32).to_le_bytes());
        let mut target = target();
        target.model = model;
        target.animation.id = animation;
        target.npc_param = None;
        let mut engine = Engine::default();
        engine.observe(now, Some(&target));
        let facts = attack::classify(&target).unwrap();
        assert_eq!(facts.kind, kind);
        let mut controller = Controller::default();
        for flags in 0..32 {
            let config = Config {
                parry: flags & 1 != 0,
                dodge: flags & 2 != 0,
                jump: flags & 4 != 0,
                mikiri: flags & 8 != 0,
                incoming_cues: flags & 16 != 0,
                ..Default::default()
            };
            let enabled = [config.parry, config.dodge, config.jump];
            let decision = if config.incoming_cues {
                engine.incoming(now, enabled, config.mikiri)
            } else {
                engine.decide_with(
                    now,
                    &Settings {
                        enabled,
                        ..Default::default()
                    },
                    &[],
                )
            };
            if !enabled.iter().any(|v| *v) && !config.mikiri {
                assert_eq!(decision.state, State::Neutral);
            }
            assert_eq!(attack::classify(&target), Some(facts));
            let request = eligible(&target, now).then_some(&target);
            assert_eq!(
                tick(&mut controller, &fixture, true, request),
                Status::Active
            );
            assert_eq!(fixture.speed(), 0.8);
        }
        assert_eq!(fixture.writes.borrow().as_slice(), &[(SPEED, 0.8)]);
        assert_eq!(tick(&mut controller, &fixture, false, None), Status::Off);
        assert_eq!(fixture.speed(), 1.0);
    }
}
