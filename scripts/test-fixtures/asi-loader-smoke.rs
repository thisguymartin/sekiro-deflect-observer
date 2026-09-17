//! Standalone Windows loader check. Never reads or starts Sekiro.
use std::{ffi::c_void, ptr, thread, time::Duration};

#[repr(C)]
struct Guid(u32, u16, u16, [u8; 8]);

#[link(name = "dinput8")]
extern "system" {
    fn DirectInput8Create(
        instance: *mut c_void,
        version: u32,
        iid: *const Guid,
        output: *mut *mut c_void,
        outer: *mut c_void,
    ) -> i32;
}
#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(name: *const u16) -> *mut c_void;
    fn GetModuleFileNameW(module: *mut c_void, path: *mut u16, size: u32) -> u32;
}

fn main() {
    // IID_IDirectInput8W. Create/release the interface to exercise DLL forwarding;
    // no input device is created, acquired, polled or written.
    let iid = Guid(
        0xbf798031,
        0x483a,
        0x4da2,
        [0xaa, 0x99, 0x5d, 0x64, 0xed, 0x36, 0x97, 0x00],
    );
    let mut interface = ptr::null_mut();
    let result = unsafe {
        DirectInput8Create(
            GetModuleHandleW(ptr::null()),
            0x800,
            &iid,
            &mut interface,
            ptr::null_mut(),
        )
    };
    assert_eq!(result, 0, "DirectInput8Create forwarding failed");
    assert!(!interface.is_null());
    unsafe {
        let table = *(interface as *const *const usize);
        let release: unsafe extern "system" fn(*mut c_void) -> u32 =
            std::mem::transmute(*table.add(2));
        release(interface);
    }
    let name: Vec<u16> = "sekiro_deflect_observer.asi\0".encode_utf16().collect();
    let log = std::path::PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap())
        .join("SekiroDeflectObserver")
        .join(format!("observer-{}.log", std::process::id()));
    for _ in 0..100 {
        let module = unsafe { GetModuleHandleW(name.as_ptr()) };
        if !module.is_null() {
            let content = std::fs::read_to_string(&log).unwrap_or_default();
            assert!(
                !content.contains("Hooks installed"),
                "Observer hooked a non-game host"
            );
            if content.contains("host is not sekiro.exe; rendering hooks were not installed") {
                let mut path = [0_u16; 32768];
                let count =
                    unsafe { GetModuleFileNameW(module, path.as_mut_ptr(), path.len() as u32) };
                let loaded =
                    std::path::PathBuf::from(String::from_utf16_lossy(&path[..count as usize]));
                assert_eq!(loaded.parent(), std::env::current_exe().unwrap().parent());
                println!("PASS: DirectInput forwarding, adjacent ASI loading, non-game host rejection. PID {}", std::process::id());
                return;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("Loader did not start the adjacent observer ASI within five seconds");
}
