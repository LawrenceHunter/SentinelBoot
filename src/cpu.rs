//! Processor code.

#[cfg(target_arch = "riscv64")]
pub use riscv64;

mod boot;

//--------------------------------------------------------------------------------------------------
// Architectural Public Reexports
//--------------------------------------------------------------------------------------------------
pub use ::riscv64::{nop, wait_forever, spin_for_cycles};
