use super::{Clocks, Mail};
use crate::bsp;
use crate::{arch, arch::sync::NullLock, interface};
use core::fmt;
use core::ops;
use register::{mmio::*, register_bitfields};

register_bitfields! {
    u32,

    DR [
        OE OFFSET(11) NUMBITS(1) [],

        BE OFFSET(10) NUMBITS(1) [],

        PE OFFSET(9) NUMBITS(1) [],

        FE OFFSET(8) NUMBITS(1) [],

        DATA OFFSET(0) NUMBITS(8) []
    ],

    RSRECR [
        OE OFFSET(3) NUMBITS(1) [],
        BE OFFSET(2) NUMBITS(1) [],
        PE OFFSET(1) NUMBITS(1) [],
        FE OFFSET(0) NUMBITS(1) []
    ],

    FR [
        RI OFFSET(8) NUMBITS(1) [],
        TXFE OFFSET(7) NUMBITS(1) [],
        RXFF OFFSET(6) NUMBITS(1) [],
        TXFF OFFSET(5) NUMBITS(1) [],
        RXFE OFFSET(4) NUMBITS(1) [],
        BUSY OFFSET(3) NUMBITS(1) [],
        DCD OFFSET(2) NUMBITS(1) [],
        DSR OFFSET(1) NUMBITS(1) [],
        CTS OFFSET(0) NUMBITS(1) []
    ],

    IBRD [
        IBRD OFFSET(0) NUMBITS(16) []
    ],

    FBRD [
        FBRD OFFSET(0) NUMBITS(6) []
    ],

    LCRH [
        SPS OFFSET(7) NUMBITS(1) [],
        WLEN OFFSET(5) NUMBITS(2) [
            Fivebit = 0b00,
            Sixbit = 0b01,
            Sevenbit = 0b10,
            Eightbit = 0b11
        ],
        FEN OFFSET(4) NUMBITS(1) [],
        STP2 OFFSET(3) NUMBITS(1) [],
        EPS OFFSET(2) NUMBITS(1) [],
        PEN OFFSET(1) NUMBITS(1) [],
        BRK OFFSET(0) NUMBITS(1) []
    ],

    CR [
        CTSEN OFFSET(15) NUMBITS(1) [],
        RTSEN OFFSET(14) NUMBITS(1) [],
        RTS OFFSET(11) NUMBITS(1) [],
        RXE OFFSET(9) NUMBITS(1) [
            Disabled = 0b0,
            Enabled = 0b1
        ],
        TXE OFFSET(8) NUMBITS(1) [
            Disabled = 0b0,
            Enabled = 0b1
        ],
        LBE OFFSET(7) NUMBITS(1) [],
        UARTEN OFFSET(0) NUMBITS(1) [
            Disabled = 0b0,
            Enabled = 0b1
        ]
    ],

    ICR [ // TODO this is wrong
        ALL OFFSET(0) NUMBITS(11) []
    ]
}

//TODO this needs to be completely filled out
#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    DR: ReadWrite<u32>,                       // 0x00
    RSRECR: ReadWrite<u32, RSRECR::Register>, // 0x04
    _reserved0: [u32; 4],                     // 0x08
    FR: ReadWrite<u32, FR::Register>,         // 0x18
    _reserved1: [u32; 1],                     // 0x1c
    ILPR: ReadWrite<u32>,                     // 0x20
    IBRD: ReadWrite<u32, IBRD::Register>,     // 0x24
    FBRD: ReadWrite<u32, FBRD::Register>,     // 0x28
    LCRH: ReadWrite<u32, LCRH::Register>,     // 0x2c
    CR: ReadWrite<u32, CR::Register>,         // 0x30
    _reserved2: [u32; 4],                     //TODO                // 0x34
    ICR: WriteOnly<u32, ICR::Register>,       // 0x44
}

pub enum UartError {
    MailboxError,
}
type Result<T> = ::core::result::Result<T, UartError>;

struct UartInner {
    base_addr: usize,
    chars_written: usize,
}

impl ops::Deref for UartInner {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl UartInner {
    const fn new(base_addr: usize) -> UartInner {
        UartInner {
            base_addr,
            chars_written: 0,
        }
    }

    fn ptr(&self) -> *const RegisterBlock {
        self.base_addr as *const _
    }

    fn init(&self, clock_speed: u32) -> interface::driver::Result {
        self.CR.set(0);

        let mut mail = Mail::new();

        if mail.set_clock_rate(Clocks::UART, clock_speed, 0).unwrap().1 != 4_000_000 {
            arch::wait_forever();
        }

        self.ICR.write(ICR::ALL::CLEAR);
        self.IBRD.write(IBRD::IBRD.val(2));
        self.FBRD.write(FBRD::FBRD.val(0xB));
        self.LCRH.write(LCRH::WLEN::Eightbit);
        self.CR
            .write(CR::UARTEN::Enabled + CR::TXE::Enabled + CR::RXE::Enabled);

        Ok(())
    }

    /// Send a character
    fn write_char(&self, c: char) {
        // wait until we can send
        loop {
            if !self.FR.is_set(FR::TXFF) {
                break;
            }

            arch::nop();
        }

        // write the character to the buffer
        self.DR.set(c as u32);
    }

    /// Receive a character
    fn read_char(&self) -> char {
        // wait until something is in the buffer
        loop {
            if !self.FR.is_set(FR::RXFE) {
                break;
            }

            arch::nop();
        }

        // read it and return
        let mut ret = self.DR.get() as u8 as char;

        // convert carrige return to newline
        if ret == '\r' {
            ret = '\n'
        }

        ret
    }
}

impl fmt::Write for UartInner {
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

pub struct Uart {
    inner: NullLock<UartInner>,
}

impl Uart {
    pub const unsafe fn new(base_addr: usize) -> Uart {
        Uart {
            inner: NullLock::new(UartInner::new(base_addr)),
        }
    }
}

impl interface::driver::DeviceDriver for Uart {
    fn compatible(&self) -> &str {
        "BCM2XXX UART"
    }

    fn init(&self) -> interface::driver::Result {
        let mut r = &self.inner;
        r.lock(|inner| inner.init(4_000_000))
    }
}

impl interface::console::Write for Uart {
    fn write_char(&self, c: char) {
        let mut r = &self.inner;
        r.lock(|inner| inner.write_char(c));
    }

    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
        let mut r = &self.inner;
        r.lock(|inner| fmt::Write::write_fmt(inner, args))
    }
}

impl interface::console::Read for Uart {
    fn read_char(&self) -> char {
        let mut r = &self.inner;
        r.lock(|inner| inner.read_char())
    }
}

impl interface::console::Statistics for Uart {
    fn chars_written(&self) -> usize {
        let mut r = &self.inner;
        r.lock(|inner| inner.chars_written)
    }
}
