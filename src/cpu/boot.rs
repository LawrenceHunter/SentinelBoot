//! Boot code.

#[cfg(target_arch = "riscv64")]
#[path = "../_arch/riscv64/cpu/boot.rs"]
mod arch_boot;
