//! Processor code.

#[cfg(target_arch = "riscv64")]
#[path = "_arch/riscv64/cpu.rs"]
mod riscv64_cpu;

mod boot;

//--------------------------------------------------------------------------------------------------
// Architectural Public Reexports
//--------------------------------------------------------------------------------------------------
pub use riscv64_cpu::{nop, wait_forever, spin_for_cycles};
