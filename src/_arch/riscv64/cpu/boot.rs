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
    include_str!("boot.s")
);
global_asm!(
    include_str!("trap.s")
);

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

#[no_mangle]
extern "C"
fn main() {
    unsafe {
        crate::loader_init()
    }
}
