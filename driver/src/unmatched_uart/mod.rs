// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
// Copyright (c) 2023-2024 Lawrence Hunter <lawrence.hunter@outlook.com>

//! Unmatched UART driver.
use core::fmt;
pub use riscv64::nop;
use synchronisation::interface::Mutex;
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite},
};
use tock_registers::interfaces::ReadWriteable;
use crate::unmatched_uart::RBR::RFIFOE;
use crate::unmatched_uart::TBR::TFIFOF;

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// Unmatched
register_bitfields! {
    u32,
    // Transmitter Buffer Register
    TBR [
        DATA OFFSET(0) NUMBITS(8) [],
        // Transmitter FIFO full
        TFIFOF OFFSET(31) NUMBITS(1) [
            Empty = 0,
            Full = 1
        ]
    ],
    // Receiver Buffer Register
    RBR [
        DATA OFFSET(0) NUMBITS(8) [],
        // Receiver FIFO empty
        RFIFOE OFFSET(31) NUMBITS(1) [
            Full = 0,
            Empty = 1
        ]
    ],
    // Transmitter Control Register
    TCR [
        // Transmit Enable
        TE OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],
        // Number of stop bits
        NSTPB OFFSET(1) NUMBITS(1) [
            OneBit = 0,
            TwoBit = 1
        ]
    ],
    // Receiver Control Register
    RCR [
        // Receive Enable
        RE OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ]
    ],
    // Interrupt Enable Register
    IER [
        // Transmit watermark interrupt Enable
        TWIE OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],
        // Receive watermark interrupt Enable
        RWIE OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ]
    ],
    // Interrupt Pending Register
    IPR [
        // Transmit watermark interrupt pending
        TWIP OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],
        // Receive watermark interrupt pending
        RWIP OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ]
    ],
    // Baud rate divisor Register
    BRD [
        // Baud rate divisor defaults tp the value we want
        DIV OFFSET(0) NUMBITS(16) [
        ]
    ]
}

// Abstraction for the associated MMIO registers
type Registers = mmio::MMIODerefWrapper<RegisterBlock>;

// Generates descriptor block of register types, addresses, and names
register_structs! {
    /// Descriptor block of register types, addresses, and names
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x00 => TBR: ReadWrite<u32, TBR::Register>),
        (0x04 => RBR: ReadWrite<u32, RBR::Register>),
        (0x08 => TCR: ReadWrite<u32, TCR::Register>),
        (0x0c => RCR: ReadWrite<u32, RCR::Register>),
        (0x10 => IER: ReadWrite<u32, IER::Register>),
        (0x14 => IPR: ReadOnly<u32, IPR::Register>),
        (0x18 => BRD: ReadWrite<u32, BRD::Register>),
        (0x1c => @END),
    }
}

#[derive(PartialEq)]
enum BlockingMode {
    Blocking,
    NonBlocking,
}

struct UnmatchedUartInner {
    registers: Registers,
    chars_written: usize,
    chars_read: usize,
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Representation of the UART.
pub struct UnmatchedUart {
    inner: synchronisation::NullLock<UnmatchedUartInner>,
}

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl UnmatchedUartInner {
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_start_addr),
            chars_written: 0,
            chars_read: 0,
        }
    }

    // Setup baudrate and characteristics
    pub fn init(&mut self) {
        // Ensure all pending chars are transmitted
        self.flush();

        // Enable transmission with one stop bit
        self.registers.TCR.write(TCR::TE::Enabled + TCR::NSTPB::OneBit);

        // Enable receiving with one stop bit
        self.registers.RCR.write(RCR::RE::Enabled);

        // Disable watermark interrupts
        self.registers.IER.write(IER::RWIE::Disabled + IER::TWIE::Disabled);
    }

    /// Send a char
    fn write_char(&mut self, c: char) {
        while self.registers.TBR.matches_all(TFIFOF::Full) {
            nop();
        }
        self.registers.TBR.modify(TBR::DATA.val(c as u32));
        self.chars_written += 1;
    }

    /// Writes all buffered chars
    fn flush(&self) {
        while self.registers.TBR.matches_all(TFIFOF::Full) {
            nop();
        }
    }

    /// Receive char
    fn read_char_converting(
        &mut self,
        blocking_mode: BlockingMode,
    ) -> Option<char> {
        if self.registers.RBR.matches_all(RFIFOE::Empty) {
            if blocking_mode == BlockingMode::NonBlocking {
                return None;
            }

            while self.registers.RBR.matches_all(RFIFOE::Empty) {
                nop();
            }
        }
        let mut ret = self.registers.RBR.read(RBR::DATA) as u8 as char;
        // Convert \r -> \n
        if ret == '\r' {
            ret = '\n'
        }
        self.chars_read += 1;
        Some(ret)
    }
}

/// Allows writing formatted strings to UART
impl fmt::Write for UnmatchedUartInner {
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
impl UnmatchedUart {
    /// Driver friendly name
    pub const NAME: &'static str = "Unmatched (UART)";

    /// Instantiates new UART driver with given address
    /// # Safety
    /// Caller must ensure mmio start address is valid for the target hardware
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: synchronisation::NullLock::new(UnmatchedUartInner::new(
                mmio_start_addr,
            )),
        }
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------

/// Implements DeviceDriver trait for UART
impl super::interface::DeviceDriver for UnmatchedUart {
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

impl console::interface::Write for UnmatchedUart {
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

impl console::interface::Read for UnmatchedUart {
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

impl console::interface::Statistics for UnmatchedUart {
    /// Returns the characters written statistic guarded by mutex
    fn chars_written(&self) -> usize {
        self.inner.lock(|inner| inner.chars_written)
    }

    /// Returns the characters read statistic guarded by mutex
    fn chars_read(&self) -> usize {
        self.inner.lock(|inner| inner.chars_read)
    }
}

impl console::interface::All for UnmatchedUart {}
