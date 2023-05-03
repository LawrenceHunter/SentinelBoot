//! BSP driver support.

use super::memory::map::mmio;
use core::sync::atomic::{AtomicBool, Ordering};
use driver::{driver_manager, DeviceDriverDescriptor, NS16550AUart};

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

/// TODO
static NS16550A_UART: NS16550AUart = unsafe { NS16550AUart::new(mmio::NS16550A_UART_START) };

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

/// TODO
fn post_init_uart() -> Result<(), &'static str> {
    console::register_console(&NS16550A_UART);
    Ok(())
}

/// TODO
fn driver_uart() -> Result<(), &'static str> {
    let uart_descriptor = DeviceDriverDescriptor::new(&NS16550A_UART, Some(post_init_uart));
    driver_manager().register_driver(uart_descriptor);
    Ok(())
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// TODO
/// # Safety
/// TODO
pub unsafe fn init() -> Result<(), &'static str> {
    static INIT_DONE: AtomicBool = AtomicBool::new(false);
    if INIT_DONE.load(Ordering::Relaxed) {
        return Err("Device already initialised");
    }

    driver_uart()?;

    INIT_DONE.store(true, Ordering::Relaxed);
    Ok(())
}
