use crate::{arch, arch::sync::NullLock, interface};
use core::fmt;
use core::ops;
use register::{mmio::*, register_bitfields};

register_bitfields! {
    u32,

    /// SYNOPSIS The AUXIRQ register is used to check any pending interrupts which may be asserted by
    /// the three Auxiliary sub blocks.
    AUX_IRQ [
        /// If set the SPI 2 module has an interrupt pending.
        SPI2_IRQ OFFSET(2) NUMBITS(1) [],

        /// If set the SPI 1 module has an interrupt pending.
        SPI1_IRQ OFFSET(1) NUMBITS(1) [],

        /// If set the Mini UART module has an interrupt pending.
        MINI_UART_IRQ OFFSET(0) NUMBITS(1) []
    ],

    /// SYNOPSIS The AUXENB register is used to enable the three modules; UART, SPI1, SPI2.
    AUX_ENABLES [
        /// If set the SPI 2 is enabled.
        /// If clear the SPI 2 is disabled. That also disables any
        /// SPI 2 register access
        SPI2_ENABLE OFFSET(2) NUMBITS(1) [],
        /// If set the SPI 1 is enabled.
        /// If clear the SPI 1 is disabled. That also disables any
        /// SPI 1 register access
        SPI1_ENABLE OFFSET(1) NUMBITS(1) [],
        /// If set the mini UART is enabled. The UART will immediately
        /// start receiving data, especially if the UART1_RX line is
        /// low.
        /// If clear the mini UART is disabled. That also disables any
        /// mini UART register access
        MINI_UART_ENABLE OFFSET(0) NUMBITS(1) []
    ]
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    AUX_IRQ: ReadOnly<u32, AUX_IRQ::Register>,          // 0x00
    AUX_ENABLES: ReadWrite<u32, AUX_ENABLES::Register>, // 0x04
}

struct AuxRegistersInner {
    base_addr: usize,
}

impl ops::Deref for AuxRegistersInner {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl AuxRegistersInner {
    const fn new(base_addr: usize) -> AuxRegistersInner {
        AuxRegistersInner { base_addr }
    }

    fn ptr(&self) -> *const RegisterBlock {
        self.base_addr as *const _
    }

    fn get_spi2_irq(&self) -> bool {
        self.AUX_IRQ.is_set(AUX_IRQ::SPI2_IRQ)
    }

    fn get_spi1_irq(&self) -> bool {
        self.AUX_IRQ.is_set(AUX_IRQ::SPI1_IRQ)
    }

    fn get_mini_uart_irq(&self) -> bool {
        self.AUX_IRQ.is_set(AUX_IRQ::MINI_UART_IRQ)
    }

    fn get_spi2_enabled(&self) -> bool {
        self.AUX_ENABLES.is_set(AUX_ENABLES::SPI2_ENABLE)
    }

    fn get_spi1_enabled(&self) -> bool {
        self.AUX_ENABLES.is_set(AUX_ENABLES::SPI1_ENABLE)
    }

    fn get_mini_uart_enabled(&self) -> bool {
        self.AUX_ENABLES.is_set(AUX_ENABLES::MINI_UART_ENABLE)
    }

    fn set_spi2(&mut self, value: bool) {
        if value == true {
            self.AUX_ENABLES.modify(AUX_ENABLES::SPI2_ENABLE::SET)
        } else {
            self.AUX_ENABLES.modify(AUX_ENABLES::SPI2_ENABLE::CLEAR)
        }
    }

    fn set_spi1(&mut self, value: bool) {
        if value == true {
            self.AUX_ENABLES.modify(AUX_ENABLES::SPI1_ENABLE::SET)
        } else {
            self.AUX_ENABLES.modify(AUX_ENABLES::SPI1_ENABLE::CLEAR)
        }
    }

    fn set_mini_uart(&mut self, value: bool) {
        if value == true {
            self.AUX_ENABLES.modify(AUX_ENABLES::MINI_UART_ENABLE::SET)
        } else {
            self.AUX_ENABLES
                .modify(AUX_ENABLES::MINI_UART_ENABLE::CLEAR)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// OS interface implementations
////////////////////////////////////////////////////////////////////////////////
use interface::sync::Mutex;

pub struct AuxRegisters {
    inner: NullLock<AuxRegistersInner>,
}

impl AuxRegisters {
    pub const unsafe fn new(base_addr: usize) -> AuxRegisters {
        AuxRegisters {
            inner: NullLock::new(AuxRegistersInner::new(base_addr)),
        }
    }

    pub fn enable_mini_uart(&self) {
        let mut r = &self.inner;
        r.lock(|inner| inner.set_mini_uart(true))
    }
}

impl interface::driver::DeviceDriver for AuxRegisters {
    fn compatible(&self) -> &str {
        "BCM2XXX AUX Register"
    }

    fn init(&self) -> interface::driver::Result {
        self.enable_mini_uart();
        Ok(())
    }
}
