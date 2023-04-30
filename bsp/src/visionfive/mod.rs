//! Top-level BSP file for the VisionFive.
pub mod memory;
pub mod device_driver;

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
