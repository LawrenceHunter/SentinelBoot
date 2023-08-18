//! BSP Memory Management Wrapper

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

#[rustfmt::skip]
pub(crate) mod map {
    // Physical devices based on feature target
    pub mod mmio {
        pub const UNMATCHED_UART_START:  usize = 0x1001_0000;
    }
}
