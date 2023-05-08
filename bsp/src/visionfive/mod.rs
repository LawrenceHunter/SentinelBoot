//! Top-level BSP file for the VisionFive.
pub mod device_driver;
pub mod memory;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Returns the board name based on enabled features
pub fn board_name() -> &'static str {
    #[cfg(feature = "visionfive")]
    {
        "VisionFive"
    }
}
