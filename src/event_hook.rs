//! Observe completed animation batches just before the engine resets their bounds.
#![cfg_attr(test, allow(dead_code))]
use crate::{attack_events, cue, reader};
use hudhook::mh;
use sha2::{Digest, Sha256};
use std::ffi::c_void;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    Mutex,
};
use std::time::Instant;

const CONSUMER_RVA: usize = 0xb5bef0;
const CONSUMER_SIZE: usize = 0xab;
const CONSUMER_SHA256: &str = "c8f27b1884982c7d4f6486251dd4f06b41a64e2e4a22bce946278f45d8cf909f";
type Consumer = unsafe extern "system" fn(usize) -> usize;
static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static ENABLED: AtomicBool = AtomicBool::new(false);
pub static CAPTURED: AtomicU64 = AtomicU64::new(0);
pub static DROPPED: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, PartialEq, Eq)]
struct Owner {
    module: usize,
    handle: u32,
    model: i32,
}
#[derive(Clone, Copy)]
struct Snapshot {
    at: Instant,
    frame: Result<cue::Animation, reader::ReadError>,
}
#[derive(Default)]
struct State {
    owner: Option<Owner>,
    generation: u64,
    latest: Option<Snapshot>,
}
impl State {
    fn select(&mut self, owner: Option<Owner>, now: Instant) -> Option<Snapshot> {
        if self.owner != owner {
            self.owner = owner;
            self.generation = self.generation.wrapping_add(1);
            self.latest = None;
        }
        self.latest
            .filter(|s| owner.is_some() && now.saturating_duration_since(s.at) < cue::FRESHNESS)
    }
}
static STATE: Mutex<State> = Mutex::new(State {
    owner: None,
    generation: 0,
    latest: None,
});

#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcess() -> *mut c_void;
    fn GetModuleHandleW(name: *const u16) -> *mut c_void;
    fn ReadProcessMemory(
        process: *mut c_void,
        address: *const c_void,
        buffer: *mut c_void,
        size: usize,
        copied: *mut usize,
    ) -> i32;
}
fn read(address: usize, output: &mut [u8]) -> bool {
    let mut copied = 0;
    unsafe {
        ReadProcessMemory(
            GetCurrentProcess(),
            address as *const c_void,
            output.as_mut_ptr().cast(),
            output.len(),
            &mut copied,
        ) != 0
            && copied == output.len()
    }
}

unsafe extern "system" fn consume(module: usize) -> usize {
    // Trampoline is published before enabling this detour and remains resident.
    let original: Consumer = unsafe { std::mem::transmute(ORIGINAL.load(Ordering::Acquire)) };
    // No allocation, logging, waiting, game calls, or direct pointer dereferences
    // in the observation path. A missed capture expires instead of blocking play.
    let _ = std::panic::catch_unwind(|| capture(module));
    unsafe { original(module) }
}
fn capture(module: usize) {
    let (owner, generation) = match STATE.try_lock() {
        Ok(state) => match state.owner {
            Some(owner) if owner.module == module => (owner, state.generation),
            _ => return,
        },
        Err(_) => {
            DROPPED.fetch_add(1, Ordering::Relaxed);
            return;
        }
    };
    let mut raw = [0; attack_events::BATCH_BYTES];
    let mut check = [0; attack_events::BATCH_BYTES];
    let frame = match module.checked_add(0x20) {
        Some(address) if read(address, &mut raw) && read(address, &mut check) && raw == check => {
            attack_events::select_batch(&raw, owner.model)
        }
        _ => Err(reader::ReadError::ChangedDuringRead),
    };
    if let Ok(mut state) = STATE.try_lock() {
        if state.owner == Some(owner) && state.generation == generation {
            state.latest = Some(Snapshot {
                at: Instant::now(),
                frame,
            });
            CAPTURED.fetch_add(1, Ordering::Relaxed);
        }
    } else {
        DROPPED.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Acquire)
}

/// Called only after executable identity and the *loaded* function bytes match.
pub fn install(hash: &str) -> Result<(), String> {
    if hash != reader::RESEARCH_HASH {
        return Err("unsupported executable; event hook disabled".into());
    }
    let base = unsafe { GetModuleHandleW(std::ptr::null()) } as usize;
    let address = base
        .checked_add(CONSUMER_RVA)
        .ok_or("invalid module base")?;
    let mut code = [0; CONSUMER_SIZE];
    if !read(address, &mut code) || format!("{:x}", Sha256::digest(code)) != CONSUMER_SHA256 {
        return Err(
            "event batch-boundary bytes differ from researched build; hook disabled".into(),
        );
    }
    unsafe { install_at(address) }
}
unsafe fn install_at(address: usize) -> Result<(), String> {
    let status = unsafe { mh::MH_Initialize() };
    if !matches!(
        status,
        mh::MH_STATUS::MH_OK | mh::MH_STATUS::MH_ERROR_ALREADY_INITIALIZED
    ) {
        return Err(format!("event hook initialization: {status:?}"));
    }
    let mut trampoline = std::ptr::null_mut();
    let status = unsafe {
        mh::MH_CreateHook(
            address as *mut c_void,
            consume as *const () as *mut c_void,
            &mut trampoline,
        )
    };
    if status != mh::MH_STATUS::MH_OK {
        return Err(format!("event hook creation: {status:?}"));
    }
    ORIGINAL.store(trampoline as usize, Ordering::Release);
    let status = unsafe { mh::MH_EnableHook(address as *mut c_void) };
    if status != mh::MH_STATUS::MH_OK {
        return Err(format!("event hook enable: {status:?}"));
    }
    ENABLED.store(true, Ordering::Release);
    Ok(())
}

