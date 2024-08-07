// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
// Copyright (c) 2023-2024 Lawrence Hunter <lawrence.hunter@outlook.com>
#![no_std]

//! Architectural processor code.
//!
//! # Orientation
//!
//! Since arch modules are imported into generic modules using the path
//! attribute, the path of this file is:
//!
//! crate::cpu::riscv64_cpu

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

use core::arch::asm;

/// Performs a `nop` operation for input cycles
/// ```
/// let x: usize = 10;
/// spin_for_cycles(x);
/// ```

pub fn spin_for_cycles(n: usize) {
    for _ in 0..n {
        nop();
    }
}

/// Performs a `nop` operation forerver
/// ```
/// wait_forever();
/// ```
pub fn wait_forever() -> ! {
    loop {
        nop();
    }
}

/// Performs a `nop` operation
/// ```
/// nop();
/// ```
pub fn nop() {
    unsafe {
        asm!("nop");
    }
}
