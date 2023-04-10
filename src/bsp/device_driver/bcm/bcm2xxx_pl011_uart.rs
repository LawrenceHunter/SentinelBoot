//! PL011 UART driver.
//!
//! # Resources
//!
//! - <https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf>
//! - <https://developer.arm.com/documentation/ddi0183/latest>

use crate::{
    bsp::device_driver::common::MMIODerefWrapper, console, cpu, driver, synchronisation,
    synchronisation::NullLock,
};
use core::fmt;
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// PL011
register_bitfields! {
    u32,

    // Flag register
    FR [
        // Transmit FIFO empty
        TXFE OFFSET(7) NUMBITS(1) [],

        // Transmity FIFO full
        TXFF OFFSET(5) NUMBITS(1) [],

        // Receive FIFO empty
        RXFE OFFSET(4) NUMBITS(1) [],

        // UART busy
        BUSY OFFSET(3) NUMBITS(1) []
    ],

    // Integer Baud Rate Divisor
    IBRD [
        BAUD_DIVINT OFFSET(0) NUMBITS(16) []
    ],

    // Fractional Baud Rate Divisor
    FBRD [
        BAUD_DIVFRAC OFFSET(0) NUMBITS(6) []
    ],

    // Line control register
    LCR_H [
        // Word length
        #[allow(clippy::enum_variant_names)]
        WLEN OFFSET(5) NUMBITS(2) [
            FiveBit = 0b00,
            SixBit = 0b01,
            SevenBit = 0b10,
            EightBit = 0b11
        ],

        // Enable FIFOs
        FEN OFFSET(4) NUMBITS(1) [
            FifosDisabled = 0,
            FifosEnabled = 1
        ]
    ],

    // Control register
    CR [
        // Receive enable
        RXE OFFSET(9) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        // Transmit enable
        TXE OFFSET(8) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        // UART enable
        UARTEN OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ]
    ],

    // Interrupt Clear Register
    ICR [
        ALL OFFSET(0) NUMBITS(11) []
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x00 => DR: ReadWrite<u32>),
        (0x04 => _reserved1),
        (0x18 => FR: ReadOnly<u32, FR::Register>),
        (0x1c => _reserved2),
        (0x24 => IBRD: WriteOnly<u32, IBRD::Register>),
        (0x28 => FBRD: WriteOnly<u32, FBRD::Register>),
        (0x2c => LCR_H: WriteOnly<u32, LCR_H::Register>),
        (0x30 => CR: WriteOnly<u32, CR::Register>),
        (0x34 => _reserved3),
        (0x44 => ICR: WriteOnly<u32, ICR::Register>),
        (0x48 => @END),
    }
}

// Abstraction for the associated MMIO registers
type Registers = MMIODerefWrapper<RegisterBlock>;

#[derive(PartialEq)]
enum BlockingMode {
    Blocking,
    NonBlocking,
}

struct PL011UartInner {
    registers: Registers,
    chars_written: usize,
    chars_read: usize,
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Representation of the UART.
pub struct PL011Uart {
    inner: NullLock<PL011UartInner>,
}

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl PL011UartInner {
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

        // Turn UART off temporarily
        self.registers.CR.set(0);

        // Clear all pending interrupts
        self.registers.ICR.write(ICR::ALL::CLEAR);

        // Set baudrate, 8N1 and FIFO enable
        self.registers.IBRD.write(IBRD::BAUD_DIVINT.val(3));
        self.registers.FBRD.write(FBRD::BAUD_DIVFRAC.val(16));
        self.registers.LCR_H.write(
            LCR_H::WLEN::EightBit + LCR_H::FEN::FifosEnabled
        );

        // Turn UART on
        self.registers.CR.write(
            CR::UARTEN::Enabled + CR::TXE::Enabled + CR::RXE::Enabled
        );
    }

    // Send a char
    fn write_char(&mut self, c: char) {
        // Spin while TX FIFO full is set
        while self.registers.FR.matches_all(FR::TXFF::SET) {
            cpu::nop();
        }

        // Write char to buffer
        self.registers.DR.set(c as u32);
        self.chars_written += 1;
    }

    // Block execution until last buffered char has been put onto TX
    fn flush(&self) {
        // spin until the busy bit is cleared
        while self.registers.FR.matches_all(FR::BUSY::SET) {
            cpu::nop();
        }
    }

    // Receive char
    fn read_char_converting(&mut self, blocking_mode: BlockingMode) -> Option<char> {
        // If RX FIFO empty
        if self.registers.FR.matches_all(FR::RXFE::SET) {
            if blocking_mode == BlockingMode::NonBlocking {
                return None;
            }

            while self.registers.FR.matches_all(FR::RXFE::SET) {
                cpu::nop();
            }
        }

        // Read one char
        let mut ret = self.registers.DR.get() as u8 as char;

        // Convert \r -> \n
        if ret == '\r' {
            ret = '\n'
        }

        self.chars_read += 1;
        Some(ret)
    }
}

impl fmt::Write for PL011UartInner {
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

impl PL011Uart {
    pub const COMPATIBLE: &'static str = "BCM PL011 UART";

    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: NullLock::new(PL011UartInner::new(mmio_start_addr)),
        }
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------
use synchronisation::interface::Mutex;

impl driver::interface::DeviceDriver for PL011Uart {
    fn compatible(&self) -> &'static str {
        Self::COMPATIBLE
    }

    unsafe fn init(&self) -> Result<(), &'static str> {
        self.inner.lock(|inner| inner.init());
        Ok(())
    }
}

impl console::interface::Write for PL011Uart {
    // Pass through 'args' to 'core::fmt::Write' implementation
    // but guarded by a Mutex to serialise access
    fn write_char(&self, c: char) {
        self.inner.lock(|inner| inner.write_char(c));
    }

    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
        self.inner.lock(|inner| fmt::Write::write_fmt(inner, args))
    }

    fn flush(&self) {
        self.inner.lock(|inner| inner.flush());
    }
}

impl console::interface::Read for PL011Uart {
    fn read_char(&self) -> char {
        self.inner.lock(
            |inner| inner.read_char_converting(BlockingMode::Blocking).unwrap()
        )
    }

    fn clear_rx(&self) {
        while self.inner.lock(
            |inner| inner.read_char_converting(BlockingMode::NonBlocking)
        ).is_some()
        {}
    }
}

impl console::interface::Statistics for PL011Uart {
    fn chars_written(&self) -> usize {
        self.inner.lock(|inner| inner.chars_written)
    }

    fn chars_read(&self) -> usize {
        self.inner.lock(|inner| inner.chars_read)
    }
}

impl console::interface::All for PL011Uart {}
