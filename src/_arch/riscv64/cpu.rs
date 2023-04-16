//! Architectural processor code.
//!
//! # Orientation
//!
//! Since arch modules are imported into generic modules using the path attribute, the path of this
//! file is:
//!
//! crate::cpu::riscv64_cpu

use riscv::asm;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
#[cfg(feature = "bsp_vsv")]
pub fn spin_for_cycles(n: usize) {
    for _ in 0..n {
        unsafe {
            asm::nop();
        }
    }
}

pub fn nop() {
    unsafe {
        asm::nop();
    }
}

pub fn wait_forever() -> ! {
    loop {
        unsafe {
            asm::wfi()
        }
    }
}
