//! BSP Memory Management.

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

#[rustfmt::skip]
pub(super) mod map {
    // Physical devices
    #[cfg(feature = "bsp_vsv")]
    pub mod mmio {
        pub const GPIO_START:       usize = 0x100_1000;
        pub const PL011_UART_START: usize = 0x1000_0000;
    }
}
