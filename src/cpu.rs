//! Processor code.

#[cfg(target_arch = "riscv64")]
#[path = "_arch/riscv64/cpu.rs"]
mod riscv_cpu;

mod boot;

//--------------------------------------------------------------------------------------------------
// Architectural Public Reexports
//--------------------------------------------------------------------------------------------------
pub use riscv_cpu::{nop, wait_forever};

#[cfg(feature = "bsp_vsv")]
pub use riscv_cpu::spin_for_cycles;
