//! Board Support Package for the Raspberry Pi 3.
//!
mod memory_map;

use super::driver;
use crate::interface;

// use lazy_static::lazy_static;
// use spin::Mutex;

pub const BOOT_CORE_ID: u64 = 0;
pub const BOOT_CORE_STACK_START: u64 = 0x80_000;
pub const CORE_MASK: u64 = 0x3;

////////////////////////////////////////////////////////////////////////////////
// Global BSP driver instances
////////////////////////////////////////////////////////////////////////////////

static GPIO: driver::GPIO = unsafe { driver::GPIO::new(memory_map::mmio::GPIO_BASE) };
static SYSTIMER: driver::SysTimer =
    unsafe { driver::SysTimer::new(memory_map::mmio::SYSTIMER_BASE) };
static AUX_REGS: driver::AuxRegisters =
    unsafe { driver::AuxRegisters::new(memory_map::mmio::AUX_BASE) };
static RNG: driver::Rng = unsafe { driver::Rng::new(memory_map::mmio::RANDOM_BASE) };
static MINI_UART: driver::MiniUart = unsafe { driver::MiniUart::new(memory_map::mmio::UART1_BASE) };
static MBOX: driver::Mbox = unsafe { driver::Mbox::new(memory_map::mmio::MAILBOX_BASE) };
static UART0: driver::Uart = unsafe { driver::Uart::new(memory_map::mmio::UART0_BASE) };
static USB: driver::USB = unsafe { driver::USB::new(memory_map::mmio::USB_BASE) };

////////////////////////////////////////////////////////////////////////////////
// Global BSP driver getters
////////////////////////////////////////////////////////////////////////////////

pub fn device_drivers() -> [&'static dyn interface::driver::DeviceDriver; 8] {
    [
        &GPIO, &SYSTIMER, &AUX_REGS, &RNG, &MINI_UART, &MBOX, &UART0, &USB,
    ]
}

pub fn init() {
    for i in device_drivers().iter() {
        if let Err(()) = i.init() {
            // This message will only be readable if, at the time of failure,
            // the return value of `bsp::console()` is already in functioning
            // state.
            panic!("Error loading driver: {}", i.compatible())
        }
    }

    GPIO.map_mini_uart();
}

// Returns a ready-to-use `console::Write` implementation.
pub fn console() -> &'static impl interface::console::All {
    &MINI_UART
}

pub fn mailbox() -> &'static driver::Mbox {
    &MBOX
}

pub fn gpio() -> &'static driver::GPIO {
    &GPIO
}

pub fn uart0() -> &'static impl interface::console::All {
    &UART0
}

////////////////////////////////////////////////////////////////////////////////
// Implementation of the kernel's BSP calls
////////////////////////////////////////////////////////////////////////////////

pub fn board_name() -> &'static str {
    "Raspberry Pi 3"
}

// This is kind of an ugly solution, this could be pushed into the Mail implementation
pub fn board_mac() -> u64 {
    let mut mail = driver::Mail::new();

    mail.get_board_mac().unwrap()
}

pub fn rand(min: usize, max: usize) -> usize {
    RNG.rand(min, max)
}

pub fn wait_usec(n: u64) {
    SYSTIMER.wait_usec(n);
}
