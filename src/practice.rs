//! Opt-in, temporary enemy animation speed. No global clock or player writes.
//! The ownership checks reduce races; they are not an atomic game snapshot.
use crate::cue::{Response, Target};
use crate::reader::{address, pointer, Memory, ReadError, RESEARCH_HASH};
use crate::timing::{Decision, State};
use std::time::Duration;

/// The native backend must check accessibility and the complete four-byte write.
pub trait SpeedMemory: Memory {
    fn write_speed(&self, address: usize, value: f32) -> Result<(), ReadError>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Status {
    #[default]
    Off,
    Ready,
    Active,
    Unavailable,
    RestorePending,
    Conflict,
    Unsupported,
}
impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Ready => "armed_waiting",
            Self::Active => "applied",
            Self::Unavailable => "speed_unavailable",
            Self::RestorePending => "restore_pending",
            Self::Conflict => "external_change_paused",
            Self::Unsupported => "unsupported_executable",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub status: Status,
    pub handle: u32,
    pub model: i32,
    pub animation_module: usize,
    pub percent: u32,
    pub original_bits: Option<u32>,
    pub applied_bits: Option<u32>,
}
impl Snapshot {
    pub fn active_for(self, target: &Target) -> bool {
        self.status == Status::Active
            && self.handle == target.handle
            && self.model == target.model
            && self.animation_module == target.animation_module
    }
}

/// Paired grabs and unresolved/no-parry effects stay normal in this first
/// prototype. An animation speed write cannot retime a projectile in flight.
pub fn eligible(target: &Target, now: Duration, decision: &Decision) -> bool {
    let captured = target
        .captured_at
        .or_else(|| (target.metadata.source == "poll").then_some(target.metadata.read_start));
    target.animation_error.is_none()
        && captured
            .and_then(|at| now.checked_sub(at))
            .is_some_and(|age| age < Duration::from_millis(50))
        && target.animation.time.is_finite()
        && target.animation.time >= 0.0
        && decision.animation_time == target.animation.time
        && matches!(decision.state, State::Incoming | State::AttackActive)
        && matches!(
            decision.response,
            Response::Parry | Response::Jump | Response::Mikiri
        )
        && decision.activation.is_some_and(|phase| {
            phase.start.is_finite()
                && phase.end.is_finite()
                && phase.start >= 0.0
                && phase.end > phase.start
                && target.animation.time < phase.end
        })
}

