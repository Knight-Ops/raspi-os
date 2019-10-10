//! Board Support Package for the Raspberry Pi 3.

// mod gpio;
// mod uart0;
// pub use uart0::Uart;
// mod uart1;
// pub use uart1::MiniUart;
// pub mod mbox;

mod driver;
mod memory_map;
mod panic_wait;
mod sync;
use sync::NullLock;

use crate::interface;
use core::sync::atomic::{compiler_fence, Ordering};
use cortex_a::{asm, regs::*};

// use lazy_static::lazy_static;
// use spin::Mutex;

/// The entry of the `kernel` binary.
///
/// The function must be named `_start`, because the linker is looking for this
/// exact name.
///
/// # Safety
///
/// - Linker script must ensure to place this function at `0x80_000`.
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    use crate::runtime_init;

    const CORE_0: u64 = 0;
    const CORE_MASK: u64 = 0x3;
    const STACK_START: u64 = 0x80_000;

    if CORE_0 == MPIDR_EL1.get() & CORE_MASK {
        SP.set(STACK_START);

        compiler_fence(Ordering::SeqCst);

        // This is a hack to not drop back into assembly. Without this get, the compiler fence was not properly
        // preventing stack allocations prior to the stack being set up.
        if SP.get() != 0 {
            runtime_init::init()
        } else {
            loop {
                asm::wfe();
            }
        }
    } else {
        // if not core0, infinitely wait for events
        loop {
            asm::wfe();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Global BSP driver instances
////////////////////////////////////////////////////////////////////////////////

static GPIO: driver::GPIO = unsafe { driver::GPIO::new(memory_map::mmio::GPIO_BASE) };
static MINI_UART: driver::MiniUart =
    unsafe { driver::MiniUart::new(memory_map::mmio::MiniUart_BASE) };
// static UART_0: driver::Uart = unsafe { driver::Uart::new(memory_map::mmio::Uart_BASE) };
// static MBOX: driver::Mbox = unsafe { driver::MBOX::New(memory_map::mmio::VIDEOCORE_MBOX) };

////////////////////////////////////////////////////////////////////////////////
// Implementation of the kernel's BSP calls
////////////////////////////////////////////////////////////////////////////////

// Returns a ready-to-use `console::Write` implementation.
pub fn console() -> &'static impl interface::console::All {
    &MINI_UART
}

pub fn device_drivers() -> [&'static dyn interface::driver::DeviceDriver; 2] {
    [&GPIO, &MINI_UART]
}
