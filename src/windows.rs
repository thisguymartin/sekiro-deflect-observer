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

use crate::{identity, input};

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
        "UNKNOWN: no validated build profile or game-state reader. No gameplay-memory reads enabled.",
    )?;

    let overlay = Observer {
        visible: AtomicBool::new(true),
        fingerprint,
    };
    Hudhook::builder()
        .with::<ImguiDx11Hooks>(overlay)
        .with_hmodule(HINSTANCE(module))
        .build()
        .apply()
        .map_err(|error| format!("DirectX 11 hook installation failed: {error:?}"))?;
    log_event(
        &mut log,
        "Hooks installed. Verify the panel in the game. F8 toggles visibility.",
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
    fingerprint: String,
}

impl ImguiRenderLoop for Observer {
    fn initialize<'a>(&'a mut self, context: &mut Context, _: &'a mut dyn RenderContext) {
        context.set_ini_filename(None);
        context.set_log_filename(None);
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
        BeforeWndProc::Continue
    }

    fn message_filter(&self, _: &Io) -> MessageFilter {
        MessageFilter::empty()
    }

    fn render(&mut self, ui: &mut Ui) {
        if !self.visible.load(Ordering::Relaxed) {
            return;
        }
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
                ui.text_colored([1.0, 0.74, 0.24, 1.0], "UNKNOWN");
                ui.text("Game-state reader not implemented.");
                ui.text("This build does not measure deflect windows.");
                ui.text(format!("Executable SHA256: {}...", &self.fingerprint[..12]));
                ui.separator();
                ui.text("F8: show / hide. Close the game to stop.");
            });
    }
}
