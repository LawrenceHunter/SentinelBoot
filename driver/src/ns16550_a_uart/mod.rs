//! NS16550A UART driver.
use core::fmt;
use synchronisation::interface::Mutex;
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields,
    register_structs,
    registers::{Aliased, ReadWrite},
};
pub use riscv64::nop;

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

// Generates dscriptor block of register types, addresses, and names
register_structs! {
    /// Descriptor block of register types, addresses, and names
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x00 => RB_RH_R: Aliased<u8, RB_RH_R::Register>),
        (0x01 => IER: ReadWrite<u8, IER::Register>),
        (0x02 => FCR: Aliased<u8, FCR::Register>),
        (0x03 => LCR: ReadWrite<u8, LCR::Register>),
        (0x04 => _reserved0),
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
        // Ensure all pending chars are transmitted
        self.flush();

        // Set line control register (LCR) word length to 8 bit
        self.registers.LCR.write(LCR::WLEN::EightBit);

        // Enable FIFO
        self.registers.FCR.set(FCR::FEN::FifosEnabled.value);

        // Enable receiver buffer interrupts
        self.registers.IER.write(IER::ERBFI::Enabled);

        // Set Divisor Latch Access Bit
        self.registers
            .LCR
            .write(LCR::WLEN::EightBit + LCR::DLAB::Enabled);

        // Write DLL and DLM
        self.registers.RB_RH_R.set(0x01);
        self.registers.IER.write(IER::DIVISOR_LATCH_MS.val(0x00));

        // Unset Divisor Latch Access Bit
        self.registers
            .LCR
            .write(LCR::WLEN::EightBit + LCR::DLAB::Disabled);
    }

    /// Send a char
    fn write_char(&mut self, c: char) {
        // Spin while TX FIFO full is set
        while self.registers.IER.matches_all(IER::ETBEI::Enabled) {
            nop();
        }

        self.registers.RB_RH_R.set(c as u8);
        self.chars_written += 1;
    }

    /// Writes all buffered chars
    fn flush(&self) {
        // spin until the busy bit is cleared
        while self.registers.IER.matches_all(IER::ETBEI::Enabled) {
            nop();
        }
    }

    /// Receive char
    fn read_char_converting(
        &mut self,
        blocking_mode: BlockingMode,
    ) -> Option<char> {
        if self.registers.LSR.is_set(LSR::DR) {
            if blocking_mode == BlockingMode::NonBlocking {
                return None;
            }

            while self.registers.IER.matches_all(IER::ERBFI::Disabled) {
                nop();
            }
        }
        let mut ret = self.registers.RB_RH_R.get() as char;
        // Convert \r -> \n
        if ret == '\r' {
            ret = '\n'
        }
        self.chars_read += 1;
        return Some(ret);
    }
}

/// Allows writing formatted strings to UART
impl fmt::Write for NS16550AUartInner {
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

/// Implemenets struct for UART
impl NS16550AUart {
    /// Driver firendly name
    pub const NAME: &'static str = "NS16550A UART";

    /// Instantiates new UART driver with given address
    /// # Safety
    /// Caller must ensure mmio start address is valid for the target hardware
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: synchronisation::NullLock::new(NS16550AUartInner::new(
                mmio_start_addr,
            )),
        }
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------

/// Implementes DeviceDriver trait for UART
impl super::interface::DeviceDriver for NS16550AUart {
    /// Returns a reference to the driver's friednly name
    fn name(&self) -> &'static str {
        Self::NAME
    }

    /// Instantiates a mutex reference to UART driver
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

    /// Writes formatted string to UART guarded by mutex
    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
        self.inner.lock(|inner| fmt::Write::write_fmt(inner, args))
    }

    /// Calls internal flush logic guarded by mutex
    fn flush(&self) {
        self.inner.lock(|inner| inner.flush());
    }
}

impl console::interface::Read for NS16550AUart {
    /// Blocking reads char from UART guarded by mutex
    fn read_char(&self) -> char {
        self.inner.lock(|inner| {
            inner.read_char_converting(BlockingMode::Blocking).unwrap()
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

impl console::interface::Statistics for NS16550AUart {
    /// Returns the characters written statistic guarded by mutex
    fn chars_written(&self) -> usize {
        self.inner.lock(|inner| inner.chars_written)
    }

    /// Returns the characters read statistic guarded by mutex
    fn chars_read(&self) -> usize {
        self.inner.lock(|inner| inner.chars_read)
    }
}

impl console::interface::All for NS16550AUart {}
