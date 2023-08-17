//! Null console.
use super::interface;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

pub struct NullConsole {}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

pub static NULL_CONSOLE: NullConsole = NullConsole {};

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl interface::Write for NullConsole {
    fn write_char(&self, _c: char) {}

    fn write_fmt(&self, _args: core::fmt::Arguments) -> core::fmt::Result {
        core::fmt::Result::Ok(())
    }

    fn flush(&self) {}
}

impl interface::Read for NullConsole {
    fn clear_rx(&self) {}
}

impl interface::Statistics for NullConsole {}
impl interface::All for NullConsole {}
