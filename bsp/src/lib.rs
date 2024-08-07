// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
// Copyright (c) 2023-2024 Lawrence Hunter <lawrence.hunter@outlook.com>

//! BSP wrapper for re-exporting based on enabled features
#![no_std]
#![feature(format_args_nl)]

#[cfg(feature = "qemu")]
#[path = "qemu/mod.rs"]
pub mod bsp;

#[cfg(feature = "qemu")]
pub use bsp::*;

#[cfg(feature = "qemu_vector")]
#[path = "qemu/mod.rs"]
pub mod bsp;

#[cfg(feature = "qemu_vector")]
pub use bsp::*;

#[cfg(feature = "visionfive")]
#[path = "visionfive/mod.rs"]
pub mod bsp;

#[cfg(feature = "visionfive")]
pub use bsp::*;

#[cfg(feature = "unmatched")]
#[path = "unmatched/mod.rs"]
pub mod bsp;

#[cfg(feature = "unmatched")]
pub use bsp::*;
