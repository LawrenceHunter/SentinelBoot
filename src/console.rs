//! System console.

mod null_console;
use crate::synchronisation::{self, NullLock};

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

pub mod interface {
    use core::fmt;

    pub trait Write {
        // Write single char
        fn write_char(&self, c: char);

        // Write a rust format string
        fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;

        // Block until last buffered char put on TX
        fn flush(&self);
    }

    pub trait Read {
        // Read a single char
        fn read_char(&self) -> char {
            ' '
        }

        // Clear RX buffers
        fn clear_rx(&self);
    }

    pub trait Statistics {
        fn chars_written(&self) -> usize {
            0
        }

        fn chars_read(&self) -> usize {
            0
        }
    }

    pub trait All: Write + Read + Statistics {}
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static CUR_CONSOLE: NullLock<&'static (dyn interface::All + Sync)> =
    NullLock::new(&null_console::NULL_CONSOLE);


//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
use synchronisation::interface::Mutex;

// Register a new console
pub fn register_console(new_console: &'static (dyn interface::All + Sync)) {
    CUR_CONSOLE.lock(|con| *con = new_console);
}

// Return a reference to the currently registered console.
pub fn console() -> &'static dyn interface::All {
    CUR_CONSOLE.lock(|con| *con)
}
