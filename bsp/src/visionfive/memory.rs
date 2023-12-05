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
        pub const DTB: usize = 0x43A0_0000;
        /// Kernel ramfs
        pub const RAMFS: usize = 0x4400_0000;
        /// Kernel HART
        pub const HART: usize = 1;
    }
}
