
//! NS16550A UART driver.
use core::fmt;
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadWrite, WriteOnly},
};
use console;
use synchronisation::interface::Mutex;
use mmio;

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// NS16550A
register_bitfields! {
    u8,

    // Receiver Buffer Register
    RB_RH_R [
        DATA OFFSET(0) NUMBITS(8) [],
    ],

    // Interrupt Enable Register
    IER [
        // Recieved data available
        ERBFI OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],
        // Transmitter holding register empty
        ETBEI OFFSET(1) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],
        // Receiever line status
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

    // // Interrupt Ident Register
    // IIR [
    // ],

    // FIFO Control Register
    FCR [
        // Enable FIFOs
        FEN OFFSET(0) NUMBITS(1) [
            FifosDisabled = 0,
            FifosEnabled = 1
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

        // Divisor Latch Access Bit
        DLAB OFFSET(7) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ]
    ],

    // // Modem Control Register
    // MCR [
    //     // Receive enable
    //     RXE OFFSET(9) NUMBITS(1) [
    //         Disabled = 0,
    //         Enabled = 1
    //     ],

    //     // Transmit enable
    //     TXE OFFSET(8) NUMBITS(1) [
    //         Disabled = 0,
    //         Enabled = 1
    //     ],

    //     // UART enable
    //     UARTEN OFFSET(0) NUMBITS(1) [
    //         Disabled = 0,
    //         Enabled = 1
    //     ]
    // ],

    // Line Status Register
    LSR [
        // Data ready
        DR OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ]
    ],

    // // MODEM Status Register
    // MSR [

    // ],

    // // Scratch Register
    // SCR [

    // ],
}

// TODO
register_structs! {
    #[allow(non_snake_case)]
    /// TODO
    pub RegisterBlock {
        (0x00 => RB_RH_R: ReadWrite<u8, RB_RH_R::Register>),
        (0x01 => IER: ReadWrite<u8, IER::Register>),
        (0x02 => FCR: WriteOnly<u8, FCR::Register>),
        (0x03 => LCR: ReadWrite<u8, LCR::Register>),
        (0x04 => _reserved1),
        (0x05 => LSR: ReadWrite<u8, LSR::Register>),
        (0x06 => _reserved2),
        (0x07 => _reserved3),
        (0x08 => @END),
    }
}

// Abstraction for the associated MMIO registers
type Registers = mmio::MMIODerefWrapper<RegisterBlock>;

#[derive(PartialEq)]
enum BlockingMode {
    Blocking,
    NonBlocking,
}

struct NS16550AUartInner {
    registers: Registers,
    chars_written: usize,
    chars_read: usize,
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Representation of the UART.
pub struct NS16550AUart {
    inner: synchronisation::NullLock<NS16550AUartInner>,
}

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl NS16550AUartInner {
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_start_addr),
            chars_written: 0,
            chars_read: 0,
        }
    }

    // Setup baudrate and characteristics
    pub fn init(&mut self) {
        // Unset Divisor Latch Access Bit
        self.registers.LCR.write(LCR::DLAB::Disabled);

        // Set line control register (LCR) word length to 8 bit
        self.registers.LCR.write(LCR::WLEN::EightBit);

        // Enable FIFO
        self.registers.FCR.write(FCR::FEN::FifosEnabled);

        // Enable receiver buffer interrupts
        self.registers.IER.write(IER::ERBFI::Enabled);


		let divisor: u16 = 592;
		let divisor_least: u8 = (divisor & 0xff).try_into().unwrap();
		let divisor_most:  u8 = (divisor >> 8).try_into().unwrap();

        // Set Divisor Latch Access Bit
        self.registers.LCR.write(LCR::DLAB::Enabled);

        // Write DLL and DLM
        self.registers.RB_RH_R.write(RB_RH_R::DATA.val(divisor_least.into()));
        self.registers.IER.write(IER::DIVISOR_LATCH_MS.val(divisor_most.into()));

        // Unset Divisor Latch Access Bit
        self.registers.LCR.write(LCR::DLAB::Disabled);
    }

    /// Send a char
    fn write_char(&mut self, c: char) {
        self.registers.RB_RH_R.set(c as u8);
        self.chars_written += 1;
    }

    /// TODO
    fn flush(&self) {
    }

    /// Receive char
    fn read_char_converting(&mut self, _blocking_mode: BlockingMode) -> Option<char> {
        let mut ret = self.registers.RB_RH_R.get() as char;
        // Convert \r -> \n
        if ret == '\r' {
            ret = '\n'
        }
        self.chars_read += 1;
        Some(ret)
    }
}

/// TODO
impl fmt::Write for NS16550AUartInner {
    /// TODO
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

/// TODO
impl NS16550AUart {
    /// TODO
    pub const COMPATIBLE: &'static str = "NS16550A UART";

    /// TODO
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: synchronisation::NullLock::new(NS16550AUartInner::new(mmio_start_addr)),
        }
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------

/// TODO
impl super::interface::DeviceDriver for NS16550AUart {
    /// TODO
    fn compatible(&self) -> &'static str {
        Self::COMPATIBLE
    }

    /// TODO
    unsafe fn init(&self) -> Result<(), &'static str> {
        self.inner.lock(|inner| inner.init());
        Ok(())
    }
}

impl console::interface::Write for NS16550AUart {
    // Pass through 'args' to 'core::fmt::Write' implementation
    // but guarded by a Mutex to serialise access
    fn write_char(&self, c: char) {
        self.inner.lock(|inner| inner.write_char(c));
    }

    /// TODO
    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
        self.inner.lock(|inner| fmt::Write::write_fmt(inner, args))
    }

    /// TODO
    fn flush(&self) {
        self.inner.lock(|inner| inner.flush());
    }
}

impl console::interface::Read for NS16550AUart {
    /// TODO
    fn read_char(&self) -> char {
        self.inner.lock(
            |inner| inner.read_char_converting(BlockingMode::Blocking).unwrap()
        )
    }

    /// TODO
    fn clear_rx(&self) {
        while self.inner.lock(
            |inner| inner.read_char_converting(BlockingMode::NonBlocking)
        ).is_some()
        {}
    }
}

impl console::interface::Statistics for NS16550AUart {
    /// TODO
    fn chars_written(&self) -> usize {
        self.inner.lock(|inner| inner.chars_written)
    }

    /// TODO
    fn chars_read(&self) -> usize {
        self.inner.lock(|inner| inner.chars_read)
    }
}

impl console::interface::All for NS16550AUart {}
