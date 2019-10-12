use crate::{arch, arch::sync::NullLock, interface};
use core::fmt;
use core::ops;
use register::{mmio::*, register_bitfields};

register_bitfields! {
    u32,

    /// SYNOPSIS The AUX_MU_IER_REG register is primary used to enable interrupts
    /// If the DLAB bit in the line control register is set this register gives access to the MS 8 bits
    /// of the baud rate. (Note: there is easier access to the baud rate register)
    AUX_MU_IER [
        /// If this bit is set the interrupt line is asserted whenever
        /// the transmit FIFO is empty.
        /// If this bit is clear no transmit interrupts are generated.
        TRANSMIT_INTERRUPTS OFFSET(1) NUMBITS(1) [],
        /// If this bit is set the interrupt line is asserted whenever
        /// the transmit FIFO holds at least 1 byte.
        /// If this bit is clear no receieve interrupts are generated.
        RECEIVE_INTERRUPTS OFFSET(1) NUMBITS(1) []
    ],

    /// SYNOPSIS The AUX_MU_IIR_REG register shows the interrupt status.
    /// It also has two FIFO enable status bits and (when writing) FIFO clear bits.
    AUX_MU_IIR [
        /// On read this register shows the interrupt ID bit
        /// 00 : No interrupts
        /// 01 : Transmit holding register empty
        /// 10 : Receiver holds valid byte
        /// 11 : <Not possible>
        /// On write:
        /// Writing with bit 1 set will clear the receive FIFO
        /// Writing with bit 2 set will clear the transmit FIFO
        FIFO_CLEAR OFFSET(1) NUMBITS(2) [
            None = 0b00,
            Rx = 0b01,
            Tx = 0b10,
            All = 0b11
        ],
        /// This bit is clear whenever an interrupt is pending
        INTERRUPT_PENDING OFFSET(0) NUMBITS(1) []
    ],

    /// SYNOPSIS The AUX_MU_LCR_REG register controls the line data format and gives access to the
    /// baudrate register
    AUX_MU_LCR [
        /// If set the first to Mini UART register give access the
        /// the Baudrate register. During operation this bit must
        /// be cleared.
        DLAB_ACCESS OFFSET(7) NUMBITS(1) [],
        /// If set high the UART1_TX line is pulled low
        /// continuously. If held for at least 12 bits times that will
        /// indicate a break condition.
        BREAK OFFSET(6) NUMBITS(1) [],
        /// 00 : the UART works in 7-bit mode
        /// 11 : the UART works in 8-bit mode
        DATA_SIZE OFFSET(0) NUMBITS(2) [
            SevenBit = 0b00,
            EightBit = 0b11
        ]
    ],

    /// SYNOPSIS The AUX_MU_MCR_REG register controls the 'modem' signals
    AUX_MU_MCR [
        /// If clear the UART1_RTS line is high
        /// If set the UART1_RTS line is low
        /// This bit is ignored if the RTS is used for auto-flow
        /// control. See the Mini Uart Extra Control register
        /// description)
        RTS OFFSET(1) NUMBITS(1) []
    ],

    /// SYNOPSIS The AUX_MU_LSR_REG register shows the data status.
    AUX_MU_LSR [
        /// This bit is set if the transmit FIFO is empty and the
        /// transmitter is idle. (Finished shifting out the last bit).
        TX_IDLE OFFSET(6) NUMBITS(1) [],
        /// This bit is set if the transmit FIFO can accept at least
        /// one byte.
        TX_EMPTY OFFSET(5) NUMBITS(1) [],
        /// This bit is set if there was a receiver overrun. That is:
        /// one or more characters arrived whilst the receive
        /// FIFO was full. The newly arrived charters have been
        /// discarded. This bit is cleared each time this register is
        /// read. To do a non-destructive read of this overrun bit
        /// use the Mini Uart Extra Status register.
        RX_OVERRUN OFFSET(1) NUMBITS(1) [],
        /// This bit is set if the receive FIFO holds at least 1
        /// symbol.
        DATA_READY OFFSET(0) NUMBITS(1) []
    ],

    /// SYNOPSIS The AUX_MU_MSR_REG register shows the 'modem' status
    AUX_MU_MSR [
        /// This bit is the inverse of the UART1_CTS input Thus:
        /// If set the UART1_CTS pin is low
        /// If clear the UART1_CTS pin is high
        CTS_STATUS OFFSET(5) NUMBITS(1) []
    ],

    /// SYNOPSIS The AUX_MU_CNTL_REG provides access to some extra useful and nice features not found on a normal 16550 UART .
    AUX_MU_CNTL [
        /// This bit allows one to invert the CTS auto flow
        /// operation polarity.
        /// If set the CTS auto flow assert level is low*
        /// If clear the CTS auto flow assert level is high*
        CTS_ASSERT_LEVEL OFFSET(7) NUMBITS(1) [],
        /// This bit allows one to invert the RTS auto flow
        /// operation polarity.
        /// If set the RTS auto flow assert level is low*
        /// If clear the RTS auto flow assert level is high*
        RTS_ASSERT_LEVEL OFFSET(6) NUMBITS(1) [],
        /// These two bits specify at what receiver FIFO level the RTS line is de-asserted in auto-flow mode.
        /// 00 : De-assert RTS when the receive FIFO has 3empty spaces left.
        /// 01 : De-assert RTS when the receive FIFO has 2empty spaces left.
        /// 10 : De-assert RTS when the receive FIFO has 1empty space left.
        /// 11 : De-assert RTS when the receive FIFO has 4empty spaces left.
        RTS_AUTO_FLOW OFFSET(4) NUMBITS(2) [],
        /// If this bit is set the transmitter will stop if the CTS line is de-asserted.
        /// If this bit is clear the transmitter will ignore the status of the CTS line
        EN_TX_AUTO_FLOW_USING_CTS OFFSET(3) NUMBITS(1) [],
        /// If this bit is set the mini UART transmitter is enabled.
        /// If this bit is clear the mini UART transmitter is disabled.
        TX_EN OFFSET(1) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// If this bit is set the mini UART receiver is enabled.
        /// If this bit is clear the mini UART receiver is disabled.
        RX_EN OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ]
    ],

    /// SYNOPSIS The AUX_MU_STAT_REG provides a lot of useful information about the internal status of the mini UART not found on a normal 16550 UART.
    AUX_MU_STAT [
        /// These bits shows how many symbols are stored in the transmit FIFO
        /// The value is in the range 0-8
        TX_FIFO_FILL_LEVEL OFFSET(24) NUMBITS(3) [],
        /// These bits shows how many symbols are stored in the receive FIFO
        /// The value is in the range 0-8
        RX_FIFO_FILL_LEVEL OFFSET(16) NUMBITS(3) [],
        /// This bit is set if the transmitter is idle and the transmit FIFO is empty.
        /// It is a logic AND of bits 2 and 8
        TX_DONE OFFSET(9) NUMBITS(1) [],
        /// If this bit is set the transmitter FIFO is empty. Thus it can accept 8 symbols
        TX_FIFO_EMPTY OFFSET(8) NUMBITS(1) [],
        /// This bit shows the status of the UART1_CTS line
        CTS OFFSET(7) NUMBITS(1) [],
        /// This bit shows the status of the UART1_RTS line
        RTS OFFSET(6) NUMBITS(1) [],
        /// This is the inverse of bit 1
        TX_FIFO_FULL OFFSET(5) NUMBITS(1) [],
        /// This bit is set if there was a receiver overrun. That is:
        /// one or more characters arrived whilst the receive
        /// FIFO was full. The newly arrived characters have
        /// been discarded. This bit is cleared each time the
        /// AUX_MU_LSR_REG register is read.
        RX_OVERRUN OFFSET(4) NUMBITS(1) [],
        /// If this bit is set the transmitter is idle.
        /// If this bit is clear the transmitter is idle.
        TX_IDLE OFFSET(3) NUMBITS(1) [],
        /// If this bit is set the receiver is idle.
        /// If this bit is clear the receiver is busy.
        /// This bit can change unless the receiver is disabled
        RX_IDLE OFFSET(2) NUMBITS(1) [],
        /// If this bit is set the mini UART transmitter FIFO can accept at least one more symbol.
        /// If this bit is clear the mini UART transmitter FIFO is full
        TX_AVAILABLE OFFSET(1) NUMBITS(1) [],
        /// If this bit is set the mini UART receiver FIFO can accept at least one more symbol.
        /// If this bit is clear the mini UART receiver FIFO is full
        RX_AVAILABLE OFFSET(0) NUMBITS(1) []
    ],

    /// SYNOPSIS The AUX_MU_BAUD register allows direct access to the 16-bit wide baudrate counter
    AUX_MU_BAUD [
        /// Mini UART baudrate counter
        RATE OFFSET(0) NUMBITS(16) []
    ]
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    AUX_MU_IO: ReadWrite<u32>,  // 0x00 - Mini Uart I/O Data
    AUX_MU_IER: ReadWrite<u32>, // 0x04 - Mini Uart Interrupt Enable
    AUX_MU_IIR: ReadWrite<u32, AUX_MU_IIR::Register>, // 0x08
    AUX_MU_LCR: ReadWrite<u32, AUX_MU_LCR::Register>, // 0x0C
    AUX_MU_MCR: ReadWrite<u32>, // 0x10
    AUX_MU_LSR: ReadOnly<u32, AUX_MU_LSR::Register>, // 0x14
    AUX_MU_MSR: ReadOnly<u32, AUX_MU_MSR::Register>,
    AUX_MU_SCRATCH: ReadWrite<u32>,
    AUX_MU_CNTL: WriteOnly<u32, AUX_MU_CNTL::Register>, // 0x20
    AUX_MU_STAT: ReadOnly<u32, AUX_MU_STAT::Register>,  // 0x24
    AUX_MU_BAUD: ReadWrite<u32, AUX_MU_BAUD::Register>, // 0x28
}

