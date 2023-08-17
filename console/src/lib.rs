//! System console.
#![no_std]
#![feature(format_args_nl)]
#![feature(trait_alias)]

mod null_console;
use synchronisation::{self, NullLock};

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Traits required for console operation
pub mod interface {
    /// Trait required to allow printing
    pub trait Write {
        /// Write single char
        fn write_char(&self, c: char);

        /// Write a rust format string
        fn write_fmt(&self, args: core::fmt::Arguments) -> core::fmt::Result;

        /// Block until last buffered char put on TX
        fn flush(&self);
    }

    /// Trait required to allow input
    pub trait Read {
        /// Read a single char
        fn read_char(&self) -> char {
            ' '
        }

        /// Clear RX buffers
        fn clear_rx(&self);
    }

    /// Useful trait to aid debugging
    pub trait Statistics {
        /// Initialise to 0 before writing anything
        fn chars_written(&self) -> usize {
            0
        }

        /// Initialise to 0 before reading anything
        fn chars_read(&self) -> usize {
            0
        }
    }

    /// Groups traits for easier programmer interface
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

use core::fmt;

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    console().write_fmt(args).unwrap();
}

/// Prints without a newline.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

/// Prints with a newline.
#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\r\n")
    };
    ($($arg:tt)*) => ({
        $crate::print!("\r");
        $crate::_print(format_args_nl!($($arg)*));
    })
}
