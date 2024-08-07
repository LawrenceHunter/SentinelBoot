// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
// Copyright (c) 2023-2024 Lawrence Hunter <lawrence.hunter@outlook.com>

//! DW8250 UART driver.
use core::fmt;
pub use riscv64::nop;
use synchronisation::interface::Mutex;
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{Aliased, ReadWrite, WriteOnly},
};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// DW8250
register_bitfields! {
    u32,
    // Receiver Buffer Register
    RB_RH_R [
        DATA OFFSET(0) NUMBITS(8) [],
    ],
    // Interrupt Enable Register
    IER [
        // Received data available
        ERBFI OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],
        // Transmitter holding register empty
        ETBEI OFFSET(1) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],
        // Receiver line status
        ELSI OFFSET(2) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],
        // MODEM status
        EDSSI OFFSET(3) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],
        DIVISOR_LATCH_MS OFFSET(0) NUMBITS(8) []
    ],
    // FIFO Control Register
    FCR [
        // Enable FIFOs
        FEN OFFSET(0) NUMBITS(1) [
            FifosDisabled = 0,
            FifosEnabled = 1
        ],
        // Reset receiver FIFO
        RRFIFO OFFSET(1) NUMBITS(1) [
            ResetLow = 0,
            ResetHigh = 1
        ],
        // Reset transmitter FIFO
        RTFIFO OFFSET(2) NUMBITS(1) [
            ResetLow = 0,
            ResetHigh = 1
        ],
        // Set DMA mode
        DMA OFFSET(3) NUMBITS(1) [
            SingleTransfer = 0,
            MultipleTransfer = 1
        ],
        // Set Transmit FIFO trigger level
        TFIFOTL OFFSET(4) NUMBITS(2) [
            OneChar = 0b00,
            FourChars = 0b01,
            EightChars = 0b10,
            FourteenChars = 0b11
        ],
        // Set Receive FIFO trigger level
        RFIFOTL OFFSET(6) NUMBITS(2) [
            OneChar = 0b00,
            FourChars = 0b01,
            EightChars = 0b10,
            FourteenChars = 0b11
        ]
    ],
    // Line control Register
    LCR [
        // Word length
        #[allow(clippy::enum_variant_names)]
        WLEN OFFSET(0) NUMBITS(2) [
            FiveBit = 0b00,
            SixBit = 0b01,
            SevenBit = 0b10,
            EightBit = 0b11
        ],
        // Set number of stop bits
        STPB OFFSET(2) NUMBITS(1) [
            OneBit = 0,
            TwoBit = 1,
        ],
        // Set number of parity bits
        PARE OFFSET(3) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1,
        ],
        // Set parity mode
        EPAR OFFSET(4) NUMBITS(1) [
            Odd = 0,
            Even = 1,
        ],
        // Divisor Latch Access Bit
        BREAK OFFSET(6) NUMBITS(1) [
            NULL = 0,
            Force = 1
        ],
        // Divisor Latch Access Bit
        DLAB OFFSET(7) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ]
    ],
    // Line Status Register
    LSR [
        // Data ready
        DR OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],
        // Transmitter Holding Register Empty
        THRE OFFSET(5) NUMBITS(1) [
            Full = 0,
            Empty = 1
        ],
        // THR and TSR empty
        TE OFFSET(6) NUMBITS(1) [
            Full = 0,
            Empty = 1
        ]
    ]
}

// Generates descriptor block of register types, addresses, and names
register_structs! {
    /// Descriptor block of register types, addresses, and names
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x00 => RB_RH_R: Aliased<u32, RB_RH_R::Register>),
        (0x04 => IER: ReadWrite<u32, IER::Register>),
        (0x08 => FCR: WriteOnly<u32, FCR::Register>),
        (0x0c => LCR: ReadWrite<u32, LCR::Register>),
        (0x10 => _reserved0),
        (0x14 => LSR: ReadWrite<u32, LSR::Register>),
        (0x18 => @END),
    }
}

// Abstraction for the associated MMIO registers
type Registers = mmio::MMIODerefWrapper<RegisterBlock>;

#[derive(PartialEq)]
enum BlockingMode {
    Blocking,
    NonBlocking,
}

struct DW8250UartInner {
    registers: Registers,
    chars_written: usize,
    chars_read: usize,
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Representation of the UART.
pub struct DW8250Uart {
    inner: synchronisation::NullLock<DW8250UartInner>,
}

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl DW8250UartInner {
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
    }

    /// Send a char
    fn write_char(&mut self, c: char) {
        // Spin while TX FIFO full is set
        while self.registers.LSR.read(LSR::THRE) == 0 {
            nop();
        }

        self.registers.RB_RH_R.set(c as u32);
        self.chars_written += 1;
    }

    /// Writes all buffered chars
    fn flush(&self) {
        // spin until the Transmission Holding Register Empty bit is cleared
        while self.registers.LSR.matches_all(LSR::THRE::Full) {
            nop();
        }
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
        let ret = char::from_u32(self.registers.RB_RH_R.get());

        match ret {
            Some(mut x) => {
                if x == '\r' {
                    x = '\n'
                }
                self.chars_read += 1;
                return Some(x);
            }
            None => return None,
        }
    }
}

/// Allows writing formatted strings to UART
impl fmt::Write for DW8250UartInner {
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
impl DW8250Uart {
    /// Driver friendly name
    pub const NAME: &'static str = "DW8250 (UART)";

    /// Instantiates new UART driver with given address
    /// # Safety
    /// Caller must ensure mmio start address is valid for the target hardware
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: synchronisation::NullLock::new(DW8250UartInner::new(
                mmio_start_addr,
            )),
        }
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------

/// Implements DeviceDriver trait for UART
impl super::interface::DeviceDriver for DW8250Uart {
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

impl console::interface::Write for DW8250Uart {
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

impl console::interface::Read for DW8250Uart {
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

impl console::interface::Statistics for DW8250Uart {
    /// Returns the characters written statistic guarded by mutex
    fn chars_written(&self) -> usize {
        self.inner.lock(|inner| inner.chars_written)
    }

    /// Returns the characters read statistic guarded by mutex
    fn chars_read(&self) -> usize {
        self.inner.lock(|inner| inner.chars_read)
    }
}

impl console::interface::All for DW8250Uart {}