struct MiniUartInner {
    base_addr: usize,
    chars_written: usize,
}

impl ops::Deref for MiniUartInner {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl MiniUartInner {
    const fn new(base_addr: usize) -> MiniUartInner {
        MiniUartInner {
            base_addr,
            chars_written: 0,
        }
    }

    fn ptr(&self) -> *const RegisterBlock {
        self.base_addr as *const _
    }

    fn init(&self) -> interface::driver::Result {
        // initialize UART
        self.AUX_MU_IER.set(0);
        self.AUX_MU_CNTL.set(0);
        self.AUX_MU_LCR.write(AUX_MU_LCR::DATA_SIZE::EightBit);
        self.AUX_MU_MCR.set(0);
        self.AUX_MU_IER.set(0);
        self.AUX_MU_IIR.write(AUX_MU_IIR::FIFO_CLEAR::All);
        self.AUX_MU_BAUD.write(AUX_MU_BAUD::RATE.val(270)); // 115200 baud

        self.AUX_MU_CNTL
            .write(AUX_MU_CNTL::RX_EN::Enabled + AUX_MU_CNTL::TX_EN::Enabled);

        self.AUX_MU_IIR.write(AUX_MU_IIR::FIFO_CLEAR::All);

        Ok(())
    }

    fn write_char(&mut self, c: char) {
        loop {
            if self.AUX_MU_LSR.is_set(AUX_MU_LSR::TX_EMPTY) {
                break;
            }

            arch::nop();
        }

        self.AUX_MU_IO.set(c as u32);
    }