fn integer(memory: &impl Memory, base: usize, offset: usize) -> Result<i32, ReadError> {
    let mut bytes = [0; 4];
    memory.read(address(base, offset)?, &mut bytes)?;
    Ok(i32::from_le_bytes(bytes))
}
fn speed(memory: &impl Memory, at: usize) -> Result<f32, ReadError> {
    Ok(f32::from_bits(integer(memory, at, 0)? as u32))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Owner {
    world: usize,
    player: usize,
    actor: usize,
    handle: u32,
    character: i32,
    resource: usize,
    npc: i32,
    modules: usize,
    animation: usize,
    behavior: usize,
    data: usize,
    player_modules: usize,
    player_data: usize,
    player_behavior: usize,
}
impl Owner {
    fn resolve(memory: &impl Memory, base: usize, handle: u32) -> Result<Option<Self>, ReadError> {
        if handle >> 28 != 1 {
            return Ok(None);
        }
        let world = pointer(memory, base, 0x3d7a1e0)?;
        if world == 0 {
            return Ok(None);
        }
        let player = pointer(memory, world, 0x88)?;
        if player == 0 {
            return Ok(None);
        }
        let group = ((handle >> 14) & 63) as usize;
        let index = (handle & 16383) as usize;
        let info = pointer(memory, world, 0x10)?;
        let groups = integer(memory, info, 0x18)?.checked_add(3);
        if !groups.is_some_and(|n| (1..=64).contains(&n) && group < n as usize) {
            return Ok(None);
        }
        let bucket = pointer(memory, world, 0x518 + group * 8)?;
        if bucket == 0 {
            return Ok(None);
        }
        let count = integer(memory, bucket, 0)?;
        if !(0..=16384).contains(&count) || index >= count as usize {
            return Ok(None);
        }
        let array = pointer(memory, bucket, 8)?;
        let actor = pointer(memory, array, index * 0x38)?;
        if actor == 0 || actor == player || integer(memory, actor, 8)? as u32 != handle {
            return Ok(None);
        }
        let character = integer(memory, actor, 0x68)?;
        if !(1000..=7999).contains(&(character / 10000)) {
            return Ok(None);
        }
        let modules = pointer(memory, actor, 0x1ff8)?;
        let animation = pointer(memory, modules, 0x10)?;
        let behavior = pointer(memory, modules, 0x28)?;
        let data = pointer(memory, modules, 0x18)?;
        let resource = pointer(memory, actor, 0x30)?;
        let npc = integer(memory, resource, 0x628)?;
        let player_modules = pointer(memory, player, 0x1ff8)?;
        let player_data = pointer(memory, player_modules, 0x18)?;
        let player_behavior = pointer(memory, player_modules, 0x28)?;
        if [
            modules,
            animation,
            behavior,
            data,
            resource,
            player_modules,
            player_data,
        ]
        .contains(&0)
            || player_behavior == 0
            || behavior == player_behavior
            || modules == player_modules
        {
            return Ok(None);
        }
        Ok(Some(Self {
            world,
            player,
            actor,
            handle,
            character,
            resource,
            npc,
            modules,
            animation,
            behavior,
            data,
            player_modules,
            player_data,
            player_behavior,
        }))
    }
    fn matches(self, target: &Target) -> bool {
        self.player == target.player_instance
            && self.animation == target.animation_module
            && self.character / 10000 == target.model
            && target.npc_param.is_none_or(|npc| npc == self.npc)
    }
    fn alive(self, memory: &impl Memory) -> Result<bool, ReadError> {
        Ok(integer(memory, self.data, 0x130)? > 0 && integer(memory, self.player_data, 0x130)? > 0)
    }
    fn speed_address(self) -> Result<usize, ReadError> {
        address(self.behavior, 0xd00)
    }
}

#[derive(Clone, Copy)]
struct Lease {
    owner: Owner,
    original: f32,
    applied: f32,
    factor: f32,
    restoring: bool,
}

#[derive(Default)]
pub struct Controller {
    lease: Option<Lease>,
    suspended: bool,
    snapshot: Snapshot,
}
impl Controller {
    pub fn snapshot(&self) -> Snapshot {
        self.snapshot
    }
    pub fn pending(&self) -> bool {
        self.lease.is_some()
    }

    fn set_status(&mut self, status: Status) -> Snapshot {
        self.snapshot = self.lease.map_or(
            Snapshot {
                status,
                ..Snapshot::default()
            },
            |lease| Snapshot {
                status,
                handle: lease.owner.handle,
                model: lease.owner.character / 10000,
                animation_module: lease.owner.animation,
                percent: (lease.factor * 100.0).round() as u32,
                original_bits: Some(lease.original.to_bits()),
                applied_bits: Some(lease.applied.to_bits()),
            },
        );
        self.snapshot
    }

    // A missing/replaced owner drops the lease without touching a stale address.
    // Read/write failures keep restoration pending and block any new lease.
    fn restore(&mut self, memory: &impl SpeedMemory, base: usize) -> Result<(), Status> {
        let Some(mut lease) = self.lease else {
            return Ok(());
        };
        lease.restoring = true;
        self.lease = Some(lease);
        let owner =
            Owner::resolve(memory, base, lease.owner.handle).map_err(|_| Status::RestorePending)?;
        if owner != Some(lease.owner) {
            self.lease = None;
            return Ok(());
        }
        let at = lease
            .owner
            .speed_address()
            .map_err(|_| Status::RestorePending)?;
        let current = speed(memory, at).map_err(|_| Status::RestorePending)?;
        if current.to_bits() == lease.original.to_bits() {
            self.lease = None;
            return Ok(());
        }
        if current.to_bits() != lease.applied.to_bits() {
            self.lease = None;
            self.suspended = true;
            return Err(Status::Conflict);
        }
        if Owner::resolve(memory, base, lease.owner.handle).map_err(|_| Status::RestorePending)?
            != Some(lease.owner)
        {
            self.lease = None;
            return Ok(());
        }
        if speed(memory, at)
            .map_err(|_| Status::RestorePending)?
            .to_bits()
            != lease.applied.to_bits()
        {
            return Err(Status::RestorePending);
        }
        memory
            .write_speed(at, lease.original)
            .map_err(|_| Status::RestorePending)?;
        // Verify readback; a failed read keeps cleanup pending for another tick.
        if speed(memory, at)
            .map_err(|_| Status::RestorePending)?
            .to_bits()
            != lease.original.to_bits()
        {
            return Err(Status::RestorePending);
        }
        self.lease = None;
        Ok(())
    }

    /// `requested` is a fresh eligible target, or None to release an active
    /// attack. `enabled` is the session toggle, separate from focus/attack state.
    pub fn update(
        &mut self,
        memory: &impl SpeedMemory,
        base: usize,
        hash: &str,
        enabled: bool,
        requested: Option<&Target>,
        factor: f32,
    ) -> Snapshot {
        if hash != RESEARCH_HASH {
            return self.set_status(Status::Unsupported);
        }
        if !enabled {
            self.suspended = false;
        }
        let requested =
            requested.filter(|_| enabled && factor.is_finite() && (0.5..1.0).contains(&factor));
        let wanted = requested.and_then(|target| {
            Owner::resolve(memory, base, target.handle)
                .ok()
                .flatten()
                .filter(|owner| owner.matches(target) && owner.alive(memory) == Ok(true))
        });
        if let Some(lease) = self.lease {
            if lease.restoring
                || wanted != Some(lease.owner)
                || factor.to_bits() != lease.factor.to_bits()
            {
                if let Err(status) = self.restore(memory, base) {
                    return self.set_status(status);
                }
            } else {
                let current = lease.owner.speed_address().and_then(|at| speed(memory, at));
                match current {
                    Ok(value) if value.to_bits() == lease.applied.to_bits() => {
                        return self.set_status(Status::Active)
                    }
                    Ok(_) => {
                        self.lease = None;
                        self.suspended = true;
                    }
                    Err(_) => {
                        if let Some(lease) = self.lease.as_mut() {
                            lease.restoring = true;
                        }
                        return self.set_status(Status::RestorePending);
                    }
                }
            }
        }
        if self.suspended {
            return self.set_status(Status::Conflict);
        }
        let Some(owner) = wanted else {
            return self.set_status(if !enabled {
                Status::Off
            } else if requested.is_some() {
                Status::Unavailable
            } else {
                Status::Ready
            });
        };
        let Ok(at) = owner.speed_address() else {
            return self.set_status(Status::Unavailable);
        };
        let Ok(original) = speed(memory, at) else {
            return self.set_status(Status::Unavailable);
        };
        if !original.is_finite()
            || !(0.25..=4.0).contains(&original)
            || Owner::resolve(memory, base, owner.handle) != Ok(Some(owner))
            || owner.alive(memory) != Ok(true)
            || speed(memory, at).map(f32::to_bits) != Ok(original.to_bits())
        {
            return self.set_status(Status::Unavailable);
        }
        let applied = original * factor;
        // Keep ownership before attempting the write, including an uncertain failure.
        self.lease = Some(Lease {
            owner,
            original,
            applied,
            factor,
            restoring: false,
        });
        if memory.write_speed(at, applied).is_err()
            || speed(memory, at).map(f32::to_bits) != Ok(applied.to_bits())
        {
            self.lease.as_mut().unwrap().restoring = true;
            return self.set_status(Status::RestorePending);
        }
        self.set_status(Status::Active)
    }
}
