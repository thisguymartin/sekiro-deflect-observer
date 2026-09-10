#![deny(unsafe_op_in_unsafe_fn)]

pub mod identity;
pub mod input;

#[cfg(all(windows, not(target_arch = "x86_64")))]
compile_error!("The Sekiro overlay requires a Windows x64 target.");

#[cfg(all(windows, target_arch = "x86_64", not(test)))]
mod windows;
