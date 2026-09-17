#![deny(unsafe_op_in_unsafe_fn)]

pub mod attack_events;
mod attack_timings;
pub mod config;
pub mod cue;
pub mod hud_art;
pub mod identity;
pub mod incoming;
mod incoming_attacks;
pub mod input;
pub mod layout;
pub mod lifecycle;
pub mod reader;
pub mod samples;
pub mod timing;

#[cfg(all(windows, target_arch = "x86_64"))]
mod event_hook;

#[cfg(all(windows, not(target_arch = "x86_64")))]
compile_error!("The Sekiro overlay requires a Windows x64 target.");

#[cfg(all(windows, target_arch = "x86_64", not(test)))]
mod windows;
