//! Null console.
use super::interface;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Implementation of Null console
pub struct NullConsole {}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

/// Global reference
pub static NULL_CONSOLE: NullConsole = NullConsole {};

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl interface::Write for NullConsole {
    /// nop
    fn write_char(&self, _c: char) {}

    /// nop
    fn write_fmt(&self, _args: core::fmt::Arguments) -> core::fmt::Result {
        core::fmt::Result::Ok(())
    }

    /// nop
    fn flush(&self) {}
}

impl interface::Read for NullConsole {
    /// nop
    fn clear_rx(&self) {}
}

impl interface::Statistics for NullConsole {}
impl interface::All for NullConsole {}
