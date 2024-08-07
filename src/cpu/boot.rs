// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
// Copyright (c) 2023-2024 Lawrence Hunter <lawrence.hunter@outlook.com

//! Boot code.

#[cfg(target_arch = "riscv64")]
#[path = "../../riscv64/src/cpu/boot.rs"]
mod arch_boot;
