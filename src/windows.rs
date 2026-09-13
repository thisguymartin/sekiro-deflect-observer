use std::ffi::c_void;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
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

    let diagnostics = diagnostics::Diagnostics::start(fingerprint.clone())?;
    let overlay = Observer {
        visible: AtomicBool::new(true),
        debug: AtomicBool::new(false),
        fingerprint,
        diagnostics,
        cue_fonts: Vec::new(),
        frame: 0,
        vertical_adjust: AtomicI32::new(0),
    };
    Hudhook::builder()
        .with::<ImguiDx11Hooks>(overlay)
        .with_hmodule(HINSTANCE(module))
        .build()
        .apply()
        .map_err(|error| format!("DirectX 11 hook installation failed: {error:?}"))?;
    log_event(
        &mut log,
        "Hooks installed. Overhead activation-estimate preview; contact timing unvalidated. F8: visibility; F9: diagnostics.",
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
    visible: AtomicBool,
    debug: AtomicBool,
    fingerprint: String,
    diagnostics: diagnostics::Diagnostics,
    cue_fonts: Vec<(f32, usize)>,
    frame: u64,
    vertical_adjust: AtomicI32,
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
            self.visible.fetch_xor(true, Ordering::Relaxed);
        }
        if input::is_diagnostics_toggle(message, key.0, flags.0)
            && unsafe { GetForegroundWindow() } == hwnd
        {
            self.debug.fetch_xor(true, Ordering::Relaxed);
        }
        if let Some(delta) = input::placement_adjustment(message, key.0, flags.0) {
            if unsafe { GetForegroundWindow() } == hwnd {
                let _ = self.vertical_adjust.fetch_update(
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                    |value| Some((value + delta).clamp(-160, 160)),
                );
            }
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
        if !self.visible.load(Ordering::Relaxed) {
            self.diagnostics.record_render(submitted);
            return;
        }
        let (target, advancing, cue_status, sample_age) = match self.diagnostics.cue.lock() {
            Ok(state) => {
                let age = state.latest.as_ref().map_or(0.0, |(at, _)| {
                    now.saturating_sub(*at).as_secs_f64() * 1000.0
                });
                let status = if state.latest.is_some() && state.current(now).is_none() {
                    "stale_sample"
                } else if state.read_error.is_some() {
                    state.read_error.unwrap().label()
                } else if state.trace.stage.is_empty() {
                    "waiting_for_reader"
                } else {
                    state.trace.stage
                };
                (
                    state.current(now).cloned(),
                    state.advancing(now),
                    status,
                    age,
                )
            }
            Err(_) => (None, false, "reader_unavailable", 0.0),
        };
        submitted.status = cue_status;
        submitted.sample_age_ms = sample_age;
        if let Some(target) = target.as_ref() {
            submitted.handle = target.handle;
            submitted.model = target.model;
            submitted.animation = target.animation.id;
            submitted.animation_time = target.animation.time;
            submitted.sequence = target.animation.sequence;
            cue_draw::draw(
                ui,
                target,
                advancing,
                self.vertical_adjust.load(Ordering::Relaxed),
                &self.cue_fonts,
                &mut submitted,
            );
        }
        self.diagnostics.record_render(submitted);
        if !self.debug.load(Ordering::Relaxed) {
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
                if let Some(target) = target.as_ref() {
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
                ui.separator();
                ui.text("Recent observations (seconds since reader start)");
                for (at, state) in &transitions {
                    ui.text(format!("{:8.3}  {state:?}", at.as_secs_f64()));
                }
                ui.text(format!("Executable SHA256: {}...", &self.fingerprint[..12]));
                ui.separator();
                ui.text("Cue preview: activation estimate; contact not validated.");
                ui.text("F6/F7: lower/raise bar. F8: visibility. F9: diagnostics.");
            });
    }
}
