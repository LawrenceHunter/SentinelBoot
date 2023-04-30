//! Boot code.

#[cfg(target_arch = "riscv64")]
#[path = "../../riscv64/src/cpu/boot.rs"]
mod arch_boot;
