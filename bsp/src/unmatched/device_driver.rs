//! BSP driver support.

use crate::memory::map::mmio;
use core::sync::atomic::{AtomicBool, Ordering};
use driver::{driver_manager, DeviceDriverDescriptor, UnmatchedUart};

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

/// Instantiation of Unmatched UART
/// Safe as the MMIO is known for this board satisfying the safety warning
static UNMATCHED_UART: UnmatchedUart = unsafe { UnmatchedUart::new(mmio::UNMATCHED_UART_START) };

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

/// After initialisation register the output device with console
fn post_init_uart() -> Result<(), &'static str> {
    console::register_console(&UNMATCHED_UART);
    Ok(())
}

/// Registers UART driver with driver manager
fn driver_uart() -> Result<(), &'static str> {
    let uart_descriptor =
        DeviceDriverDescriptor::new(&UNMATCHED_UART, Some(post_init_uart));
    driver_manager().register_driver(uart_descriptor);
    Ok(())
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Instantiate all drivers required by unmatched BSP
/// # Safety
/// Caller must ensure the board is a unmatched as this is before the
pub unsafe fn init() -> Result<(), &'static str> {
    static INIT_DONE: AtomicBool = AtomicBool::new(false);
    if INIT_DONE.load(Ordering::Relaxed) {
        return Err("Device already initialised");
    }

    match driver_uart() {
        Ok(_) => {}
        Err(_) => return Err("UART Initialisation fail!")
    }

    INIT_DONE.store(true, Ordering::Relaxed);
    Ok(())
}
