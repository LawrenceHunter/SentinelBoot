//! BSP Memory Management.

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

#[rustfmt::skip]
pub(super) mod map {
    pub const GPIO_OFFSET:          usize = 0x0020_0000;
    pub const UART_OFFSET:          usize = 0x0020_1000;

    // Physical devices
    #[cfg(feature = "bsp_vsv")]
    pub mod mmio {
        use super::*;

        pub const START:            usize = 0x3F00_0000;
        pub const GPIO_START:       usize = START + GPIO_OFFSET;
        pub const PL011_UART_START: usize = START + UART_OFFSET;
    }
}
