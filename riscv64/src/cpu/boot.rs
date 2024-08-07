// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
// Copyright (c) 2023-2024 Lawrence Hunter <lawrence.hunter@outlook.com>

//! Architectural boot code.
//!
//! # Orientation
//!
//! Since arch modules are imported into generic modules using the path
//! attribute, the path of this file is:
//!
//! crate::cpu::boot::arch_boot

use core::arch::global_asm;
// Assembly counterpart to this file.
#[cfg(feature = "qemu")]
global_asm!(include_str!("boot-u-boot.s"));

#[cfg(feature = "visionfive")]
global_asm!(include_str!("boot-u-boot.s"));

#[cfg(feature = "unmatched")]
global_asm!(include_str!("boot-u-boot.s"));

#[cfg(feature = "qemu_vector")]
global_asm!(include_str!("boot-u-boot.s"));

global_asm!(include_str!("trap.s"));
global_asm!(include_str!("mem.s"));

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

#[no_mangle]
extern "C" fn main() {
    crate::loader_init()
}

#[no_mangle]
extern "C" fn machine_mode_rs() {
    crate::loader_machine()
}
