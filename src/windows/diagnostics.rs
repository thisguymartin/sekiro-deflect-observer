use crate::cue::{self, LiveCue};
use crate::reader::{self, Memory, ReadError};
use crate::samples::{History, Sample};
use std::ffi::c_void;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::mpsc::{self, SyncSender};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const POLL: Duration = Duration::from_millis(8);
const READ_BUDGET: Duration = Duration::from_millis(10);
const LOG_LIMIT: u64 = 16 * 1024 * 1024;

/// Render decisions are submitted by the DX11 callback, not inferred from reads.
#[derive(Clone, Debug, Default)]
pub(super) struct RenderSample {
    pub at: Duration,
    pub frame: u64,
    pub status: &'static str,
    pub handle: u32,
    pub model: i32,
    pub animation: i32,
    pub animation_time: f32,
    pub sequence: i32,
    pub sample_age_ms: f64,
    pub position: Option<[f32; 2]>,
    pub phase: Option<cue::Timeline>,
}
impl RenderSample {
    fn row(&self) -> String {
        let (x, y) = self.position.map_or((String::new(), String::new()), |p| {
            (format!("{:.2}", p[0]), format!("{:.2}", p[1]))
        });
        let phase = self.phase.map_or_else(
            || ",,,,,false,unverified,false".into(),
            |p| {
                format!(
                    "{:.6},{:.6},{:.6},{},{},{},{},{}",
                    p.start,
                    p.end,
                    p.progress,
                    p.in_reach,
                    p.classified,
                    p.press_now,
                    p.response.label(),
                    p.response_now
                )
            },
        );
        format!(
            "{},{},{},{:x},{},{},{:.6},{},{:.2},{},{},{}\n",
            self.at.as_micros(),
            self.frame,
            self.status,
            self.handle,
            self.model,
            self.animation,
            self.animation_time,
            self.sequence,
            self.sample_age_ms,
            x,
            y,
            phase
        )
    }
}

#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcess() -> *mut c_void;
    fn GetModuleHandleW(name: *const u16) -> *mut c_void;
    fn ReadProcessMemory(
        process: *mut c_void,
        address: *const c_void,
        buffer: *mut c_void,
        size: usize,
        read: *mut usize,
    ) -> i32;
}

struct LocalMemory {
    started: Instant,
}
impl Memory for LocalMemory {
    fn read(&self, address: usize, bytes: &mut [u8]) -> Result<(), ReadError> {
        if self.started.elapsed() >= READ_BUDGET {
            return Err(ReadError::BudgetExceeded);
        }
        let mut copied = 0;
        // Windows checks accessibility and copies to our owned buffer. No Rust
        // reference to game memory, writes, protection changes or game calls.
        let ok = unsafe {
            ReadProcessMemory(
                GetCurrentProcess(),
                address as *const c_void,
                bytes.as_mut_ptr().cast(),
                bytes.len(),
                &mut copied,
            )
        };
        if ok == 0 || copied != bytes.len() {
            return Err(ReadError::Unreadable);
        }
        if self.started.elapsed() >= READ_BUDGET {
            return Err(ReadError::BudgetExceeded);
        }
        Ok(())
    }
}

