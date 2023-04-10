//! Architectural boot code.
//!
//! # Orientation
//!
//! Since arch modules are imported into generic modules using the path attribute, the path of this
//! file is:
//!
//! crate::cpu::boot::arch_boot

use core::arch::global_asm;

// Assembly counterpart to this file.
global_asm!(
    include_str!("boot.s"),
    // CONST_CORE_ID_MASK = const 0b11
);

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

#[no_mangle]
pub unsafe fn _start_rust() -> ! {
    crate::kernel_init()
}
