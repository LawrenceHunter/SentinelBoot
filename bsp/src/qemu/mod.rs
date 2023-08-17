//! Top-level BSP file for QEMU.
pub mod device_driver;
pub mod memory;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Returns the board name based on enabled features
pub fn board_name() -> &'static str {
    #[cfg(feature = "qemu")]
    {
        "QEMU"
    }
}
