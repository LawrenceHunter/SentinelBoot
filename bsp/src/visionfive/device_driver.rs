//! BSP driver support.

use crate::memory::map::mmio;
use core::sync::atomic::{AtomicBool, Ordering};
use driver::{driver_manager, DeviceDriverDescriptor, DW8250Uart};

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

/// Unsafe instantiation of DW8250Uart UART
static DW8250_UART: DW8250Uart =
    unsafe { DW8250Uart::new(mmio::DW8250_UART_START) };

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

/// After initialisation register the output device with console
fn post_init_uart() -> Result<(), &'static str> {
    console::register_console(&DW8250_UART);
    Ok(())
}

/// Registers UART driver with driver manager
fn driver_uart() -> Result<(), &'static str> {
    let uart_descriptor =
        DeviceDriverDescriptor::new(&DW8250_UART, Some(post_init_uart));
    driver_manager().register_driver(uart_descriptor);
    Ok(())
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Instantiate all drivers required by VisionFive BSP
/// # Safety
/// Caller must ensure the board is a VisionFive 2 as this is before the
pub unsafe fn init() -> Result<(), &'static str> {
    static INIT_DONE: AtomicBool = AtomicBool::new(false);
    if INIT_DONE.load(Ordering::Relaxed) {
        return Err("Device already initialised");
    }

    driver_uart()?;

    INIT_DONE.store(true, Ordering::Relaxed);
    Ok(())
}
