#![deny(unsafe_op_in_unsafe_fn)]

mod attack_timings;
pub mod cue;
pub mod identity;
pub mod input;
pub mod reader;
pub mod samples;

#[cfg(all(windows, not(target_arch = "x86_64")))]
compile_error!("The Sekiro overlay requires a Windows x64 target.");

#[cfg(all(windows, target_arch = "x86_64", not(test)))]
mod windows;
