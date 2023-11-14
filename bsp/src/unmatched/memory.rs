//! BSP Memory Management Wrapper

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

#[rustfmt::skip]
/// Board memory map
pub mod map {
    /// Physical devices based on feature target
    pub mod mmio {
        /// UART start address
        pub const UNMATCHED_UART_START:  usize = 0x1001_0000;
    }

    /// Kernel entry point address
    pub mod kernel {
        /// Kernel entry point
        pub const KERNEL:  usize = 0x4020_0000;
        /// Kernel dtb
        pub const DTB:  usize = 0x43a0_0000;
        /// Kernel ramfs
        pub const RAMFS:  usize = 0x4400_0000;
        /// Kernel HART
        pub const HART:  usize = 1;
    }
}
