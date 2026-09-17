//! Worker-owned speed lease and bounded transition log. Never runs in DX11 or
//! the game's animation hook; no game functions, code patches or protection changes.
use super::diagnostics::LocalMemory;
use crate::{
    cue::Target,
    practice::{Controller, Snapshot},
};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Duration;

pub(super) struct Session {
    controller: Controller,
    base: usize,
    hash: String,
    log: BufWriter<File>,
    written: usize,
    previous: Option<Snapshot>,
}
impl Session {
    pub fn new(base: usize, hash: &str, dll_hash: &str, epoch_unix_us: u128, file: File) -> Self {
        let mut log = BufWriter::new(file);
        let header = format!("# version={}; executable_sha256={hash}; dll_sha256={dll_hash}; epoch_unix_us={epoch_unix_us}; evidence=checked_speed_write_not_measured_gameplay\nat_us,status,handle,model,animation_module,percent,original_speed,applied_speed\n",env!("CARGO_PKG_VERSION"));
        let _ = log.write_all(header.as_bytes());
        Self {
            controller: Controller::default(),
            base,
            hash: hash.into(),
            log,
            written: header.len(),
            previous: None,
        }
    }
    pub fn update(
        &mut self,
        at: Duration,
        enabled: bool,
        target: Option<&Target>,
        factor: f32,
    ) -> Snapshot {
        let snapshot = self.controller.update(
            &LocalMemory::new(),
            self.base,
            &self.hash,
            enabled,
            target,
            factor,
        );
        if self.previous != Some(snapshot) && self.written < 16 * 1024 * 1024 {
            let number =
                |bits: Option<u32>| bits.map_or(String::new(), |b| f32::from_bits(b).to_string());
            let row = format!(
                "{},{},{:x},{},{:x},{},{},{}\n",
                at.as_micros(),
                snapshot.status.label(),
                snapshot.handle,
                snapshot.model,
                snapshot.animation_module,
                snapshot.percent,
                number(snapshot.original_bits),
                number(snapshot.applied_bits)
            );
            if self.log.write_all(row.as_bytes()).is_ok() && self.log.flush().is_ok() {
                self.written += row.len();
            } else {
                self.written = 16 * 1024 * 1024;
            }
        }
        self.previous = Some(snapshot);
        snapshot
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        // Cleanup also runs if the worker unwinds. A forced process termination
        // frees the entire process; no on-disk game or save data was changed.
        for _ in 0..4 {
            self.controller
                .update(&LocalMemory::new(), self.base, &self.hash, false, None, 0.8);
            if !self.controller.pending() {
                break;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        if self.controller.pending() {
            super::report_failure("Practice speed restoration remains pending during worker shutdown; restart Sekiro before continuing.");
        }
        let _ = self.log.flush();
    }
}
