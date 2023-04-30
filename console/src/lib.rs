//! System console.
#![no_std]
#![feature(format_args_nl)]
#![feature(trait_alias)]

mod null_console;
use synchronisation::{self, NullLock};

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// TODO
pub mod interface {
    /// TODO
    pub trait Write {
        /// Write single char
        fn write_char(&self, c: char);

        /// Write a rust format string
        fn write_fmt(&self, args: core::fmt::Arguments) -> core::fmt::Result;

        /// Block until last buffered char put on TX
        fn flush(&self);
    }

    /// TODO
    pub trait Read {
        /// Read a single char
        fn read_char(&self) -> char {
            ' '
        }

        /// Clear RX buffers
        fn clear_rx(&self);
    }

    /// TODO
    pub trait Statistics {
        /// TODO
        fn chars_written(&self) -> usize {
            0
        }

        /// TODO
        fn chars_read(&self) -> usize {
            0
        }
    }

    /// TODO
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

/// Register a new console
pub fn register_console(new_console: &'static (dyn interface::All + Sync)) {
    CUR_CONSOLE.lock(|con| *con = new_console);
}

/// Return a reference to the currently registered console.
pub fn console() -> &'static dyn interface::All {
    CUR_CONSOLE.lock(|con| *con)
}

//------------------------------------------------------------------------------
// Programmer Interface Code
//------------------------------------------------------------------------------
#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    console().write_fmt(args).unwrap();
}

/// Prints without a newline.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
}

/// Prints with a newline.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => (console::_print(format_args!($($arg)*)));
}
