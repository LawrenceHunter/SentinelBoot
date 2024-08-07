// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
// Copyright (c) 2023-2024 Lawrence Hunter <lawrence.hunter@outlook.com>

//! BSP Memory Management Wrapper

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Board memory map
pub mod map {
    /// Physical devices based on feature target
    pub mod mmio {
        /// UART start address
        pub const DW8250_UART_START: usize = 0x1000_0000;
    }

    /// Kernel entry point address
    pub mod kernel {
        /// Kernel signature
        pub const SIGNATURE: usize = 0x401F_FF00;
        /// Kernel entry point
        pub const KERNEL: usize = 0x4020_0000;
        /// Kernel dtb
        pub const DTB: usize = 0x44A0_0000;
        /// Kernel ramfs
        pub const RAMFS: usize = 0x4500_0000;
        /// Kernel HART
        pub const HART: usize = 1;
    }
}
