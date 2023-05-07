//! Top-level BSP file for the VisionFive.
pub mod device_driver;
pub mod memory;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// TODO
pub fn board_name() -> &'static str {
    #[cfg(feature = "visionfive")]
    {
        "VisionFive"
    }
}
