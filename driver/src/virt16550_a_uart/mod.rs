//! VIRT16550A UART driver.
use core::fmt;
pub use riscv64::nop;
use synchronisation::interface::Mutex;
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{Aliased, ReadWrite},
};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// VIRT16550A
register_bitfields! {
    u8,
    // Receiver Buffer Register
    RB_RH_R [
        DATA OFFSET(0) NUMBITS(8) [],
    ],
    // Line Status Register
    LSR [
        // Data ready
        DR OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ]
    ]
}

// Generates descriptor block of register types, addresses, and names
register_structs! {
    /// Descriptor block of register types, addresses, and names
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x00 => RB_RH_R: Aliased<u8, RB_RH_R::Register>),
        (0x01 => _reserved0),
        (0x02 => _reserved1),
        (0x03 => _reserved2),
        (0x04 => _reserved3),
        (0x05 => LSR: ReadWrite<u8, LSR::Register>),
        (0x06 => @END),
    }
}

// Abstraction for the associated MMIO registers
type Registers = mmio::MMIODerefWrapper<RegisterBlock>;

#[derive(PartialEq)]
enum BlockingMode {
    Blocking,
    NonBlocking,
}

struct VIRT16550AUartInner {
    registers: Registers,
    chars_written: usize,
    chars_read: usize,
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Representation of the UART.
pub struct VIRT16550AUart {
    inner: synchronisation::NullLock<VIRT16550AUartInner>,
}

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl VIRT16550AUartInner {
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_start_addr),
            chars_written: 0,
            chars_read: 0,
        }
    }

    // Setup baudrate and characteristics
    pub fn init(&mut self) {
    }

    /// Send a char
    fn write_char(&mut self, c: char) {
        self.registers.RB_RH_R.set(c as u8);
        self.chars_written += 1;
    }

    /// Writes all buffered chars
    fn flush(&self) {
    }

    /// Receive char
    fn read_char_converting(
        &mut self,
        blocking_mode: BlockingMode,
    ) -> Option<char> {
        if !self.registers.LSR.is_set(LSR::DR) {
            if blocking_mode == BlockingMode::NonBlocking {
                return None;
            }

            while !self.registers.LSR.is_set(LSR::DR) {
                nop();
            }
        }
        let mut ret = self.registers.RB_RH_R.get() as char;
        // Convert \r -> \n
        if ret == '\r' {
            ret = '\n'
        }
        self.chars_read += 1;
        Some(ret)
    }
}

/// Allows writing formatted strings to UART
impl fmt::Write for VIRT16550AUartInner {
    /// Writes each char in string to UART
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Implements struct for UART
impl VIRT16550AUart {
    /// Driver friendly name
    pub const NAME: &'static str = "VIRT16550A (UART)";

    /// Instantiates new UART driver with given address
    /// # Safety
    /// Caller must ensure mmio start address is valid for the target hardware
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: synchronisation::NullLock::new(VIRT16550AUartInner::new(
                mmio_start_addr,
            )),
        }
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------

/// Implementes DeviceDriver trait for UART
impl super::interface::DeviceDriver for VIRT16550AUart {
    /// Returns a reference to the driver's friendly name
    fn name(&self) -> &'static str {
        Self::NAME
    }

    /// Instantiates a mutex reference to UART driver
    unsafe fn init(&self) -> Result<(), &'static str> {
        self.inner.lock(|inner| inner.init());
        Ok(())
    }
}

impl console::interface::Write for VIRT16550AUart {
    // Pass through 'args' to 'core::fmt::Write' implementation
    // but guarded by a Mutex to serialise access
    fn write_char(&self, c: char) {
        self.inner.lock(|inner| inner.write_char(c));
    }

    /// Writes formatted string to UART guarded by mutex
    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
        self.inner.lock(|inner| fmt::Write::write_fmt(inner, args))
    }

    /// Calls internal flush logic guarded by mutex
    fn flush(&self) {
        self.inner.lock(|inner| inner.flush());
    }
}

impl console::interface::Read for VIRT16550AUart {
    /// Blocking reads char from UART guarded by mutex
    /// Upon a read error ¿ will be printed
    fn read_char(&self) -> char {
        self.inner.lock(|inner| {
            inner.read_char_converting(BlockingMode::Blocking).unwrap_or('¿')
        })
    }

    /// Busy loops until UART is no longer blocked guarded by mutex
    fn clear_rx(&self) {
        while self
            .inner
            .lock(|inner| inner.read_char_converting(BlockingMode::NonBlocking))
            .is_some()
        {}
    }
}

impl console::interface::Statistics for VIRT16550AUart {
    /// Returns the characters written statistic guarded by mutex
    fn chars_written(&self) -> usize {
        self.inner.lock(|inner| inner.chars_written)
    }

    /// Returns the characters read statistic guarded by mutex
    fn chars_read(&self) -> usize {
        self.inner.lock(|inner| inner.chars_read)
    }
}

impl console::interface::All for VIRT16550AUart {}
