#![no_std]

//! Architectural processor code.
//!
//! # Orientation
//!
//! Since arch modules are imported into generic modules using the path attribute, the path of this
//! file is:
//!
//! crate::cpu::riscv64_cpu

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

use core::arch::asm;

/// TODO
pub fn spin_for_cycles(n: usize) {
    for _ in 0..n {
        unsafe {
            asm!("nop");
        }
    }
}

/// TODO
pub fn nop() {
    unsafe {
        asm!("nop");
    }
}

/// TODO
pub fn wait_forever() -> ! {
    loop {
        unsafe {
            asm!("nop");
        }
    }
}
