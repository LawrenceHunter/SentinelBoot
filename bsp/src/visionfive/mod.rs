// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
// Copyright (c) 2023-2024 Lawrence Hunter <lawrence.hunter@outlook.com>

//! Top-level BSP file for the VisionFive.
pub mod device_driver;
pub mod memory;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Returns the board name based on enabled features
pub fn board_name() -> &'static str {
    "VisionFive"
}

/// Output board information
pub fn print_info() {
    console::println!("\tNAME: {}", crate::board_name());
}
