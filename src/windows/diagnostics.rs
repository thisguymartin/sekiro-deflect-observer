use crate::cue::{self, LiveCue};
use crate::reader::{self, Memory, ReadError};
use crate::samples::{History, Sample};
use crate::{config, timing};
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
    pub npc_param: Option<i32>,
    pub animation: i32,
    pub animation_time: f32,
    pub sequence: i32,
    pub sample_age_ms: f64,
    pub position: Option<[f32; 2]>,
    pub decision: timing::Decision,
    pub layout_mode: &'static str,
    pub display_mode: &'static str,
    pub surface: [f32; 2],
    pub viewport: [f32; 4],
    pub bounds: Option<[f32; 4]>,
    pub captured_at: Option<Duration>,
    pub observed_at: Option<Duration>,
    pub metadata: cue::CaptureMetadata,
    pub invalidated_at: Duration,
    pub generation: u64,
    pub profile_evidence: String,
    pub profile_scope: String,
    pub profile_trials: String,
    pub practice: crate::practice::Snapshot,
}
impl RenderSample {
    fn row(&self) -> String {
        let (x, y) = self.position.map_or((String::new(), String::new()), |p| {
            (format!("{:.2}", p[0]), format!("{:.2}", p[1]))
        });
        let d = &self.decision;
        let number = |v: Option<f32>| v.map_or(String::new(), |v| format!("{v:.6}"));
        let mut fields = vec![
            self.at.as_micros().to_string(),
            self.frame.to_string(),
            self.status.into(),
            format!("{:x}", self.handle),
            self.model.to_string(),
            self.animation.to_string(),
            format!("{:.6}", self.animation_time),
            self.sequence.to_string(),
            format!("{:.3}", self.sample_age_ms),
            x,
            y,
            d.state.label().into(),
            d.response.label().into(),
            d.occurrence.to_string(),
            d.phase.map_or(String::new(), |v| v.to_string()),
            number(d.contact.map(|p| p.start)),
            number(d.contact.map(|p| p.end)),
            number(d.press.map(|p| p.start)),
            number(d.press.map(|p| p.end)),
            number(d.preferred),
            d.calibrated.to_string(),
            number(d.rate),
            d.source_age_ms.to_string(),
            d.pulse_emitted.to_string(),
            d.pulse.to_string(),
            d.reason.into(),
            format!("{:.6}", d.animation_time),
            self.layout_mode.into(),
            self.display_mode.into(),
            self.surface[0].to_string(),
            self.surface[1].to_string(),
            self.viewport
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(";"),
            self.captured_at
                .map_or(String::new(), |t| t.as_micros().to_string()),
            self.observed_at
                .map_or(String::new(), |t| t.as_micros().to_string()),
        ];
        fields.extend([
            self.metadata.observation_id.to_string(),
            self.metadata.capture_id.to_string(),
            self.metadata.source.into(),
            d.capture_id.to_string(),
            d.capture_source.into(),
            self.metadata.read_finished.as_micros().to_string(),
            self.metadata.published_at.as_micros().to_string(),
            self.metadata.owner_generation.to_string(),
            self.generation.to_string(),
            self.invalidated_at.as_micros().to_string(),
            if self.position.is_none() && !self.invalidated_at.is_zero() {
                self.at
                    .saturating_sub(self.invalidated_at)
                    .as_micros()
                    .to_string()
            } else {
                String::new()
            },
            self.profile_evidence.clone(),
            self.profile_scope.clone(),
            self.profile_trials.clone(),
        ]);
        fields.extend([
            "unobserved".into(),
            "unobserved".into(),
            "unobserved".into(),
            "unobserved".into(),
        ]);
        fields.extend([
            self.bounds.map_or(String::new(), |b| {
                b.iter().map(f32::to_string).collect::<Vec<_>>().join(";")
            }),
            number(d.reach.map(|r| r.distance).filter(|v| v.is_finite())),
            number(d.reach.map(|r| r.distance_limit).filter(|v| v.is_finite())),
            number(d.reach.map(|r| r.vertical_delta).filter(|v| v.is_finite())),
            number(d.reach.map(|r| r.facing_length).filter(|v| v.is_finite())),
            number(d.reach.and_then(|r| r.facing_cosine)),
            d.reach.map_or("", |r| r.reason).into(),
            self.npc_param.map_or(String::new(), |v| v.to_string()),
            self.npc_param
                .and_then(|npc| crate::attack::npc_variation(self.model, npc))
                .map_or(String::new(), |v| v.to_string()),
            number(d.activation.map(|p| p.start)),
            number(d.activation.map(|p| p.end)),
            d.progress.to_string(),
            self.practice.status.label().into(),
            self.practice.percent.to_string(),
        ]);
        fields
            .iter()
            .map(|s| {
                if s.contains([',', '"', '\n', '\r']) {
                    format!("\"{}\"", s.replace('"', "\"\""))
                } else {
                    s.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(",")
            + "\n"
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

pub(super) struct LocalMemory {
    started: Instant,
}
impl LocalMemory {
    pub(super) fn check_budget(&self) -> Result<(), ReadError> {
        if self.started.elapsed() >= READ_BUDGET {
            Err(ReadError::BudgetExceeded)
        } else {
            Ok(())
        }
    }
    pub(super) fn new() -> Self {
        Self {
            started: Instant::now(),
        }
    }
}
impl Memory for LocalMemory {
    fn read(&self, address: usize, bytes: &mut [u8]) -> Result<(), ReadError> {
        self.check_budget()?;
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
        self.check_budget()?;
        Ok(())
    }
}

/// Immutable configuration and mutable timing state share one short critical
/// section with draw submission. No file I/O runs while this mutex is held.
pub(super) struct CueState {
    pub live: LiveCue,
    pub engine: timing::Engine,
    pub config: config::Config,
    pub settings: timing::Settings,
    pub profiles: Vec<timing::Calibration>,
    pub config_diagnostic: String,
    pub generation: u64,
    pub observation_metadata: cue::CaptureMetadata,
    pub practice: crate::practice::Snapshot,
}
impl CueState {
    fn new(config: config::Config) -> Self {
        let mut state = Self {
            live: LiveCue::default(),
            engine: timing::Engine::default(),
            config: config.clone(),
            settings: timing::Settings::default(),
            profiles: Vec::new(),
            config_diagnostic: String::new(),
            generation: 0,
            observation_metadata: Default::default(),
            practice: Default::default(),
        };
        state.configure(config);
        state
    }
    fn configure(&mut self, config: config::Config) {
        self.settings = timing::Settings {
            preparation_ms: config.preparation_ms,
            latency_ms: config.display_latency_ms + config.input_latency_ms,
            leads_ms: [
                config.parry_lead_ms,
                config.dodge_lead_ms,
                config.jump_lead_ms,
            ],
            enabled: [config.parry, config.dodge, config.jump],
            pulse_duration_ms: config.pulse_duration_ms,
            pulse_intensity: config.pulse_intensity,
            reduced_flash: config.reduced_flash,
        };
        // No read-only runtime form discriminator is established. Explicit
        // all-variants scope is required, never borrow a named-form profile.
        self.profiles = config
            .profiles
            .iter()
            .filter(|p| p.form == "model-animation-phase")
            .map(|p| timing::Calibration {
                model: p.model,
                animation: p.animation,
                phase: p.phase,
                activation_s: p.activation_s,
                end_s: p.end_s,
                response: p.response,
                contact_min_ms: p.contact_min_ms,
                contact_max_ms: p.contact_max_ms,
                early_ms: p.early_ms,
                late_ms: p.late_ms,
                preferred_ms: p.preferred_ms,
            })
            .collect();
        self.config = config;
        self.engine.clear();
    }
}
#[derive(Clone, Copy)]
pub(super) enum Command {
    Placement(f32),
    Reset,
    Visibility(bool),
    Diagnostics(bool),
}

pub(super) struct Diagnostics {
    pub history: Arc<Mutex<History>>,
    pub cue: Arc<Mutex<CueState>>,
    pub visible: Arc<AtomicBool>,
    pub debug: Arc<AtomicBool>,
    pub practice_enabled: Arc<AtomicBool>,
    pub gate: Arc<crate::lifecycle::Gate>,
    pub command_dropped: std::sync::atomic::AtomicU64,
    command_tx: SyncSender<Command>,
    pub epoch: Instant,
    pub log_ok: Arc<AtomicBool>,
    pub cue_log_ok: Arc<AtomicBool>,
    pub render_log_ok: Arc<AtomicBool>,
    pub alert_log_ok: Arc<AtomicBool>,
    pub render_dropped: Arc<std::sync::atomic::AtomicU64>,
    render_tx: SyncSender<RenderSample>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl Diagnostics {
    pub fn start(hash: String, dll_hash: String) -> std::io::Result<Self> {
        let history = Arc::new(Mutex::new(History::default()));
        let config_path = super::log_path()?.with_file_name("cue.toml");
        let missing_config = !config_path.exists();
        let mut store = config::Store::open(config_path.clone());
        if missing_config && store.diagnostic().is_none() {
            super::report_failure(&format!(
                "Configuration: created safe defaults at {}",
                config_path.display()
            ));
        }
        let visible = Arc::new(AtomicBool::new(store.current().visible));
        let debug = Arc::new(AtomicBool::new(store.current().diagnostics));
        let practice_enabled = Arc::new(AtomicBool::new(false));
        let worker_practice = Arc::clone(&practice_enabled);
        let gate = Arc::new(crate::lifecycle::Gate::default());
        let worker_gate = Arc::clone(&gate);
        let worker_visible = Arc::clone(&visible);
        let worker_debug = Arc::clone(&debug);
        let cue_state = Arc::new(Mutex::new(CueState::new(store.current().clone())));
        let (command_tx, command_rx) = mpsc::sync_channel::<Command>(64);
        let stop = Arc::new(AtomicBool::new(false));
        let log_ok = Arc::new(AtomicBool::new(true));
        let cue_log_ok = Arc::new(AtomicBool::new(true));
        let render_log_ok = Arc::new(AtomicBool::new(true));
        let alert_log_ok = Arc::new(AtomicBool::new(true));
        let render_dropped = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let (render_tx, render_rx) = mpsc::sync_channel::<RenderSample>(512);
        let epoch = Instant::now();
        let path = super::log_path()?.with_extension("samples.csv");
        let file = File::create(path)?;
        let shared = Arc::clone(&history);
        let shared_cue = Arc::clone(&cue_state);
        let cue_file = File::create(super::log_path()?.with_extension("cue.csv"))?;
        let render_file = File::create(super::log_path()?.with_extension("render.csv"))?;
        let alert_file = File::create(super::log_path()?.with_extension("alerts.csv"))?;
        let event_file = File::create(super::log_path()?.with_extension("events.csv"))?;
        let practice_file = File::create(super::log_path()?.with_extension("practice.csv"))?;
        let cue_logging_ok = Arc::clone(&cue_log_ok);
        let render_logging_ok = Arc::clone(&render_log_ok);
        let alert_logging_ok = Arc::clone(&alert_log_ok);
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
            let mut alert_log = BufWriter::new(alert_file);
            let mut alert_sampler = crate::alert_log::Sampler::default();
            let mut event_log = BufWriter::new(event_file);
            let mut event_tracker = crate::attack_events::Tracker::default();
            let event_header = format!("# dll_sha256={dll_hash}; executable_sha256={hash}; version={}; epoch_unix_us={epoch_unix_us}; evidence=animation_time_crossing_not_contact\nat_us,capture_source,handle,model,animation_id,event,phase_time,observed_animation_time,player_instance,animation_module,owner_generation,trigger_observation_id,trigger_capture_id,trigger_capture_us,reader_stage\n",env!("CARGO_PKG_VERSION"));
            let mut event_written = event_header.len() as u64;
            let mut event_logging = event_log.write_all(event_header.as_bytes()).is_ok();
            let incoming_data_hash = crate::identity::sha256(&mut &include_bytes!("../incoming_attacks.rs")[..]).unwrap();
            let render_header = format!("# dll_sha256={dll_hash}; executable_sha256={hash}; data_sha256={}; version={}; epoch_unix_us={epoch_unix_us}; incoming_data_sha256={incoming_data_hash}; evidence=draw_submission_not_present_or_contact\nat_us,frame,status,handle,model,animation_id,animation_time,sequence,sample_age_ms,anchor_x,anchor_y,state,response,occurrence,phase,contact_start,contact_end,press_start,press_end,preferred,calibrated,rate,capture_age_ms,pulse_emitted,pulse,reason,projected_animation_time,anchor_mode,display_mode,surface_width,surface_height,viewport,captured_at_us,observed_at_us,observation_id,read_capture_id,read_source,decision_capture_id,decision_source,read_finished_us,published_us,owner_generation,render_generation,invalidated_us,hidden_submission_delay_us,profile_evidence,profile_scope,profile_trials_successes_failures,visible_presentation,actual_input,contact_observation,deflect_result,cue_bounds,coarse_distance,coarse_distance_limit,vertical_delta,facing_length,facing_cosine,reach_reason\n",crate::identity::sha256(&mut &include_bytes!("../attack_timings.rs")[..]).unwrap(),env!("CARGO_PKG_VERSION"));
            let render_header = render_header.trim_end().to_string() + ",npc_param_id,behavior_variation,activation_start,activation_end,phase_progress,practice_status,practice_percent\n";
            let alert_header = "# sampling=received_decision_transitions_and_1s_heartbeat; not_every_frame\n".to_string() + &render_header;
            let mut alert_written = alert_header.len() as u64;
            let mut alert_logging = alert_log.write_all(alert_header.as_bytes()).is_ok();
            let mut render_written = render_header.len() as u64;
            let mut render_logging = render_log.write_all(render_header.as_bytes()).is_ok();
            let cue_header = format!("# dll_sha256={dll_hash}; version={}; executable_sha256={hash}; epoch_unix_us={epoch_unix_us}; timing=activation_estimate; contact=unvalidated\nstart_us,end_us,handle,model,animation_id,animation_time,sequence,activation_only_estimate,reason,stage,lock_enabled,lock_points,point_selected,animation_error,observation_id,capture_id,capture_source,captured_at_us,published_us,owner_generation\n", env!("CARGO_PKG_VERSION"));
            let mut cue_written = cue_header.len() as u64;
            let mut cue_logging = cue_log.write_all(cue_header.as_bytes()).is_ok();
            let header = format!("# executable_sha256={hash}; candidate=105010; semantics=unvalidated\nstart_us,end_us,state,reason,player,effect_ids\n");
            let mut written = header.len() as u64;
            logging.store(log.write_all(header.as_bytes()).is_ok(), Ordering::Relaxed);
            let _ = cue_log.flush();
            let _ = render_log.flush();
            let _ = alert_log.flush();
            let _ = log.flush();
            let base = unsafe { GetModuleHandleW(std::ptr::null()) } as usize;
            let mut practice = super::practice::Session::new(base, &hash, &dll_hash, epoch_unix_us, practice_file);
            let mut last_flush = Instant::now();
            let mut last_reload=Instant::now()-Duration::from_secs(1);
            let mut last_config_diagnostic=String::new();
            let mut previous_owner=None;
            let mut observation_id=0_u64;
            while !stopping.load(Ordering::Relaxed) {
                let cycle_started = Instant::now();
                let record=store.current().diagnostic_logging;
                let started = Instant::now();
                let mut trace = cue::ReadTrace::default();
                let mut cue_result = cue::observe_traced(&LocalMemory { started }, base, &hash, &mut trace);
                if matches!(cue_result, Err(ReadError::ChangedDuringRead)) {
                    cue_result = cue::observe_traced(&LocalMemory { started }, base, &hash, &mut trace);
                }
                if !cue_result.as_ref().is_ok_and(|t|t.is_some()) && previous_owner.is_some() {
                    worker_gate.invalidate(epoch.elapsed());
                    previous_owner=None;
                }
                let event_source = crate::event_hook::apply(cue_result.as_mut().ok().and_then(Option::as_mut), epoch);
                if crate::event_hook::enabled() && cue_result.as_ref().is_ok_and(|r|r.is_some()) {
                    trace.stage = event_source;
                    trace.animation_error = cue_result.as_ref().ok().and_then(Option::as_ref).and_then(|t|t.animation_error);
                }
                let cue_finished = epoch.elapsed();
                let at = started.duration_since(epoch);
                observation_id=observation_id.wrapping_add(1);
                if let Some(target)=cue_result.as_mut().ok().and_then(Option::as_mut) {
                    target.metadata.observation_id=observation_id;
                    if target.metadata.source=="poll" {target.metadata.capture_id=observation_id;}
                    target.metadata.read_start=at;
                    target.metadata.read_finished=cue_finished;
                }
                let owner=cue_result.as_ref().ok().and_then(Option::as_ref).filter(|t|t.animation_error.is_none()).map(|t|(t.player_instance,t.animation_module,t.handle,t.model,t.animation.id,t.npc_param));
                if owner!=previous_owner { worker_gate.invalidate(cue_finished); previous_owner=owner; }
                let mut published_metadata=cue::CaptureMetadata {observation_id,read_start:at,read_finished:cue_finished,source:"invalid_or_missing",..Default::default()};
                let mut practice_target = None;
                // Publish before ALL configuration, event/log formatting and file work.
                if let Ok(mut state) = shared_cue.lock() {
                    if let Some(target)=cue_result.as_mut().ok().and_then(Option::as_mut) {
                        target.metadata.published_at=epoch.elapsed();
                        target.metadata.owner_generation=worker_gate.generation();
                    }
                    let target=cue_result.as_ref().ok().and_then(Option::as_ref);
                    let engine_at=if target.is_some_and(|t|t.captured_at.is_some()){cue_finished}else{at};
                    let generation=worker_gate.generation();
                    if state.generation!=generation { state.engine.clear(); state.generation=generation; }
                    state.engine.observe(engine_at,target);
                    published_metadata=target.map_or(cue::CaptureMetadata {published_at:epoch.elapsed(),owner_generation:generation,..published_metadata},|t|t.metadata);
                    state.observation_metadata=published_metadata;
                    state.live.trace=trace.clone(); state.live.push(at,cue_result.clone());
                    let now = epoch.elapsed();
                    if state.live.advancing(now) {
                        practice_target = target.filter(|t|crate::practice::eligible(t, now)).cloned();
                    }
                } else { break; }
                // Release the render mutex before OS writes and transition logging.
                let context = super::game_has_focus() && worker_visible.load(Ordering::Relaxed)
                    && store.current().visible;
                if !worker_visible.load(Ordering::Relaxed) || !store.current().visible {
                    worker_practice.store(false, Ordering::Relaxed);
                }
                let practice_status = practice.update(epoch.elapsed(), worker_practice.load(Ordering::Relaxed),
                    practice_target.as_ref().filter(|_|context), store.current().practice_speed);
                if let Ok(mut state) = shared_cue.lock() { state.practice = practice_status; }
                let event_target = cue_result.as_ref().ok().and_then(Option::as_ref);
                let event_sample = event_target.filter(|t|t.animation_error.is_none()).map(|t|crate::attack_events::Observation {player_instance:t.player_instance,animation_module:t.animation_module,owner_generation:t.metadata.owner_generation,handle:t.handle,model:t.model,animation:t.animation});
                for event in event_tracker.update(event_sample) {
                    if event_logging && record {
                        let observed_time=event_target.filter(|t|t.handle==event.handle && t.player_instance==event.player_instance && t.animation_module==event.animation_module && t.animation.id==event.animation).map_or(String::new(),|t|format!("{:.6}",t.animation.time));
                        let source=event_target.map_or("no_capture",|t|t.metadata.source);
                        let row = format!("{},{},{:x},{},{},{},{:.6},{},{:x},{:x},{},{},{},{},{}\n",at.as_micros(),source,event.handle,event.model,event.animation,event.kind,event.phase_time,observed_time,event.player_instance,event.animation_module,event.owner_generation,observation_id,event_target.map_or(0,|t|t.metadata.capture_id),event_target.and_then(|t|t.captured_at.or_else(||(t.metadata.source=="poll").then_some(at))).map_or(String::new(),|at|at.as_micros().to_string()),event_source);
                        if event_written + row.len() as u64 > LOG_LIMIT || event_log.write_all(row.as_bytes()).is_err() { event_logging=false; }
                        else { event_written += row.len() as u64; }
                    }
                }
                let cue_row = match &cue_result {
                    Ok(Some(target)) => format!("{},{},{:x},{},{},{:.6},{},{},", at.as_micros(),cue_finished.as_micros(),target.handle,target.model,target.animation.id,target.animation.time,target.animation.sequence,cue::estimated_press(target)),
                    Ok(None) => format!("{},{},,,,,,,No target",at.as_micros(),cue_finished.as_micros()),
                    Err(error) => format!("{},{},,,,,,,{}",at.as_micros(),cue_finished.as_micros(),error.label()),
                };
                let metadata=published_metadata;
                let capture_time=event_target.and_then(|t|t.captured_at.or_else(||(t.metadata.source=="poll").then_some(at))).map_or(String::new(),|t|t.as_micros().to_string());
                let cue_row = format!("{cue_row},{},{},{},{},{},{},{},{},{},{},{}\n",trace.stage,trace.lock_enabled.map_or(String::new(),|b| b.to_string()),trace.points,trace.selected,trace.animation_error.map_or("",ReadError::label),metadata.observation_id,metadata.capture_id,metadata.source,capture_time,metadata.published_at.as_micros(),metadata.owner_generation);
                if cue_logging && record {
                    if cue_written + cue_row.len() as u64 > LOG_LIMIT {
                        let _ = cue_log.flush();
                        cue_logging = false;
                    } else if cue_log.write_all(cue_row.as_bytes()).is_err() {
                        cue_logging = false;
                    } else { cue_written += cue_row.len() as u64; }
                }
                for submitted in render_rx.try_iter().take(512) {
                    if record {
                        let sparse = alert_logging && alert_sampler.record(submitted.at, crate::alert_log::Key {
                            handle:submitted.handle,model:submitted.model,npc_param:submitted.npc_param,
                            animation:submitted.animation,occurrence:submitted.decision.occurrence,
                            phase:submitted.decision.phase,state:submitted.decision.state,response:submitted.decision.response,
                            status:submitted.status,submitted:submitted.position.is_some(),generation:submitted.generation,
                            practice:submitted.practice.status,
                        });
                        if !render_logging && !sparse {continue;}
                        let row = submitted.row();
                        if render_logging {
                            if render_written + row.len() as u64 > LOG_LIMIT || render_log.write_all(row.as_bytes()).is_err() { render_logging=false; }
                            else { render_written += row.len() as u64; }
                        }
                        if sparse {
                            if alert_written + row.len() as u64 > LOG_LIMIT || alert_log.write_all(row.as_bytes()).is_err() { alert_logging=false; }
                            else { alert_written += row.len() as u64; }
                        }
                    }
                }
                cue_logging_ok.store(cue_logging, Ordering::Relaxed);
                render_logging_ok.store(render_logging, Ordering::Relaxed);
                alert_logging_ok.store(alert_logging, Ordering::Relaxed);
                let started = Instant::now();
                let result = reader::observe(&LocalMemory { started }, base, &hash);
                let sample = Sample { started: started.duration_since(epoch), finished: epoch.elapsed(), result };
                let (reason, player, ids) = match &sample.result {
                    Ok(observation) => ("", observation.player, observation.effects.iter().map(i32::to_string).collect::<Vec<_>>().join(";")),
                    Err(error) => (error.label(), 0, String::new()),
                };
                let row = format!("{},{},{:?},{},{:x},{}\n", sample.started.as_micros(), sample.finished.as_micros(), sample.state(), reason, player, ids);
                if logging.load(Ordering::Relaxed) && record {
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
                    if alert_log.flush().is_err() { alert_logging=false; }
                    if event_log.flush().is_err() { event_logging=false; }
                    last_flush = Instant::now();
                }
                match shared.lock() {
                    Ok(mut history) => { history.push(sample); }
                    Err(_) => break,
                }
                if last_reload.elapsed()>=Duration::from_secs(1) {
                    let _=store.reload();
                    last_reload=Instant::now();
                }
                for command in command_rx.try_iter().take(64) {
                    let _=match command {Command::Placement(delta)=>store.adjust_placement(delta),Command::Reset=>store.reset_placement(),Command::Visibility(value)=>store.update_visibility(Some(value),None),Command::Diagnostics(value)=>store.update_visibility(None,Some(value))};
                }
                let config_diagnostic=store.diagnostic().unwrap_or("").to_string();
                if config_diagnostic!=last_config_diagnostic {
                    if !config_diagnostic.is_empty() {super::report_failure(&format!("Configuration: {config_diagnostic}"));}
                    last_config_diagnostic=config_diagnostic.clone();
                }
                if let Ok(mut state)=shared_cue.lock() {
                    if state.config!=*store.current() {
                        state.configure(store.current().clone());
                        worker_visible.store(state.config.visible,Ordering::Relaxed);
                        worker_debug.store(state.config.diagnostics,Ordering::Relaxed);
                    }
                    state.config_diagnostic=config_diagnostic;
                }
                thread::sleep(POLL.saturating_sub(cycle_started.elapsed()));
            }
            let _ = log.flush();
            let _ = cue_log.flush();
            let _ = render_log.flush();
            let _ = alert_log.flush();
            let _ = event_log.flush();
        })?;
        Ok(Self {
            history,
            cue: cue_state,
            visible,
            debug,
            practice_enabled,
            command_tx,
            gate,
            command_dropped: std::sync::atomic::AtomicU64::new(0),
            epoch,
            log_ok,
            cue_log_ok,
            render_log_ok,
            alert_log_ok,
            render_dropped,
            render_tx,
            stop,
            worker: Some(worker),
        })
    }

    pub fn command(&self, command: Command) {
        if self.command_tx.try_send(command).is_err() {
            self.command_dropped.fetch_add(1, Ordering::Relaxed);
        }
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
