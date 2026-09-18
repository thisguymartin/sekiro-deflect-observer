//! A portable invalidation generation shared by reader, hotkeys and renderer.
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
#[derive(Default)]
pub struct Gate {
    generation: AtomicU64,
    invalidated_us: AtomicU64,
}
impl Gate {
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }
    pub fn invalidate(&self, at: Duration) {
        self.invalidated_us.store(
            at.as_micros().min(u64::MAX as u128) as u64,
            Ordering::Release,
        );
        self.generation.fetch_add(1, Ordering::AcqRel);
    }
    pub fn accepts(&self, snapshot_generation: u64) -> bool {
        self.generation() == snapshot_generation
    }
    pub fn invalidated_at(&self) -> Duration {
        Duration::from_micros(self.invalidated_us.load(Ordering::Acquire))
    }
}
