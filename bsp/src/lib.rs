//! BSP wrapper for re-exporting based on enabled features
#![no_std]
#![feature(format_args_nl)]

#[cfg(feature = "qemu")]
#[path = "qemu/mod.rs"]
mod bsp;

#[cfg(feature = "qemu")]
pub use bsp::*;

#[cfg(feature = "visionfive")]
#[path = "visionfive/mod.rs"]
mod bsp;

#[cfg(feature = "visionfive")]
pub use bsp::*;
