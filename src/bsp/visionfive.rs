//! Top-level BSP file for the VisionFive.
pub mod driver;
pub mod memory;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

pub fn board_name() -> &'static str {
    #[cfg(feature = "bsp_vsv")]
    {
        "VisionFive"
    }
}