    fn read_char(&self) -> char {
        loop {
            if self.AUX_MU_LSR.is_set(AUX_MU_LSR::DATA_READY) {
                break;
            }

            arch::nop();
        }

        let mut ret = self.AUX_MU_IO.get() as u8 as char;

        if ret == '\r' {
            ret = '\n'
        }

        ret
    }
}

impl fmt::Write for MiniUartInner {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            if c == '\n' {
                self.write_char('\r');
            }

            self.write_char(c);
        }

        self.chars_written += s.len();

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
// OS interface implementations
////////////////////////////////////////////////////////////////////////////////
use interface::sync::Mutex;

pub struct MiniUart {
    inner: NullLock<MiniUartInner>,
}

impl MiniUart {
    pub const unsafe fn new(base_addr: usize) -> MiniUart {
        MiniUart {
            inner: NullLock::new(MiniUartInner::new(base_addr)),
        }
    }
}

impl interface::driver::DeviceDriver for MiniUart {
    fn compatible(&self) -> &str {
        "BCM2XXX MiniUart"
    }

    fn init(&self) -> interface::driver::Result {
        let mut r = &self.inner;
        r.lock(|inner| inner.init())
    }
}

impl interface::console::Write for MiniUart {
    fn write_char(&self, c: char) {
        let mut r = &self.inner;
        r.lock(|inner| inner.write_char(c));
    }

    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
        let mut r = &self.inner;
        r.lock(|inner| fmt::Write::write_fmt(inner, args))
    }
}

impl interface::console::Read for MiniUart {
    fn read_char(&self) -> char {
        let mut r = &self.inner;
        r.lock(|inner| inner.read_char())
    }
}

impl interface::console::Statistics for MiniUart {
    fn chars_written(&self) -> usize {
        let mut r = &self.inner;
        r.lock(|inner| inner.chars_written)
    }
}
