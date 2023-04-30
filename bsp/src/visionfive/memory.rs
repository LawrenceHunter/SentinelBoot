//! BSP Memory Management.

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

#[rustfmt::skip]
pub(super) mod map {
    // Physical devices
    #[cfg(feature = "visionfive")]
    pub mod mmio {
        pub const NS16550A_UART_START:  usize = 0x1000_0000;
    }
}
