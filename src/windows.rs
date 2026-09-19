use std::ffi::c_void;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use hudhook::hooks::dx11::ImguiDx11Hooks;
use hudhook::imgui::{self, Condition, Context, Io, Ui, WindowFlags};
use hudhook::windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, WPARAM};
use hudhook::windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
use hudhook::{BeforeWndProc, Hudhook, ImguiRenderLoop, MessageFilter, RenderContext};

use crate::samples::State;
use crate::{identity, input};

mod cue_draw;
mod diagnostics;
mod practice;

const DLL_PROCESS_ATTACH: u32 = 1;
static STARTED: AtomicBool = AtomicBool::new(false);

#[link(name = "kernel32")]
extern "system" {
    fn CreateThread(
        attributes: *const c_void,
        stack_size: usize,
        start: unsafe extern "system" fn(*mut c_void) -> u32,
        parameter: *mut c_void,
        flags: u32,
        thread_id: *mut u32,
    ) -> *mut c_void;
    fn CloseHandle(handle: *mut c_void) -> i32;
    fn OutputDebugStringW(message: *const u16);
    fn GetModuleFileNameW(module: *mut c_void, name: *mut u16, size: u32) -> u32;
}

#[link(name = "user32")]
extern "system" {
    fn GetWindowThreadProcessId(window: HWND, process_id: *mut u32) -> u32;
    fn GetKeyState(key: i32) -> i16;
}
fn game_has_focus() -> bool {
    let mut process_id = 0;
    let thread = unsafe { GetWindowThreadProcessId(GetForegroundWindow(), &mut process_id) };
    thread != 0 && process_id == std::process::id()
}

#[no_mangle]
unsafe extern "system" fn DllMain(module: *mut c_void, reason: u32, _: *mut c_void) -> i32 {
    if reason != DLL_PROCESS_ATTACH || STARTED.swap(true, Ordering::Relaxed) {
        return 1;
    }

    // Windows delays the new thread's entry until DLL initialization returns.
    let thread = unsafe {
        CreateThread(
            std::ptr::null(),
            0,
            initialize,
            module,
            0,
            std::ptr::null_mut(),
        )
    };
    if thread.is_null() {
        return 0;
    }
    // Closing our handle does not terminate the initialization thread.
    unsafe { CloseHandle(thread) };
    1
}

unsafe extern "system" fn initialize(module: *mut c_void) -> u32 {
    std::panic::catch_unwind(|| match start_observer(module) {
        Ok(()) => 0,
        Err(error) => {
            report_failure(&format!("Initialization failed: {error}"));
            1
        }
    })
    .unwrap_or(1)
}

fn start_observer(module: *mut c_void) -> Result<(), Box<dyn std::error::Error>> {
    let mut log = open_log()?;
    log_event(
        &mut log,
        &format!("Observer {} starting", env!("CARGO_PKG_VERSION")),
    )?;
    let mut module_name = vec![0_u16; 32768];
    let length =
        unsafe { GetModuleFileNameW(module, module_name.as_mut_ptr(), module_name.len() as u32) }
            as usize;
    if length == 0 || length >= module_name.len() {
        return Err("could not identify loaded DLL path".into());
    }
    use std::os::windows::ffi::OsStringExt;
    let module_path = PathBuf::from(std::ffi::OsString::from_wide(&module_name[..length]));
    let dll_hash = identity::sha256(&mut File::open(module_path)?)?;
    log_event(&mut log, &format!("Loaded DLL SHA256: {dll_hash}"))?;
    let executable = std::env::current_exe()?;
    if !executable.file_name().is_some_and(identity::is_sekiro_host) {
        return Err("host is not sekiro.exe; rendering hooks were not installed".into());
    }

    let fingerprint = identity::sha256(&mut File::open(&executable)?)?;
    log_event(&mut log, &format!("Executable SHA256: {fingerprint}"))?;
    log_event(
        &mut log,
        "Diagnostic reader: candidate effect 105010; deflect-window semantics NOT validated.",
    )?;

    let diagnostics = diagnostics::Diagnostics::start(fingerprint.clone(), dll_hash)?;
    let overlay = Observer {
        fingerprint: fingerprint.clone(),
        diagnostics,
        cue_fonts: Vec::new(),
        frame: 0,
    };
    Hudhook::builder()
        .with::<ImguiDx11Hooks>(overlay)
        .with_hmodule(HINSTANCE(module))
        .build()
        .apply()
        .map_err(|error| format!("DirectX 11 hook installation failed: {error:?}"))?;
    match crate::event_hook::install(&fingerprint) {
        Ok(()) => log_event(&mut log, "Animation-event batch-boundary hook installed; completed batches drive timing. Contact/result hooks not implemented.")?,
        Err(reason) => log_event(&mut log, &format!("Event hook unavailable: {reason}. Using polling fallback."))?,
    }
    log_event(
        &mut log,
        "Hooks installed. Incoming attack response HUD by default; no reach/contact prediction. Optional legacy timing mode. F6/F7: persist placement; F8: visibility; F9: diagnostics; F10: reset offsets; F11: session-only enemy speed practice (initially off); Shift+F11: select 90%/80%/70%/60% speed.",
    )?;
    Ok(())
}

