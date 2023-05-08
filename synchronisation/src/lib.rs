//! Synchronization primitives.
//!
//! # Resources
//!
//!   - <https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html>
//!   - <https://stackoverflow.com/questions/59428096/understanding-the-send-trait>
//!   - <https://doc.rust-lang.org/std/cell/index.html>

#![no_std]
use core::cell::UnsafeCell;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Unifying module for mutex trait
pub mod interface {
    /// Allows mutual exclusion lock on a sized data cell
    pub trait Mutex {
        /// Data to obtain lock on
        type Data;
        /// Can only be called once until result is out of scope ergo generating
        /// a mutex lock
        fn lock<'a, R>(&'a self, f: impl FnOnce(&'a mut Self::Data) -> R) -> R;
    }
}

/// Sized generic data cell
pub struct NullLock<T>
where
    T: ?Sized,
{
    data: UnsafeCell<T>,
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

unsafe impl<T> Send for NullLock<T> where T: ?Sized + Send {}
unsafe impl<T> Sync for NullLock<T> where T: ?Sized + Send {}

impl<T> NullLock<T> {
    /// Instantiate new UnsafeCell lock
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }
}

//--------------------------------------------------------------------------------------------------
// OS Interface Code
//--------------------------------------------------------------------------------------------------

impl<T> interface::Mutex for NullLock<T> {
    type Data = T;

    fn lock<'a, R>(&'a self, f: impl FnOnce(&'a mut Self::Data) -> R) -> R {
        let data = unsafe { &mut *self.data.get() };
        f(data)
    }
}
