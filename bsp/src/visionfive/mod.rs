//! Top-level BSP file for the VisionFive.
pub mod device_driver;
pub mod memory;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Returns the board name based on enabled features
pub fn name() -> &'static str {
    #[cfg(feature = "visionfive")]
    {
        "VisionFive"
    }
}

/// Returns the board hart count based on enabled features
pub fn hart_count() -> &'static str {
    #[cfg(feature = "visionfive")]
    {
        "5"
    }
}

/// Returns placeholder string for unknown boot information
pub fn unknown() -> &'static str {
    #[cfg(feature = "visionfive")]
    {
        "???"
    }
}
