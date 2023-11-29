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
        pub const VIRT16550A_UART_START:  usize = 0x1000_0000;
    }

    /// Kernel entry point address
    pub mod kernel {
        /// Server public key
        pub const PUBK:  usize = 0x800f_0000;
        /// Kernel entry point
        pub const KERNEL:  usize = 0x8020_0100;
        /// Kernel dtb
        pub const DTB:  usize = 0x84a0_0000;
        /// Kernel ramfs
        pub const RAMFS:  usize = 0x8500_0000;
        /// Kernel HART
        pub const HART:  usize = 0;
    }
}
