//! Processor code.

#[cfg(target_arch = "riscv64")]
pub use riscv64;

mod boot;

//--------------------------------------------------------------------------------------------------
// Architectural Public Reexports
//--------------------------------------------------------------------------------------------------
pub use riscv64::wait_forever;
