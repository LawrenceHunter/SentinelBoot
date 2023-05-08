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

/// Wrapper for DeviceDriver trait
pub mod interface {
    /// Wrapper for name and initialisation of driver
    pub trait DeviceDriver {
        /// Returns a reference to the driver's friendly name
        fn name(&self) -> &'static str;

        /// Instantiates empty device driver
        /// # Safety
        /// Caller must ensure implementation is valid for target driver on
        /// target hardware
        unsafe fn init(&self) -> Result<(), &'static str> {
            Ok(())
        }
    }
}

/// Function pointer for post initialisation
/// # Safety
/// Caller must ensure the function pointed to is valid for the target driver on
/// the targeted hardware
pub type DeviceDriverPostInitCallback = unsafe fn() -> Result<(), &'static str>;

#[derive(Copy, Clone)]
/// Wrapper for driver and callback function
pub struct DeviceDriverDescriptor {
    device_driver: &'static (dyn interface::DeviceDriver + Sync),
    post_init_callback: Option<DeviceDriverPostInitCallback>,
}

/// Wrapper for inner mutex
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
    /// Instantiates a default manager inner
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

/// Implements descriptor trait for driver
impl DeviceDriverDescriptor {
    /// Instantiates a new descriptor
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

/// Returns a reference to the driver manager
pub fn driver_manager() -> &'static DriverManager {
    &DRIVER_MANAGER
}

impl DriverManager {
    /// Instantiates an empty manager with mutex
    pub const fn new() -> Self {
        Self {
            inner: NullLock::new(DriverManagerInner::new()),
        }
    }

    /// Adds the `DeviceDriverDescriptor` to the DriverManagers internal
    /// descriptors array
    pub fn register_driver(&self, descriptor: DeviceDriverDescriptor) {
        self.inner.lock(|inner| {
            inner.descriptors[inner.next_index] = Some(descriptor);
            inner.next_index += 1;
        })
    }

    /// Implements for each allowing easier iteration through driver descriptors
    fn for_each_descriptor<'a>(
        &'a self,
        f: impl FnMut(&'a DeviceDriverDescriptor),
    ) {
        self.inner.lock(|inner| {
            inner
                .descriptors
                .iter()
                .filter_map(|x| x.as_ref())
                .for_each(f)
        })
    }

    /// Initialises all registered device drivers for the target hardware
    /// # Safety
    /// Caller must ensure `DeviceDriverDescriptor` is valid for the target
    /// hardware
    pub unsafe fn init_drivers(&self) {
        self.for_each_descriptor(|descriptor| {
            // Initialise driver
            if let Err(x) = descriptor.device_driver.init() {
                panic!(
                    "Error initialising driver: {}: {}",
                    descriptor.device_driver.name(),
                    x
                );
            }
            // Call corresponding post init callback
            if let Some(callback) = &descriptor.post_init_callback {
                if let Err(x) = callback() {
                    panic!(
                        "Error during dirver post-init callback: {}: {}",
                        descriptor.device_driver.name(),
                        x
                    );
                }
            }
        });
    }

    /// Enumerate all registered device drivers
    pub fn enumerate(&self) {
        let mut i: usize = 1;
        self.for_each_descriptor(|descriptor| {
            console::println!("   {}. {}", i, descriptor.device_driver.name());
            i += 1;
        });
    }
}
