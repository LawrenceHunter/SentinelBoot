//! BSP console facilities.

use crate::console;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return a reference to the console.
pub fn console() -> &'static dyn console::interface::All {
    self::NS16550AUart
}