/// Register only an independently validated locked target. Stale captures cannot
/// cross target changes, death, missing reads, or loss of lock.
pub fn apply(target: Option<&mut cue::Target>) -> &'static str {
    if !enabled() {
        return "event_hook_unavailable";
    }
    let Ok(mut state) = STATE.lock() else {
        if let Some(target) = target {
            target.animation_error = Some(reader::ReadError::ChangedDuringRead);
        }
        return "event_hook_state_failed";
    };
    let owner = target.as_ref().map(|t| Owner {
        module: t.animation_module,
        handle: t.handle,
        model: t.model,
    });
    let snapshot = state.select(owner, Instant::now());
    let Some(target) = target else {
        return "event_hook_no_target";
    };
    match snapshot {
        Some(Snapshot {
            frame: Ok(animation),
            ..
        }) => {
            target.animation = animation;
            target.animation_error = None;
            "event_batch"
        }
        Some(Snapshot {
            frame: Err(error), ..
        }) => {
            target.animation_error = Some(error);
            "event_batch_invalid"
        }
        None => {
            // Never silently mix polling with hooked timestamps once enabled.
            target.animation_error = Some(reader::ReadError::ChangedDuringRead);
            "event_batch_waiting_or_stale"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    static CALLS: AtomicUsize = AtomicUsize::new(0);
    #[inline(never)]
    unsafe extern "system" fn fixture(module: usize) -> usize {
        CALLS.fetch_add(1, Ordering::Relaxed);
        // Mimic the engine reset. The hook must capture BEFORE this write.
        unsafe {
            let head = std::ptr::read_unaligned((module + 0xe8) as *const i32);
            std::ptr::write_unaligned((module + 0xec) as *mut i32, head);
        }
        std::hint::black_box(module).wrapping_add(7)
    }
    #[test]
    fn rejects_unknown_build_before_hooking() {
        assert!(install("unknown").is_err());
    }
    #[test]
    fn captures_expire_and_cannot_cross_target_or_lock_changes() {
        let now = Instant::now();
        let owner = Owner {
            module: 0x10000,
            handle: 1,
            model: 1020,
        };
        let mut state = State {
            owner: Some(owner),
            generation: 0,
            latest: Some(Snapshot {
                at: now,
                frame: Err(reader::ReadError::InvalidAnimation),
            }),
        };
        assert!(state.select(Some(owner), now).is_some());
        assert!(state.select(Some(owner), now + cue::FRESHNESS).is_none());
        assert!(state
            .select(Some(Owner { handle: 2, ..owner }), now)
            .is_none());
        assert!(state.select(Some(owner), now).is_none());
        state.latest = Some(Snapshot {
            at: now,
            frame: Err(reader::ReadError::InvalidAnimation),
        });
        assert!(state.select(None, now).is_none());
        assert!(state.select(Some(owner), now).is_none());
    }
    #[test]
    fn native_detour_preserves_original_call_and_return() {
        let mut memory = [0_u8; 256];
        memory[0x20..0x24].copy_from_slice(&3000_i32.to_le_bytes());
        memory[0x24..0x28].copy_from_slice(&0.49_f32.to_le_bytes());
        memory[0x28..0x2c].copy_from_slice(&0.5_f32.to_le_bytes());
        memory[0x2c..0x30].copy_from_slice(&2.0_f32.to_le_bytes());
        memory[0xe8..0xec].copy_from_slice(&1_i32.to_le_bytes());
        let module = memory.as_mut_ptr() as usize;
        STATE.lock().unwrap().owner = Some(Owner {
            module,
            handle: 1,
            model: 1020,
        });
        let address = fixture as *const () as usize;
        unsafe {
            install_at(address).unwrap();
        }
        let call: Consumer = unsafe { std::mem::transmute(std::hint::black_box(address)) };
        assert_eq!(unsafe { call(module) }, module + 7);
        assert_eq!(CALLS.load(Ordering::Relaxed), 1);
        assert_eq!(
            STATE.lock().unwrap().latest.unwrap().frame.unwrap().id,
            3000
        );
        assert_eq!(&memory[0xe8..0xec], &memory[0xec..0xf0]);
        assert_eq!(unsafe { call(module) }, module + 7);
        assert_eq!(STATE.lock().unwrap().latest.unwrap().frame.unwrap().id, -1);
        STATE.lock().unwrap().owner = None;
        STATE.lock().unwrap().latest = None;
        unsafe {
            mh::MH_DisableHook(address as *mut c_void).ok().unwrap();
        }
        ENABLED.store(false, Ordering::Release);
    }
}