pub(super) struct Diagnostics {
    pub history: Arc<Mutex<History>>,
    pub cue: Arc<Mutex<LiveCue>>,
    pub epoch: Instant,
    pub log_ok: Arc<AtomicBool>,
    pub cue_log_ok: Arc<AtomicBool>,
    pub render_log_ok: Arc<AtomicBool>,
    pub render_dropped: Arc<std::sync::atomic::AtomicU64>,
    render_tx: SyncSender<RenderSample>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl Diagnostics {
    pub fn start(hash: String) -> std::io::Result<Self> {
        let history = Arc::new(Mutex::new(History::default()));
        let cue_state = Arc::new(Mutex::new(LiveCue::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let log_ok = Arc::new(AtomicBool::new(true));
        let cue_log_ok = Arc::new(AtomicBool::new(true));
        let render_log_ok = Arc::new(AtomicBool::new(true));
        let render_dropped = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let (render_tx, render_rx) = mpsc::sync_channel::<RenderSample>(512);
        let epoch = Instant::now();
        let path = super::log_path()?.with_extension("samples.csv");
        let file = File::create(path)?;
        let shared = Arc::clone(&history);
        let shared_cue = Arc::clone(&cue_state);
        let cue_file = File::create(super::log_path()?.with_extension("cue.csv"))?;
        let render_file = File::create(super::log_path()?.with_extension("render.csv"))?;
        let cue_logging_ok = Arc::clone(&cue_log_ok);
        let render_logging_ok = Arc::clone(&render_log_ok);
        let epoch_unix_us = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros();
        let stopping = Arc::clone(&stop);
        let logging = Arc::clone(&log_ok);
        let worker = thread::Builder::new().name("observer-reader".into()).spawn(move || {
            let mut log = BufWriter::new(file);
            let mut cue_log = BufWriter::new(cue_file);
            let mut render_log = BufWriter::new(render_file);
            let render_header = format!("# version={}; epoch_unix_us={epoch_unix_us}; evidence=draw_submission_not_present_or_contact\nat_us,frame,status,handle,model,animation_id,animation_time,sequence,sample_age_ms,anchor_x,anchor_y,phase_start,phase_end,progress,in_reach,classified,press_submitted,response,response_submitted\n",env!("CARGO_PKG_VERSION"));
            let mut render_written = render_header.len() as u64;
            let mut render_logging = render_log.write_all(render_header.as_bytes()).is_ok();
            let cue_header = format!("# version={}; executable_sha256={hash}; epoch_unix_us={epoch_unix_us}; timing=activation_estimate; contact=unvalidated\nstart_us,end_us,handle,model,animation_id,animation_time,sequence,estimated_press,reason,stage,lock_enabled,lock_points,point_selected,animation_error\n", env!("CARGO_PKG_VERSION"));
            let mut cue_written = cue_header.len() as u64;
            let mut cue_logging = cue_log.write_all(cue_header.as_bytes()).is_ok();
            let header = format!("# executable_sha256={hash}; candidate=105010; semantics=unvalidated\nstart_us,end_us,state,reason,player,effect_ids\n");
            let mut written = header.len() as u64;
            logging.store(log.write_all(header.as_bytes()).is_ok(), Ordering::Relaxed);
            let _ = cue_log.flush();
            let _ = render_log.flush();
            let _ = log.flush();
            let base = unsafe { GetModuleHandleW(std::ptr::null()) } as usize;
            let mut last_flush = Instant::now();
            while !stopping.load(Ordering::Relaxed) {
                let cycle_started = Instant::now();
                let started = cycle_started;
                let mut trace = cue::ReadTrace::default();
                let mut cue_result = cue::observe_traced(&LocalMemory { started }, base, &hash, &mut trace);
                if matches!(cue_result, Err(ReadError::ChangedDuringRead)) {
                    cue_result = cue::observe_traced(&LocalMemory { started }, base, &hash, &mut trace);
                }
                let cue_finished = epoch.elapsed();
                let at = started.duration_since(epoch);
                let cue_row = match &cue_result {
                    Ok(Some(target)) => format!("{},{},{:x},{},{},{:.6},{},{},", at.as_micros(),cue_finished.as_micros(),target.handle,target.model,target.animation.id,target.animation.time,target.animation.sequence,cue::estimated_press(target)),
                    Ok(None) => format!("{},{},,,,,,,No target",at.as_micros(),cue_finished.as_micros()),
                    Err(error) => format!("{},{},,,,,,,{}",at.as_micros(),cue_finished.as_micros(),error.label()),
                };
                let cue_row = format!("{cue_row},{},{},{},{},{}\n",trace.stage,trace.lock_enabled.map_or(String::new(),|b| b.to_string()),trace.points,trace.selected,trace.animation_error.map_or("",ReadError::label));
                // Publish before diagnostic file work; never hold the cue lock while drawing.
                if let Ok(mut state) = shared_cue.lock() { state.trace=trace; state.push(at,cue_result); } else { break; }
                if cue_logging {
                    if cue_written + cue_row.len() as u64 > LOG_LIMIT {
                        let _ = cue_log.flush();
                        cue_logging = false;
                    } else if cue_log.write_all(cue_row.as_bytes()).is_err() {
                        cue_logging = false;
                    } else { cue_written += cue_row.len() as u64; }
                }
                for submitted in render_rx.try_iter().take(512) {
                    if render_logging {
                        let row = submitted.row();
                        if render_written + row.len() as u64 > LOG_LIMIT || render_log.write_all(row.as_bytes()).is_err() { render_logging=false; }
                        else { render_written += row.len() as u64; }
                    }
                }
                cue_logging_ok.store(cue_logging, Ordering::Relaxed);
                render_logging_ok.store(render_logging, Ordering::Relaxed);
                let started = Instant::now();
                let result = reader::observe(&LocalMemory { started }, base, &hash);
                let sample = Sample { started: started.duration_since(epoch), finished: epoch.elapsed(), result };
                let (reason, player, ids) = match &sample.result {
                    Ok(observation) => ("", observation.player, observation.effects.iter().map(i32::to_string).collect::<Vec<_>>().join(";")),
                    Err(error) => (error.label(), 0, String::new()),
                };
                let row = format!("{},{},{:?},{},{:x},{}\n", sample.started.as_micros(), sample.finished.as_micros(), sample.state(), reason, player, ids);
                if logging.load(Ordering::Relaxed) {
                    if written + row.len() as u64 > LOG_LIMIT {
                        let _ = log.flush();
                        logging.store(false, Ordering::Relaxed);
                    } else if log.write_all(row.as_bytes()).is_err() {
                        logging.store(false, Ordering::Relaxed);
                    } else {
                        written += row.len() as u64;
                    }
                }
                if last_flush.elapsed() >= Duration::from_secs(1) {
                    if log.flush().is_err() { logging.store(false, Ordering::Relaxed); }
                    if cue_log.flush().is_err() { cue_logging=false; }
                    if render_log.flush().is_err() { render_logging=false; }
                    last_flush = Instant::now();
                }
                match shared.lock() {
                    Ok(mut history) => { history.push(sample); }
                    Err(_) => break,
                }
                thread::sleep(POLL.saturating_sub(cycle_started.elapsed()));
            }
            let _ = log.flush();
            let _ = cue_log.flush();
            let _ = render_log.flush();
        })?;
        Ok(Self {
            history,
            cue: cue_state,
            epoch,
            log_ok,
            cue_log_ok,
            render_log_ok,
            render_dropped,
            render_tx,
            stop,
            worker: Some(worker),
        })
    }

    pub fn record_render(&self, sample: RenderSample) {
        if self.render_tx.try_send(sample).is_err() {
            self.render_dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
}

impl Drop for Diagnostics {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