fn log_path() -> io::Result<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "LOCALAPPDATA is missing"))?;
    let directory = PathBuf::from(local).join("SekiroDeflectObserver");
    fs::create_dir_all(&directory)?;
    Ok(directory.join(format!("observer-{}.log", std::process::id())))
}

fn open_log() -> io::Result<File> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path()?)
}

fn log_event(log: &mut File, message: &str) -> io::Result<()> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    writeln!(log, "{seconds} {message}")?;
    log.flush()
}

fn report_failure(message: &str) {
    if let Ok(mut log) = open_log() {
        let _ = log_event(&mut log, message);
    }
    let wide: Vec<u16> = format!("Sekiro Deflect Observer: {message}")
        .encode_utf16()
        .chain(Some(0))
        .collect();
    // The buffer remains alive and null-terminated for the duration of the call.
    unsafe { OutputDebugStringW(wide.as_ptr()) };
}

struct Observer {
    fingerprint: String,
    diagnostics: diagnostics::Diagnostics,
    cue_fonts: Vec<(f32, usize)>,
    frame: u64,
}

impl ImguiRenderLoop for Observer {
    fn initialize<'a>(&'a mut self, context: &mut Context, _: &'a mut dyn RenderContext) {
        context.set_ini_filename(None);
        context.set_log_filename(None);
        self.cue_fonts = cue_draw::initialize_fonts(context);
        context
            .io_mut()
            .config_flags
            .remove(imgui::ConfigFlags::NAV_ENABLE_KEYBOARD);
    }

    fn before_wnd_proc(
        &self,
        hwnd: HWND,
        message: u32,
        key: WPARAM,
        flags: LPARAM,
    ) -> BeforeWndProc {
        if input::is_visibility_toggle(message, key.0, flags.0)
            && unsafe { GetForegroundWindow() } == hwnd
        {
            let value = !self.diagnostics.visible.fetch_xor(true, Ordering::Relaxed);
            if !value {
                self.diagnostics
                    .practice_enabled
                    .store(false, Ordering::Relaxed);
                self.diagnostics
                    .gate
                    .invalidate(self.diagnostics.epoch.elapsed());
            }
            self.diagnostics
                .command(diagnostics::Command::Visibility(value));
        }
        if input::is_diagnostics_toggle(message, key.0, flags.0)
            && unsafe { GetForegroundWindow() } == hwnd
        {
            let value = !self.diagnostics.debug.fetch_xor(true, Ordering::Relaxed);
            self.diagnostics
                .command(diagnostics::Command::Diagnostics(value));
        }
        if unsafe { GetForegroundWindow() } == hwnd
            && self.diagnostics.visible.load(Ordering::Relaxed)
        {
            let shift = unsafe { GetKeyState(0x10) } < 0;
            match input::practice_action(message, key.0, flags.0, shift) {
                Some(input::PracticeAction::Toggle) => {
                    self.diagnostics
                        .practice_enabled
                        .fetch_xor(true, Ordering::Relaxed);
                }
                Some(input::PracticeAction::CycleSpeed) => {
                    self.diagnostics
                        .command(diagnostics::Command::PracticeSpeed);
                }
                None => {}
            }
        }
        if let Some(delta) = input::placement_adjustment(message, key.0, flags.0) {
            if unsafe { GetForegroundWindow() } == hwnd {
                self.diagnostics
                    .command(diagnostics::Command::Placement(delta as f32));
            }
        }
        if input::is_placement_reset(message, key.0, flags.0)
            && unsafe { GetForegroundWindow() } == hwnd
        {
            self.diagnostics.command(diagnostics::Command::Reset);
        }
        BeforeWndProc::Continue
    }

    fn message_filter(&self, _: &Io) -> MessageFilter {
        MessageFilter::empty()
    }

    fn render(&mut self, ui: &mut Ui) {
        self.frame += 1;
        let now = self.diagnostics.epoch.elapsed();
        let mut submitted = diagnostics::RenderSample {
            at: now,
            frame: self.frame,
            status: "hidden",
            ..Default::default()
        };
        let focused = game_has_focus();
        submitted.generation = self.diagnostics.gate.generation();
        submitted.invalidated_at = self.diagnostics.gate.invalidated_at();
        submitted.surface = ui.io().display_size;
        submitted.display_mode = "unobserved_record_in_trial";
        let mut practice_speed = crate::config::Config::default().practice_speed;
        let (target, advancing, cue_status, sample_age, config_diagnostic, inactive_profiles) =
            match self.diagnostics.cue.lock() {
                Ok(mut state) => {
                    let now = self.diagnostics.epoch.elapsed();
                    submitted.at = now;
                    submitted.generation = self.diagnostics.gate.generation();
                    submitted.invalidated_at = self.diagnostics.gate.invalidated_at();
                    let target = state.live.current_lock(now).cloned();
                    let age = now
                        .saturating_sub(state.observation_metadata.read_start)
                        .as_secs_f64()
                        * 1000.0;
                    submitted.metadata = state.observation_metadata;
                    submitted.practice = state.practice;
                    practice_speed = state.config.practice_speed;
                    let status = if state.live.latest.is_some() && target.is_none() {
                        "stale_sample"
                    } else if let Some(error) = state.live.read_error {
                        error.label()
                    } else if state.live.trace.stage.is_empty() {
                        "waiting_for_reader"
                    } else {
                        state.live.trace.stage
                    };
                    submitted.status = status;
                    submitted.sample_age_ms = age;
                    submitted.observed_at = Some(state.observation_metadata.read_start);
                    let settings = state.settings.clone();
                    let profiles = state.profiles.clone();
                    if !self.diagnostics.gate.accepts(state.generation)
                        || !self.diagnostics.visible.load(Ordering::Relaxed)
                        || !focused
                        || target.is_none()
                    {
                        state.engine.clear();
                        submitted.status = if !focused {
                            "unfocused"
                        } else if !self.diagnostics.visible.load(Ordering::Relaxed) {
                            "hidden_by_user"
                        } else if !self.diagnostics.gate.accepts(state.generation) {
                            "invalidated_before_publish"
                        } else {
                            status
                        };
                    } else if let Some(target) = target.as_ref() {
                        submitted.handle = target.handle;
                        submitted.model = target.model;
                        submitted.npc_param = target.npc_param;
                        submitted.animation = target.animation.id;
                        submitted.animation_time = target.animation.time;
                        submitted.sequence = target.animation.sequence;
                        submitted.captured_at = target.captured_at.or_else(|| {
                            (target.metadata.source == "poll").then_some(target.metadata.read_start)
                        });
                        submitted.metadata = target.metadata;
                        submitted.decision = if state.config.incoming_cues {
                            state
                                .engine
                                .incoming(now, settings.enabled, state.config.mikiri)
                        } else {
                            state.engine.decide_with(now, &settings, &profiles)
                        };
                        if submitted.decision.calibrated {
                            if let Some(profile) = state.config.profiles.iter().find(|p| {
                                p.model == target.model
                                    && p.animation == target.animation.id
                                    && Some(p.phase) == submitted.decision.phase
                                    && p.form == "model-animation-phase"
                            }) {
                                submitted.profile_evidence = profile.evidence.clone();
                                submitted.profile_scope =
                                    format!("{} / {}", profile.encounter, profile.form);
                                submitted.profile_trials = format!(
                                    "{};{};{}",
                                    profile.trials, profile.successes, profile.failures
                                );
                            }
                        }
                        // Last check immediately before submitting any cue geometry.
                        if self.diagnostics.gate.accepts(state.generation) {
                            cue_draw::draw(
                                ui,
                                target,
                                &submitted.decision.clone(),
                                &state.config,
                                &self.cue_fonts,
                                &mut submitted,
                            );
                        } else {
                            submitted.status = "invalidated_before_draw";
                        }

                        if submitted.position.is_none() {
                            state.engine.clear();
                            submitted.decision = crate::timing::Decision::default();
                        }
                    }
                    // The session crest also draws with no lock or usable attack.
                    // Only a fresh, matching owner may display an applied rate.
                    if focused && self.diagnostics.visible.load(Ordering::Relaxed) {
                        let mut practice = state.practice;
                        if practice.status == crate::practice::Status::Active
                            && (!state.live.advancing(now)
                                || !self.diagnostics.gate.accepts(state.generation)
                                || !target.as_ref().is_some_and(|t| practice.active_for(t)))
                        {
                            practice.status = crate::practice::Status::Ready;
                        }
                        cue_draw::draw_practice_indicator(
                            ui,
                            &state.config,
                            &self.cue_fonts,
                            self.diagnostics.practice_enabled.load(Ordering::Relaxed),
                            practice,
                            target
                                .as_ref()
                                .and_then(|t| t.camera)
                                .map_or(16.0 / 9.0, |c| c.aspect),
                        );
                    }
                    (
                        target,
                        state.live.advancing(now),
                        status,
                        age,
                        state.config_diagnostic.clone(),
                        state.config.profiles.len() - state.profiles.len(),
                    )
                }
                Err(_) => (None, false, "reader_unavailable", 0.0, String::new(), 0),
            };
        let render_status = submitted.status;
        let practice_status = submitted.practice.status.label();
        self.diagnostics.record_render(submitted);
        if !self.diagnostics.debug.load(Ordering::Relaxed) {
            return;
        }
        let (state, reason, count, age_ms, read_ms, last_present_ms, transitions) =
            match self.diagnostics.history.lock() {
                Ok(history) => {
                    let state = history.state(now);
                    let (reason, count, age_ms, read_ms) = match &history.latest {
                        Some(sample) => {
                            let reason = if now.saturating_sub(sample.started)
                                >= crate::samples::FRESHNESS
                            {
                                "Samples stale"
                            } else {
                                sample
                                    .result
                                    .as_ref()
                                    .err()
                                    .map_or("", |error| error.label())
                            };
                            (
                                reason,
                                sample
                                    .result
                                    .as_ref()
                                    .map_or(0, |value| value.effects.len()),
                                now.saturating_sub(sample.started).as_secs_f64() * 1000.0,
                                sample.finished.saturating_sub(sample.started).as_secs_f64()
                                    * 1000.0,
                            )
                        }
                        None => ("Waiting for first sample", 0, 0.0, 0.0),
                    };
                    let last_present_ms = history
                        .last_present_duration(now)
                        .map(|duration| duration.as_secs_f64() * 1000.0);
                    (
                        state,
                        reason,
                        count,
                        age_ms,
                        read_ms,
                        last_present_ms,
                        history
                            .transitions
                            .iter()
                            .rev()
                            .take(5)
                            .copied()
                            .collect::<Vec<_>>(),
                    )
                }
                Err(_) => (
                    State::Unknown,
                    "Reader unavailable",
                    0,
                    0.0,
                    0.0,
                    None,
                    Vec::new(),
                ),
            };
        ui.window("Sekiro Deflect Observer")
            .position([24.0, 24.0], Condition::Always)
            .bg_alpha(0.88)
            .flags(
                WindowFlags::NO_INPUTS
                    | WindowFlags::NO_SAVED_SETTINGS
                    | WindowFlags::NO_COLLAPSE
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_RESIZE
                    | WindowFlags::ALWAYS_AUTO_RESIZE
                    | WindowFlags::NO_FOCUS_ON_APPEARING,
            )
            .build(|| {
                ui.text(format!("Observer loaded | {}", env!("CARGO_PKG_VERSION")));
                ui.separator();
                ui.text(format!("Cue reader: {cue_status} | age {sample_age:.1} ms"));
                ui.text(format!("Overlay: {render_status}"));
                if !config_diagnostic.is_empty() {
                    ui.text_wrapped(format!("Configuration: {config_diagnostic}"));
                }
                let dropped = self.diagnostics.command_dropped.load(Ordering::Relaxed);
                if dropped > 0 {
                    ui.text(format!(
                        "Settings commands dropped: {dropped}; repeat the last change"
                    ));
                }
                if inactive_profiles > 0 {
                    ui.text(format!(
                        "Inactive profiles: {inactive_profiles}; runtime form is unobserved"
                    ));
                }
                ui.text(format!(
                    "Event hook: {} | batches: {} | missed captures: {}",
                    crate::event_hook::enabled(),
                    crate::event_hook::CAPTURED.load(Ordering::Relaxed),
                    crate::event_hook::DROPPED.load(Ordering::Relaxed)
                ));
                ui.text("Event crossings are animation timing, not confirmed contact.");
                if let Some(target) = target.as_ref() {
                    let display = ui.io().display_size;
                    ui.text(format!(
                        "Surface: {:.0} x {:.0} | camera aspect: {:.4}",
                        display[0],
                        display[1],
                        target.camera.map_or(0.0, |c| c.aspect)
                    ));
                    ui.text(format!(
                        "Locked model c{:04} | anim {} @ {:.3}s",
                        target.model, target.animation.id, target.animation.time
                    ));
                    ui.text(format!(
                        "Mapped: {} | advancing: {} | special: {}",
                        crate::cue::attack_mapped(target),
                        advancing,
                        crate::cue::special_attack(target)
                    ));
                    let reach = crate::cue::reach(target);
                    ui.text(format!(
                        "Coarse gate: {} | distance {:.2}/{:.2} | height {:.2}",
                        reach.reason, reach.distance, reach.distance_limit, reach.vertical_delta
                    ));
                    ui.text(format!(
                        "Facing length {:.3} | cosine {:?} (not collision)",
                        reach.facing_length, reach.facing_cosine
                    ));
                }
                ui.text(format!(
                    "Cue log: {} | render log: {} | dropped frames: {}",
                    self.diagnostics.cue_log_ok.load(Ordering::Relaxed),
                    self.diagnostics.render_log_ok.load(Ordering::Relaxed),
                    self.diagnostics.render_dropped.load(Ordering::Relaxed)
                ));
                let (color, label) = match state {
                    State::Present => ([0.3, 1.0, 0.5, 1.0], "CANDIDATE PRESENT"),
                    State::Absent => ([0.7, 0.75, 0.85, 1.0], "CANDIDATE ABSENT"),
                    State::Unknown => ([1.0, 0.74, 0.24, 1.0], "UNKNOWN"),
                };
                ui.text_colored(color, label);
                ui.text("Effect 105010 | Research build");
                ui.text("Deflect-window meaning not yet validated.");
                if !reason.is_empty() {
                    ui.text(reason);
                }
                ui.text(format!("Effects: {count} | Sample age: {age_ms:.1} ms"));
                ui.text(format!("Read time: {read_ms:.2} ms"));
                if let Some(duration) = last_present_ms {
                    ui.text(format!("Last candidate duration: {duration:.1} ms"));
                }
                if !self.diagnostics.log_ok.load(Ordering::Relaxed) {
                    ui.text("Sample logging stopped (size limit or I/O error).");
                }
                ui.text(format!(
                    "Sparse alert logging: {}",
                    self.diagnostics.alert_log_ok.load(Ordering::Relaxed)
                ));
                ui.separator();
                ui.text("Recent observations (seconds since reader start)");
                for (at, state) in &transitions {
                    ui.text(format!("{:8.3}  {state:?}", at.as_secs_f64()));
                }
                ui.text(format!("Executable SHA256: {}...", &self.fingerprint[..12]));
                ui.separator();
                ui.text("Incoming labels identify move responses; they do not predict contact.");
                ui.text("F6/F7: placement. F8: cue/off. F9: research. F10: reset. F11: practice. Shift+F11: 90/80/70/60%.");
                ui.text(format!(
                    "Enemy speed practice: {practice_status}; {}% configured; session toggle {}",
                    practice_speed * 100.0,
                    self.diagnostics.practice_enabled.load(Ordering::Relaxed)
                ));
            });
    }
}
