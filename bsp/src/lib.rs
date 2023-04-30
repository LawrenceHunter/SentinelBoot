//! Device driver.
#![no_std]
#![feature(format_args_nl)]

#[cfg(feature = "visionfive")]
#[path = "visionfive/mod.rs"]
mod bsp;

#[cfg(feature = "visionfive")]
pub use bsp::*;
