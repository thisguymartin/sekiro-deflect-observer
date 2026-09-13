//! Hash-gated target/camera reads and an explicitly experimental attack cue.
//! Attack activation is used as a timing estimate, not a proven contact time.
use crate::reader::{address, pointer, Memory, ReadError, RESEARCH_HASH};
use std::collections::BTreeSet;
use std::time::Duration;

pub const FRESHNESS: Duration = Duration::from_millis(50);
const LEAD: f32 = 0.150;

/// Diagnostic stages distinguish a lost lock from a failed dependent read.
#[derive(Clone, Debug, Default)]
pub struct ReadTrace {
    pub stage: &'static str,
    pub lock_enabled: Option<bool>,
    pub points: usize,
    pub selected: bool,
    pub animation_error: Option<ReadError>,
}

const MAX_DISTANCE: f32 = 3.0;

fn bytes<const N: usize>(
    m: &impl Memory,
    base: usize,
    offset: usize,
) -> Result<[u8; N], ReadError> {
    let mut result = [0; N];
    m.read(address(base, offset)?, &mut result)?;
    Ok(result)
}
fn integer(m: &impl Memory, base: usize, offset: usize) -> Result<i32, ReadError> {
    Ok(i32::from_le_bytes(bytes(m, base, offset)?))
}
fn vector(m: &impl Memory, base: usize, offset: usize) -> Result<[f32; 3], ReadError> {
    let raw = bytes::<12>(m, base, offset)?;
    let result =
        std::array::from_fn(|i| f32::from_le_bytes(raw[i * 4..i * 4 + 4].try_into().unwrap()));
    if result
        .iter()
        .any(|v| !v.is_finite() || v.abs() > 1_000_000.0)
    {
        return Err(ReadError::InvalidAddress);
    }
    Ok(result)
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
fn subtract(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Animation {
    pub id: i32,
    pub previous: f32,
    pub time: f32,
    pub sequence: i32,
}
pub fn animation(m: &impl Memory, module: usize) -> Result<Animation, ReadError> {
    let head = integer(m, module, 0xe8)?;
    if !(0..10).contains(&head) {
        return Err(ReadError::InvalidAnimation);
    }
    let raw = bytes::<20>(m, module, 0x20 + ((head + 9) % 10) as usize * 0x14)?;
    if integer(m, module, 0xe8)? != head {
        return Err(ReadError::ChangedDuringRead);
    }
    decode_animation(raw)
}

fn decode_animation(raw: [u8; 20]) -> Result<Animation, ReadError> {
    let id = i32::from_le_bytes(raw[0..4].try_into().unwrap());
    let previous = f32::from_le_bytes(raw[4..8].try_into().unwrap());
    let time = f32::from_le_bytes(raw[8..12].try_into().unwrap());
    let duration = f32::from_le_bytes(raw[12..16].try_into().unwrap());
    let sequence = i32::from_le_bytes(raw[16..20].try_into().unwrap());
    if id < 0 || duration == 0.0 {
        return Ok(Animation {
            id: -1,
            previous: 0.0,
            time: 0.0,
            sequence,
        });
    }
    if ![previous, time, duration].iter().all(|v| v.is_finite())
        || !(0.0..=3600.0).contains(&time)
        || !(0.0..=3600.0).contains(&previous)
        || duration <= 0.0
        || duration > 3600.0
    {
        return Err(ReadError::InvalidAnimation);
    }
    Ok(Animation {
        id,
        previous,
        time,
        sequence,
    })
}

/// Read only the engine's current submission batch, [module+ec, module+e8).
/// Loaded 0xb5bef0 advances the batch boundary; 0xb5c730 consumes this range.
/// Multiple tracks can be submitted per frame (Ogre's trailing 40000 loop).
/// Never search older history for a convenient attack or use a previous batch.
pub fn animation_for_model(
    m: &impl Memory,
    module: usize,
    model: i32,
) -> Result<Animation, ReadError> {
    let bounds = bytes::<8>(m, module, 0xe8)?;
    let head = i32::from_le_bytes(bounds[..4].try_into().unwrap());
    let begin = i32::from_le_bytes(bounds[4..].try_into().unwrap());
    if !(0..10).contains(&head) || !(0..10).contains(&begin) {
        return Err(ReadError::InvalidAnimation);
    }
    let count = ((head - begin + 10) % 10) as usize;
    let mut records = Vec::with_capacity(count);
    let mut latest = Animation {
        id: -1,
        previous: 0.0,
        time: 0.0,
        sequence: 0,
    };
    let mut selected: Option<Animation> = None;
    for step in 0..count {
        let slot = 0x20 + ((begin as usize + step) % 10) * 0x14;
        let raw = bytes::<20>(m, module, slot)?;
        let frame = decode_animation(raw)?;
        records.push((slot, raw));
        latest = frame;
        let key = (model, frame.id);
        let relevant = crate::attack_timings::ATTACKS
            .binary_search_by_key(&key, |a| (a.0, a.1))
            .is_ok()
            || crate::attack_timings::SPECIALS
                .binary_search_by_key(&key, |a| (a.0, a.1))
                .is_ok();
        if relevant {
            if selected.is_some_and(|prior| prior.id != frame.id) {
                // Blend/competing attacks need more than a guessed priority.
                return Err(ReadError::InvalidAnimation);
            }
            selected = Some(frame);
        }
    }
    // Recheck bytes as well as indices: the ring could wrap during a read.
    for (slot, raw) in records {
        if bytes::<20>(m, module, slot)? != raw {
            return Err(ReadError::ChangedDuringRead);
        }
    }
    if bytes::<8>(m, module, 0xe8)? != bounds {
        return Err(ReadError::ChangedDuringRead);
    }
    Ok(selected.unwrap_or(latest))
}

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub right: [f32; 3],
    pub up: [f32; 3],
    pub forward: [f32; 3],
    pub position: [f32; 3],
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}
impl Camera {
    fn valid(self) -> bool {
        let axes = [self.right, self.up, self.forward];
        self.position.iter().all(|v| v.is_finite())
            && axes
                .iter()
                .all(|a| a.iter().all(|v| v.is_finite()) && (dot(*a, *a) - 1.0).abs() < 0.02)
            && [(0, 1), (0, 2), (1, 2)]
                .into_iter()
                .all(|(i, j)| dot(axes[i], axes[j]).abs() < 0.02)
            && self.fov.is_finite()
            && (0.1..3.0).contains(&self.fov)
            && self.aspect.is_finite()
            && (16.0 / 9.0 - 0.01..8.0).contains(&self.aspect)
            && self.near.is_finite()
            && self.far.is_finite()
            && self.near > 0.0
            && self.far > self.near
    }
    pub fn project(self, anchor: [f32; 3], display: [f32; 2]) -> Option<[f32; 2]> {
        if !self.valid()
            || !anchor.iter().all(|v| v.is_finite())
            || !display.iter().all(|v| v.is_finite() && *v > 0.0)
            || (display[0] / display[1] - self.aspect).abs() > 0.03
        {
            return None;
        }
        let delta = subtract(anchor, self.position);
        let z = dot(delta, self.forward);
        if z <= self.near || z >= self.far {
            return None;
        }
        let tangent = (self.fov / 2.0).tan();
        let x = dot(delta, self.right) / (z * tangent * self.aspect);
        let y = dot(delta, self.up) / (z * tangent);
        if !(-1.0..1.0).contains(&x) || !(-1.0..1.0).contains(&y) {
            return None;
        }
        Some([(x + 1.0) * 0.5 * display[0], (1.0 - y) * 0.5 * display[1]])
    }
}

#[derive(Clone, Debug)]
pub struct Target {
    pub handle: u32,
    pub model: i32,
    pub animation: Animation,
    pub position: [f32; 3],
    pub anchor: [f32; 3],
    pub facing: [f32; 3],
    pub player_position: [f32; 3],
    pub body_radius: f32,
    pub camera: Camera,
    pub animation_error: Option<ReadError>,
}

pub fn observe(m: &impl Memory, base: usize, hash: &str) -> Result<Option<Target>, ReadError> {
    observe_traced(m, base, hash, &mut ReadTrace::default())
}

pub fn observe_traced(
    m: &impl Memory,
    base: usize,
    hash: &str,
    trace: &mut ReadTrace,
) -> Result<Option<Target>, ReadError> {
    *trace = ReadTrace::default();
    trace.stage = "build";
    if hash != RESEARCH_HASH {
        return Err(ReadError::UnsupportedBuild);
    }
    trace.stage = "player_unavailable";
    let world = pointer(m, base, 0x3d7a1e0)?;
    if world == 0 {
        return Ok(None);
    }
    let player = pointer(m, world, 0x88)?;
    if player == 0 {
        return Ok(None);
    }
    trace.stage = "lock_manager_unavailable";
    let manager = pointer(m, base, 0x3d78058)?;
    if manager == 0 {
        return Ok(None);
    }
    trace.stage = "lock_disabled";
    let enabled = bytes::<1>(m, manager, 0x2830)?[0] != 0;
    trace.lock_enabled = Some(enabled);
    if !enabled {
        return Ok(None);
    }
    trace.stage = "selected_point_missing";
    let head = pointer(m, manager, 0x10)?;
    let mut point = head;
    let mut seen = BTreeSet::new();
    while point != 0 {
        if !seen.insert(point) {
            return Err(ReadError::Cycle);
        }
        trace.points = seen.len();
        if seen.len() > 128 {
            return Err(ReadError::TooManyNodes);
        }
        if bytes::<1>(m, point, 0x90)?[0] & 0x20 != 0 {
            break;
        }
        point = pointer(m, point, 0x80)?;
    }
    if point == 0 {
        return Ok(None);
    }
    trace.selected = true;
    trace.stage = "target_accessor";
    let accessor = pointer(m, point, 0x88)?;
    let handle = integer(m, accessor, 0x78)? as u32;
    if handle >> 28 != 1 {
        trace.stage = "non_character_lock";
        return Ok(None);
    }
    trace.stage = "character_bucket";
    let group = ((handle >> 14) & 63) as usize;
    let index = (handle & 16383) as usize;
    let info = pointer(m, world, 0x10)?;
    let groups = integer(m, info, 0x18)?
        .checked_add(3)
        .ok_or(ReadError::InvalidAddress)?;
    if !(1..=64).contains(&groups) || group >= groups as usize {
        return Err(ReadError::InvalidAddress);
    }
    let bucket = pointer(m, world, 0x518 + group * 8)?;
    let count = integer(m, bucket, 0)?;
    // The handle allocates 14 bits to the bucket index. This reads one slot,
    // not all actors; the previous arbitrary 256-slot cap could reject larger buckets.
    if !(0..=16384).contains(&count) || index >= count as usize {
        return Err(ReadError::InvalidAddress);
    }
    let array = pointer(m, bucket, 8)?;
    let actor = pointer(m, array, index * 0x38)?;
    if actor == player || integer(m, actor, 8)? as u32 != handle {
        return Err(ReadError::ChangedDuringRead);
    }
    trace.stage = "character_modules";
    let modules = pointer(m, actor, 0x1ff8)?;
    let animation_module = pointer(m, modules, 0x10)?;
    let physics = pointer(m, modules, 0x68)?;
    let data = pointer(m, modules, 0x18)?;
    let player_modules = pointer(m, player, 0x1ff8)?;
    let player_physics = pointer(m, player_modules, 0x68)?;
    let player_data = pointer(m, player_modules, 0x18)?;
    trace.stage = "target_dead";
    if integer(m, data, 0x130)? <= 0 {
        return Ok(None);
    }
    trace.stage = "player_dead";
    if integer(m, player_data, 0x130)? <= 0 {
        return Ok(None);
    }
    trace.stage = "animation";
    let model = integer(m, actor, 0x68)? / 10000;
    let mut frame = animation_for_model(m, animation_module, model);
    if frame == Err(ReadError::ChangedDuringRead) {
        frame = animation_for_model(m, animation_module, model);
    }
    // Keep a freshly resolved lock visible if only its animation is unavailable.
    // There is no history fallback and no green cue for a failed animation read.
    let animation_error = frame.as_ref().err().copied();
    trace.animation_error = animation_error;
    let animation = frame.unwrap_or(Animation {
        id: -1,
        previous: 0.0,
        time: 0.0,
        sequence: 0,
    });
    trace.stage = "positions";
    let position = vector(m, physics, 0x80)?;
    let player_position = vector(m, player_physics, 0x80)?;
    let bounds_min = vector(m, accessor, 0x40)?;
    let bounds_max = vector(m, accessor, 0x50)?;
    let extents = subtract(bounds_max, bounds_min);
    // Approximate distance from the body's horizontal bounds, so large bosses
    // are not measured as if their root were the edge of a soldier's body.
    let body_radius = if extents.iter().all(|v| (0.0..=100.0).contains(v)) {
        (0.5 * (extents[0] * extents[0] + extents[2] * extents[2]).sqrt()).min(6.0)
    } else {
        0.0
    };
    // Lower standing approximation, based on recording 02's excessive gap.
    // A head-bone anchor remains future work for crouches and acrobatics.
    let anchor = player_anchor(player_position);
    let facing = vector(m, accessor, 0x30)?;
    trace.stage = "camera";
    let field = pointer(m, base, 0x3d5c0a0)?;
    let render = pointer(m, field, 0x20)?;
    if integer(m, render, 0xe0)? != 0 {
        trace.stage = "debug_camera";
        return Ok(None);
    }
    let cam = pointer(m, field, 0x30)?;
    let lens = bytes::<16>(m, cam, 0x50)?;
    let lens: [f32; 4] =
        std::array::from_fn(|i| f32::from_le_bytes(lens[i * 4..i * 4 + 4].try_into().unwrap()));
    let camera = Camera {
        right: vector(m, cam, 0x10)?,
        up: vector(m, cam, 0x20)?,
        forward: vector(m, cam, 0x30)?,
        position: vector(m, cam, 0x40)?,
        fov: lens[0],
        aspect: lens[1],
        near: lens[2],
        far: lens[3],
    };
    if !camera.valid() {
        return Err(ReadError::InvalidCamera);
    }
    trace.stage = "ownership_recheck";
    if pointer(m, base, 0x3d7a1e0)? != world
        || pointer(m, world, 0x88)? != player
        || pointer(m, base, 0x3d78058)? != manager
        || pointer(m, manager, 0x10)? != head
        || bytes::<1>(m, manager, 0x2830)?[0] == 0
        || bytes::<1>(m, point, 0x90)?[0] & 0x20 == 0
        || pointer(m, point, 0x88)? != accessor
        || integer(m, accessor, 0x78)? as u32 != handle
        || pointer(m, world, 0x518 + group * 8)? != bucket
        || pointer(m, bucket, 8)? != array
        || pointer(m, array, index * 0x38)? != actor
        || integer(m, actor, 8)? as u32 != handle
        || pointer(m, actor, 0x1ff8)? != modules
        || pointer(m, modules, 0x10)? != animation_module
        || pointer(m, modules, 0x68)? != physics
        || pointer(m, modules, 0x18)? != data
        || pointer(m, player, 0x1ff8)? != player_modules
        || pointer(m, player_modules, 0x68)? != player_physics
        || pointer(m, player_modules, 0x18)? != player_data
        || pointer(m, base, 0x3d5c0a0)? != field
        || pointer(m, field, 0x30)? != cam
        || pointer(m, field, 0x20)? != render
        || integer(m, render, 0xe0)? != 0
    {
        return Err(ReadError::ChangedDuringRead);
    }
    trace.stage = if animation_error.is_some() {
        "locked_animation_unavailable"
    } else {
        "locked"
    };
    Ok(Some(Target {
        handle,
        model,
        animation,
        position,
        anchor,
        facing,
        player_position,
        body_radius,
        camera,
        animation_error,
    }))
}

pub fn player_anchor(position: [f32; 3]) -> [f32; 3] {
    [position[0], position[1] + 1.55, position[2]]
}

fn in_reach(target: &Target) -> bool {
    let delta = subtract(target.player_position, target.position);
    let distance = dot(delta, delta).sqrt();
    let horizontal = (delta[0] * delta[0] + delta[2] * delta[2]).sqrt();
    let facing_length =
        (target.facing[0] * target.facing[0] + target.facing[2] * target.facing[2]).sqrt();
    distance.is_finite()
        && target.body_radius.is_finite()
        && (0.0..=6.0).contains(&target.body_radius)
        && distance <= MAX_DISTANCE + target.body_radius
        && delta[1].abs() <= 1.5
        && horizontal >= 0.1
        && (0.9..=1.1).contains(&facing_length)
        && (delta[0] * target.facing[0] + delta[2] * target.facing[2])
            / (horizontal * facing_length)
            >= 0.3
}

#[derive(Clone, Copy, Debug)]
pub struct Timeline {
    pub start: f32,
    pub end: f32,
    pub progress: f32,
    pub green_start: f32,
    pub green_end: f32,
    pub remaining: f32,
    pub in_reach: bool,
    pub press_now: bool,
    pub classified: bool,
    pub response: Response,
    pub response_now: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Response {
    #[default]
    Unverified,
    Parry,
    Dodge,
    Jump,
}
impl Response {
    pub fn label(self) -> &'static str {
        match self {
            Self::Unverified => "unverified",
            Self::Parry => "parry",
            Self::Dodge => "dodge",
            Self::Jump => "jump",
        }
    }
    fn lead(self) -> f32 {
        match self {
            Self::Dodge | Self::Jump => 0.300,
            _ => LEAD,
        }
    }
}

fn phase_response(target: &Target, start: f32, classified: bool) -> Response {
    let entries = crate::attack_timings::RESPONSES;
    let key = (target.model, target.animation.id);
    let first = entries.partition_point(|a| (a.0, a.1) < key);
    let last = entries.partition_point(|a| (a.0, a.1) <= key);
    match entries[first..last]
        .iter()
        .find(|a| a.2 == start)
        .map(|a| a.4)
    {
        Some(1) => Response::Dodge,
        Some(2) => Response::Jump,
        _ if classified => Response::Parry,
        _ => Response::Unverified,
    }
}

/// The live enemy clock triggers and drives each swing; no guard-input timer.
/// Retain the current swing through its active phase, then advance to the next
/// combo hit. Restarting/changing the animation naturally resets this timeline.
pub fn timeline(target: &Target) -> Option<Timeline> {
    if target.animation_error.is_some() {
        return None;
    }
    let time = target.animation.time;
    if !time.is_finite() || time < 0.0 {
        return None;
    }
    let entries = crate::attack_timings::ATTACKS;
    let key = (target.model, target.animation.id);
    let first = entries.partition_point(|a| (a.0, a.1) < key);
    let last = entries.partition_point(|a| (a.0, a.1) <= key);
    let matching = || entries[first..last].iter();
    // Anticipate the next activation even while the previous swing is recovering.
    let selected = matching()
        .find(|a| time < a.2 && time >= a.2 - phase_response(target, a.2, a.4).lead())
        .or_else(|| matching().find(|a| time < a.3))?;
    let &(_, _, start, end, classified) = selected;
    let response = phase_response(target, start, classified);
    let lead = response.lead();
    let previous_end = matching()
        .filter(|&&(_, _, other_start, other_end, _)| other_start < start && other_end <= start)
        .map(|entry| entry.3)
        .fold(0.0_f32, f32::max);
    // Zoom into the final 0.65 seconds of wind-up so the green zone remains legible.
    let begin = previous_end
        .min((start - lead).max(0.0))
        .max((start - 0.65).max(0.0));
    let display_end = end.min(start + 0.25);
    let span = display_end - begin;
    if span <= 0.0 {
        return None;
    }
    let in_reach = in_reach(target);
    Some(Timeline {
        start,
        end,
        progress: ((time - begin) / span).clamp(0.0, 1.0),
        green_start: ((start - lead - begin) / span).clamp(0.0, 1.0),
        green_end: ((start - begin) / span).clamp(0.0, 1.0),
        remaining: (start - time).max(0.0),
        in_reach,
        classified,
        response,
        response_now: response != Response::Unverified
            && in_reach
            && time >= start - lead
            && time < start,
        press_now: classified
            && response == Response::Parry
            && in_reach
            && time >= start - LEAD
            && time < start,
    })
}

pub fn estimated_press(target: &Target) -> bool {
    timeline(target).is_some_and(|phase| phase.press_now)
}

pub fn special_attack(target: &Target) -> bool {
    crate::attack_timings::SPECIALS
        .binary_search_by_key(&(target.model, target.animation.id), |a| (a.0, a.1))
        .is_ok()
}

pub fn attack_mapped(target: &Target) -> bool {
    crate::attack_timings::ATTACKS
        .binary_search_by_key(&(target.model, target.animation.id), |a| (a.0, a.1))
        .is_ok()
}

#[derive(Default)]
pub struct LiveCue {
    pub latest: Option<(Duration, Target)>,
    last_frame: Option<(u32, i32, i32, f32)>,
    progressed: Option<Duration>,
    pub trace: ReadTrace,
    pub read_error: Option<ReadError>,
}
impl LiveCue {
    pub fn push(&mut self, at: Duration, value: Result<Option<Target>, ReadError>) {
        self.read_error = value.as_ref().err().copied();
        match value {
            Ok(Some(target)) => {
                let frame = (
                    target.handle,
                    target.animation.id,
                    target.animation.sequence,
                    target.animation.time,
                );
                if self.last_frame != Some(frame) {
                    self.progressed = Some(at);
                }
                self.last_frame = Some(frame);
                self.latest = Some((at, target));
            }
            _ => {
                self.latest = None;
                self.last_frame = None;
                self.progressed = None;
            }
        }
    }
    pub fn advancing(&self, now: Duration) -> bool {
        self.current(now).is_some()
            && self
                .progressed
                .is_some_and(|at| now.saturating_sub(at) < FRESHNESS)
    }
    pub fn current(&self, now: Duration) -> Option<&Target> {
        let (at, target) = self.latest.as_ref()?;
        if now.saturating_sub(*at) >= FRESHNESS {
            return None;
        }
        Some(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::collections::BTreeMap;

    fn camera() -> Camera {
        Camera {
            right: [1.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            forward: [0.0, 0.0, 1.0],
            position: [0.0; 3],
            fov: 1.0,
            aspect: 16.0 / 9.0,
            near: 0.08,
            far: 10000.0,
        }
    }
    fn target() -> Target {
        Target {
            handle: 0x10004001,
            model: 1010,
            animation: Animation {
                id: 3000,
                previous: 0.59,
                time: 0.60,
                sequence: 123,
            },
            position: [0.0, 0.0, 2.0],
            anchor: [0.0, 1.8, 2.0],
            facing: [0.0, 0.0, -1.0],
            player_position: [0.0; 3],
            body_radius: 0.0,
            camera: camera(),
            animation_error: None,
        }
    }
    #[test]
    fn slider_tracks_windup_green_recovery_and_repeated_combo_hits() {
        let mut t = target();
        t.animation.time = 0.1;
        let early = timeline(&t).unwrap();
        assert!(early.progress < early.green_start);
        assert!(!early.press_now);
        t.animation.time = 0.6;
        let green = timeline(&t).unwrap();
        assert!(green.press_now);
        assert!(green.progress >= green.green_start && green.progress < green.green_end);
        t.animation.time = 0.72;
        let late = timeline(&t).unwrap();
        assert!(!late.press_now);
        assert!(late.progress > late.green_end);
        t.animation.id = 3008;
        t.animation.time = 1.08;
        let first = timeline(&t).unwrap();
        assert!(first.progress >= first.green_start && first.progress < first.green_end);
        assert!(!first.press_now); // this combo carries an unclassified effect event
        t.animation.time = 1.48;
        let second = timeline(&t).unwrap();
        assert!(second.start > first.start);
        assert!(second.progress >= second.green_start && second.progress < second.green_end);
        t.animation.id = 0;
        assert!(timeline(&t).is_none());
        t.animation.id = 3000;
        t.animation.time = 0.1;
        assert!(timeline(&t).unwrap().progress < early.green_start);
    }
    #[test]
    fn overhead_anchor_follows_player_independently_of_enemy() {
        assert_eq!(player_anchor([4.0, 10.0, 8.0]), [4.0, 11.55, 8.0]);
        let f = Fixture::target_layout();
        let t = observe(&f, 0x140000000, RESEARCH_HASH).unwrap().unwrap();
        assert_eq!(t.anchor, player_anchor(t.player_position));
        assert_ne!(t.anchor[2], t.position[2]);
    }
    #[test]
    fn coverage_contains_major_bosses_and_never_prompts_for_uncertain_phases() {
        for model in [
            5000, 5020, 5060, 5080, 5090, 5100, 5400, 7000, 7020, 7100, 7110, 7400, 1150, 1190,
            1240,
        ] {
            let attack = crate::attack_timings::ATTACKS
                .iter()
                .find(|a| a.0 == model)
                .unwrap();
            let mut t = target();
            t.model = model;
            t.animation.id = attack.1;
            t.animation.time = (attack.2 - 0.05).max(0.0);
            // Overlapping phases can supersede an individual event; the table
            // still must supply at least one mapped phase for this animation.
            assert!(timeline(&t).is_some());
        }
        let attack = crate::attack_timings::ATTACKS
            .iter()
            .find(|a| a.0 == 5200)
            .unwrap();
        let mut t = target();
        t.model = 5200;
        t.animation.id = attack.1;
        t.animation.time = attack.2 - 0.05;
        assert!(!estimated_press(&t));
    }
    #[test]
    fn wider_window_and_boss_body_distance_are_bounded() {
        let mut t = target();
        t.animation.time = 0.53;
        assert!(estimated_press(&t));
        let phase = timeline(&t).unwrap();
        assert!(phase.green_end - phase.green_start >= 0.15);
        t.position = [0.0, 0.0, 5.0];
        assert!(!estimated_press(&t));
        t.body_radius = 2.5;
        assert!(estimated_press(&t));
        t.body_radius = f32::NAN;
        assert!(!estimated_press(&t));
    }
    #[test]
    fn projection_handles_center_vertical_direction_and_invalid_views() {
        let c = camera();
        assert_eq!(
            c.project([0.0, 0.0, 5.0], [1920.0, 1080.0]),
            Some([960.0, 540.0])
        );
        assert!(c.project([0.0, 1.0, 5.0], [1920.0, 1080.0]).unwrap()[1] < 540.0);
        for point in [
            [0.0, 0.0, -5.0],
            [0.0, 0.0, 0.01],
            [100.0, 0.0, 5.0],
            [f32::NAN, 0.0, 5.0],
        ] {
            assert!(c.project(point, [1920.0, 1080.0]).is_none());
        }
        assert!(c.project([0.0, 0.0, 5.0], [1080.0, 1920.0]).is_none());
        let mut c = c;
        c.up = c.right;
        assert!(c.project([0.0, 0.0, 5.0], [1920.0, 1080.0]).is_none());
    }
    #[test]
    fn preview_never_uses_unknown_animation_or_out_of_range_or_away_facing_target() {
        let mut t = target();
        assert!(estimated_press(&t));
        t.animation.time = 0.4;
        assert!(!estimated_press(&t));
        t.animation.time = 0.8;
        assert!(!estimated_press(&t));
        t.animation.time = 0.6;
        t.animation.id = 999999;
        assert!(!estimated_press(&t));
        t.animation.id = 3000;
        t.model = 9999;
        assert!(!estimated_press(&t));
        t.model = 1010;
        t.position = [0.0, 0.0, 5.0];
        assert!(!estimated_press(&t));
        t.position = [0.0, 0.0, 2.0];
        t.facing = [0.0, 0.0, 1.0];
        assert!(!estimated_press(&t));
    }
    #[test]
    fn frozen_clock_errors_and_missing_targets_suppress_existing_cues() {
        let mut state = LiveCue::default();
        state.push(Duration::ZERO, Ok(Some(target())));
        assert!(state.current(Duration::from_millis(10)).is_some());
        state.push(Duration::from_millis(40), Ok(Some(target())));
        assert!(state.current(Duration::from_millis(50)).is_some());
        assert!(!state.advancing(Duration::from_millis(50)));
        let mut changed = target();
        changed.animation.time += 0.02;
        state.push(Duration::from_millis(60), Ok(Some(changed)));
        assert!(state.current(Duration::from_millis(70)).is_some());
        state.push(Duration::from_millis(75), Err(ReadError::Unreadable));
        assert!(state.current(Duration::from_millis(76)).is_none());
        state.push(Duration::from_millis(80), Ok(Some(target())));
        state.push(Duration::from_millis(85), Ok(None));
        assert!(state.current(Duration::from_millis(86)).is_none());
    }

    #[derive(Default)]
    struct Fixture {
        data: BTreeMap<usize, u8>,
        calls: Cell<usize>,
        change_head: bool,
        change_owner: bool,
    }
    impl Fixture {
        fn put(&mut self, at: usize, value: &[u8]) {
            for (i, b) in value.iter().enumerate() {
                self.data.insert(at + i, *b);
            }
        }
        fn ptr(&mut self, at: usize, value: usize) {
            self.put(at, &(value as u64).to_le_bytes());
        }
        fn int(&mut self, at: usize, value: i32) {
            self.put(at, &value.to_le_bytes());
        }
        fn vec(&mut self, at: usize, value: [f32; 3]) {
            self.put(
                at,
                &value
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect::<Vec<_>>(),
            );
        }
        fn target_layout() -> Self {
            let mut f = Self::default();
            for (at, value) in [
                (0x140000000 + 0x3d7a1e0, 0x20000),
                (0x20000 + 0x88, 0x30000),
                (0x140000000 + 0x3d78058, 0x40000),
                (0x40000 + 0x10, 0x50000),
                (0x50000 + 0x88, 0x60000),
                (0x20000 + 0x10, 0x70000),
                (0x20000 + 0x518 + 8, 0x80000),
                (0x80000 + 8, 0x90000),
                (0x90000 + 0x38, 0xa0000),
                (0xa0000 + 0x1ff8, 0xb0000),
                (0xb0000 + 0x10, 0xc0000),
                (0xb0000 + 0x68, 0xd0000),
                (0xb0000 + 0x18, 0xe0000),
                (0x30000 + 0x1ff8, 0xf0000),
                (0xf0000 + 0x68, 0x100000),
                (0xf0000 + 0x18, 0x110000),
                (0x140000000 + 0x3d5c0a0, 0x120000),
                (0x120000 + 0x20, 0x130000),
                (0x120000 + 0x30, 0x140000),
            ] {
                f.ptr(at, value);
            }
            f.put(0x40000 + 0x2830, &[1]);
            f.put(0x50000 + 0x90, &[0x20]);
            for (at, value) in [
                (0x60000 + 0x78, 0x10004001),
                (0x70000 + 0x18, 0),
                (0x80000, 2),
                (0xa0000 + 8, 0x10004001),
                (0xe0000 + 0x130, 195),
                (0x110000 + 0x130, 320),
                (0xa0000 + 0x68, 10100000),
                (0xc0000 + 0xe8, 1),
                (0xc0000 + 0xec, 0),
                (0x130000 + 0xe0, 0),
            ] {
                f.int(at, value);
            }
            f.put(
                0xc0000 + 0x20,
                &[
                    3000i32.to_le_bytes(),
                    0.59f32.to_le_bytes(),
                    0.60f32.to_le_bytes(),
                    2.0f32.to_le_bytes(),
                    123i32.to_le_bytes(),
                ]
                .concat(),
            );
            f.vec(0xd0000 + 0x80, [0.0, 0.0, 2.0]);
            f.vec(0x100000 + 0x80, [0.0; 3]);
            f.vec(0x60000 + 0x40, [-0.5, 0.0, 1.5]);
            f.vec(0x60000 + 0x50, [0.5, 1.7, 2.5]);
            f.vec(0x60000 + 0x30, [0.0, 0.0, -1.0]);
            let c = camera();
            f.vec(0x140000 + 0x10, c.right);
            f.vec(0x140000 + 0x20, c.up);
            f.vec(0x140000 + 0x30, c.forward);
            f.vec(0x140000 + 0x40, c.position);
            f.put(
                0x140000 + 0x50,
                &[c.fov, c.aspect, c.near, c.far]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect::<Vec<_>>(),
            );
            f
        }
    }
    impl Memory for Fixture {
        fn read(&self, at: usize, output: &mut [u8]) -> Result<(), ReadError> {
            self.calls.set(self.calls.get() + 1);
            if self.change_owner && at == 0x140000000 + 0x3d7a1e0 && self.calls.get() > 1 {
                output.copy_from_slice(&0x21000u64.to_le_bytes());
                return Ok(());
            }
            if self.change_head && at == 0x20000 + 0xe8 && self.calls.get() > 1 {
                output[..4].copy_from_slice(&1i32.to_le_bytes());
                for (i, b) in output.iter_mut().enumerate().skip(4) {
                    *b = *self.data.get(&(at + i)).ok_or(ReadError::Unreadable)?;
                }
                return Ok(());
            }
            for (i, b) in output.iter_mut().enumerate() {
                *b = *self.data.get(&(at + i)).ok_or(ReadError::Unreadable)?;
            }
            Ok(())
        }
    }
    #[test]
    fn latest_ring_entry_wraps_and_rejects_changes_and_unreadable_frames() {
        for head in 0..10i32 {
            let mut f = Fixture::default();
            f.put(0x20000 + 0xe8, &head.to_le_bytes());
            let entry = 0x20000 + 0x20 + ((head + 9) % 10) as usize * 0x14;
            let raw = [
                3000i32.to_le_bytes(),
                0.5f32.to_le_bytes(),
                0.6f32.to_le_bytes(),
                2.0f32.to_le_bytes(),
                123i32.to_le_bytes(),
            ]
            .concat();
            f.put(entry, &raw);
            assert_eq!(animation(&f, 0x20000).unwrap().time, 0.6);
            f.data.remove(&(entry + 3));
            assert_eq!(animation(&f, 0x20000), Err(ReadError::Unreadable));
        }
        let mut f = Fixture::default();
        f.put(0x20000 + 0xe8, &0i32.to_le_bytes());
        f.put(
            0x20000 + 0x20 + 9 * 0x14,
            &[
                3000i32.to_le_bytes(),
                0.5f32.to_le_bytes(),
                0.6f32.to_le_bytes(),
                2.0f32.to_le_bytes(),
                123i32.to_le_bytes(),
            ]
            .concat(),
        );
        f.change_head = true;
        assert_eq!(animation(&f, 0x20000), Err(ReadError::ChangedDuringRead));
    }
    fn ring_frame(f: &mut Fixture, slot: usize, id: i32, time: f32) {
        f.put(
            0x20020 + slot * 0x14,
            &[
                id.to_le_bytes(),
                (time - 0.016).to_le_bytes(),
                time.to_le_bytes(),
                7.3333335f32.to_le_bytes(),
                123i32.to_le_bytes(),
            ]
            .concat(),
        );
    }
    #[test]
    fn current_batch_finds_ogre_attack_before_auxiliary_and_never_revives_old_history() {
        for head in 0..10i32 {
            let mut f = Fixture::default();
            let begin = (head + 8) % 10;
            f.int(0x200e8, head);
            f.int(0x200ec, begin);
            ring_frame(&mut f, begin as usize, 100003005, 0.45);
            ring_frame(&mut f, ((head + 9) % 10) as usize, 40000, 2.7);
            assert_eq!(animation(&f, 0x20000).unwrap().id, 40000);
            let selected = animation_for_model(&f, 0x20000, 5020).unwrap();
            assert_eq!(selected.id, 100003005);
            assert_eq!(selected.time, 0.45);
            // The previous attack remains in the ring but is outside this batch.
            f.int(0x200ec, (head + 9) % 10);
            assert_eq!(animation_for_model(&f, 0x20000, 5020).unwrap().id, 40000);
            f.int(0x200ec, head);
            assert_eq!(animation_for_model(&f, 0x20000, 5020).unwrap().id, -1);
        }
    }
    #[test]
    fn animation_batch_rejects_competing_attacks_missing_bytes_and_changed_boundaries() {
        let mut f = Fixture::default();
        f.int(0x200e8, 2);
        f.int(0x200ec, 0);
        ring_frame(&mut f, 0, 100003005, 0.45);
        ring_frame(&mut f, 1, 100003000, 0.45);
        assert_eq!(
            animation_for_model(&f, 0x20000, 5020),
            Err(ReadError::InvalidAnimation)
        );
        ring_frame(&mut f, 1, 40000, 2.7);
        f.data.remove(&0x20022);
        assert_eq!(
            animation_for_model(&f, 0x20000, 5020),
            Err(ReadError::Unreadable)
        );
        ring_frame(&mut f, 0, 100003005, 0.45);
        f.calls.set(0);
        f.change_head = true;
        assert_eq!(
            animation_for_model(&f, 0x20000, 5020),
            Err(ReadError::ChangedDuringRead)
        );
        f.change_head = false;
        f.int(0x200ec, 10);
        assert_eq!(
            animation_for_model(&f, 0x20000, 5020),
            Err(ReadError::InvalidAnimation)
        );
    }
    #[test]
    fn unsupported_build_performs_no_game_reads() {
        let f = Fixture::default();
        assert!(matches!(
            observe(&f, 0x140000000, "unknown"),
            Err(ReadError::UnsupportedBuild)
        ));
        assert_eq!(f.calls.get(), 0);
    }
    #[test]
    fn trace_distinguishes_lock_loss_death_and_animation_failure() {
        let mut f = Fixture::target_layout();
        let mut trace = ReadTrace::default();
        f.put(0x40000 + 0x2830, &[0]);
        assert!(observe_traced(&f, 0x140000000, RESEARCH_HASH, &mut trace)
            .unwrap()
            .is_none());
        assert_eq!(trace.stage, "lock_disabled");
        assert_eq!(trace.lock_enabled, Some(false));
        f.put(0x40000 + 0x2830, &[1]);
        f.int(0xe0000 + 0x130, 0);
        assert!(observe_traced(&f, 0x140000000, RESEARCH_HASH, &mut trace)
            .unwrap()
            .is_none());
        assert_eq!(trace.stage, "target_dead");
        assert!(trace.selected);
        f.int(0xe0000 + 0x130, 195);
        f.int(0xc0000 + 0xe8, 12);
        let neutral = observe_traced(&f, 0x140000000, RESEARCH_HASH, &mut trace)
            .unwrap()
            .unwrap();
        assert_eq!(trace.stage, "locked_animation_unavailable");
        assert_eq!(neutral.animation_error, Some(ReadError::InvalidAnimation));
        assert!(timeline(&neutral).is_none());
    }
    #[test]
    fn large_buckets_use_handle_bound_and_still_check_the_selected_slot() {
        let mut f = Fixture::target_layout();
        f.int(0x80000, 512);
        assert!(observe(&f, 0x140000000, RESEARCH_HASH).unwrap().is_some());
        f.int(0x80000, 16385);
        assert!(observe(&f, 0x140000000, RESEARCH_HASH).is_err());
        f.int(0x80000, 1);
        assert!(observe(&f, 0x140000000, RESEARCH_HASH).is_err());
    }
    #[test]
    fn generated_phases_are_ordered_disjoint_and_specials_never_green() {
        let attacks = crate::attack_timings::ATTACKS;
        for pair in attacks.windows(2) {
            assert!((pair[0].0, pair[0].1) <= (pair[1].0, pair[1].1));
            if (pair[0].0, pair[0].1) == (pair[1].0, pair[1].1) {
                assert!(pair[0].3 < pair[1].2);
            }
        }
        for &(model, id, _, _, green) in attacks {
            if crate::attack_timings::SPECIALS
                .binary_search_by_key(&(model, id), |a| (a.0, a.1))
                .is_ok()
            {
                assert!(!green);
            }
        }
    }
    #[test]
    fn ogre_and_ape_responses_are_distinct_from_parry_and_expire() {
        for (model, animation, time, response) in [
            (5020, 100003005, 0.45, Response::Dodge),
            (5100, 3016, 1.6, Response::Dodge),
            (5100, 100003009, 1.05, Response::Jump),
        ] {
            let mut t = target();
            t.model = model;
            t.animation.id = animation;
            t.animation.time = time;
            let phase = timeline(&t).unwrap();
            assert_eq!(phase.response, response);
            assert!(phase.response_now);
            assert!(!phase.press_now);
            t.animation.time = phase.start;
            assert!(!timeline(&t).unwrap().response_now);
            t.position = [0.0, 0.0, 30.0];
            t.animation.time = time;
            assert!(!timeline(&t).unwrap().response_now);
            t.animation_error = Some(ReadError::InvalidAnimation);
            assert!(timeline(&t).is_none());
        }
    }
    #[test]
    fn quick_followup_lead_is_not_hidden_by_previous_recovery() {
        let mut found = false;
        for pair in crate::attack_timings::ATTACKS.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if (a.0, a.1) == (b.0, b.1) && b.2 - a.3 < LEAD {
                let mut t = target();
                t.model = b.0;
                t.animation.id = b.1;
                t.animation.time = (b.2 - LEAD).max(a.2 + 0.001) + 0.00001;
                if t.animation.time < a.3 {
                    assert_eq!(timeline(&t).unwrap().start, b.2);
                    found = true;
                    break;
                }
            }
        }
        assert!(found, "Expected a short-gap combo in the extracted data");
    }
    #[test]
    fn coherent_target_reads_reject_any_missing_byte_and_owner_change() {
        let f = Fixture::target_layout();
        let t = observe(&f, 0x140000000, RESEARCH_HASH).unwrap().unwrap();
        assert_eq!(t.handle, 0x10004001);
        assert_eq!(t.model, 1010);
        assert!(estimated_press(&t));
        for at in f.data.keys() {
            let mut failed = Fixture::target_layout();
            failed.data.remove(at);
            let result = observe(&failed, 0x140000000, RESEARCH_HASH);
            if (0xc0000..0xc0100).contains(at) {
                let neutral = result.unwrap().unwrap();
                assert!(neutral.animation_error.is_some());
                assert!(!estimated_press(&neutral));
            } else {
                assert!(result.is_err(), "missing {at:x}");
            }
        }
        let mut changed = Fixture::target_layout();
        changed.change_owner = true;
        assert!(matches!(
            observe(&changed, 0x140000000, RESEARCH_HASH),
            Err(ReadError::ChangedDuringRead)
        ));
        let mut mismatched = Fixture::target_layout();
        mismatched.int(0xa0000 + 8, 0x10004002);
        assert!(matches!(
            observe(&mismatched, 0x140000000, RESEARCH_HASH),
            Err(ReadError::ChangedDuringRead)
        ));
    }
}
