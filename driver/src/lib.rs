//! Device driver.
#![no_std]
#![feature(format_args_nl)]

#[cfg(feature = "visionfive")]
mod ns16550_a_uart;

#[cfg(feature = "visionfive")]
pub use ns16550_a_uart::*;

use synchronisation::{interface::Mutex, NullLock};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

const NUM_DRIVERS: usize = 5;

struct DriverManagerInner {
    next_index: usize,
    descriptors: [Option<DeviceDriverDescriptor>; NUM_DRIVERS],
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// TODO
pub mod interface {
    /// TODO
    pub trait DeviceDriver {
        /// TODO
        fn compatible(&self) -> &'static str;

        /// TODO
        /// # Safety
        /// TODO
        unsafe fn init(&self) -> Result<(), &'static str> {
            Ok(())
        }
    }
}

/// TODO
/// # Safety
/// TODO
pub type DeviceDriverPostInitCallback = unsafe fn() -> Result<(), &'static str>;

#[derive(Copy, Clone)]
/// TODO
pub struct DeviceDriverDescriptor {
    device_driver: &'static (dyn interface::DeviceDriver + Sync),
    post_init_callback: Option<DeviceDriverPostInitCallback>,
}

/// TODO
pub struct DriverManager {
    inner: NullLock<DriverManagerInner>,
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static DRIVER_MANAGER: DriverManager = DriverManager::new();

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl DriverManagerInner {
    /// TODO
    pub const fn new() -> Self {
        Self {
            next_index: 0,
            descriptors: [None; NUM_DRIVERS],
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// TODO
impl DeviceDriverDescriptor {
    /// TODO
    pub fn new(
        device_driver: &'static (dyn interface::DeviceDriver + Sync),
        post_init_callback: Option<DeviceDriverPostInitCallback>,
    ) -> Self {
        Self {
            device_driver,
            post_init_callback,
        }
    }
}

/// TODO
pub fn driver_manager() -> &'static DriverManager {
    &DRIVER_MANAGER
}

impl DriverManager {
    /// TODO
    pub const fn new() -> Self {
        Self {
            inner: NullLock::new(DriverManagerInner::new()),
        }
    }

    /// TODO
    pub fn register_driver(&self, descriptor: DeviceDriverDescriptor) {
        self.inner.lock(|inner| {
            inner.descriptors[inner.next_index] = Some(descriptor);
            inner.next_index += 1;
        })
    }

    /// TODO
    fn for_each_descriptor<'a>(&'a self, f: impl FnMut(&'a DeviceDriverDescriptor)) {
        self.inner.lock(|inner| {
            inner
                .descriptors
                .iter()
                .filter_map(|x| x.as_ref())
                .for_each(f)
        })
    }

    /// TODO
    /// # Safety
    /// TODO
    pub unsafe fn init_drivers(&self) {
        self.for_each_descriptor(|descriptor| {
            // Initialise driver
            if let Err(x) = descriptor.device_driver.init() {
                panic!(
                    "Error initialising driver: {}: {}",
                    descriptor.device_driver.compatible(),
                    x
                );
            }
            // Call corresponding post init callback
            if let Some(callback) = &descriptor.post_init_callback {
                if let Err(x) = callback() {
                    panic!(
                        "Error during dirver post-init callback: {}: {}",
                        descriptor.device_driver.compatible(),
                        x
                    );
                }
            }
        });
    }
}
