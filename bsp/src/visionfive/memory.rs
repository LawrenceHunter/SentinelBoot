//! BSP Memory Management Wrapper

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

#[rustfmt::skip]
pub(crate) mod map {
    // Physical devices based on feature target
    pub mod mmio {
        pub const DW8250_UART_START:  usize = 0x1000_0000;
    }
}
