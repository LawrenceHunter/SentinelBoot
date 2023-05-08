//! BSP Memory Management Wrapper

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

#[rustfmt::skip]
pub(crate) mod map {
    // Physical devices based on feature target
    #[cfg(feature = "visionfive")]
    pub mod mmio {
        pub const NS16550A_UART_START:  usize = 0x1000_0000;
    }
}
