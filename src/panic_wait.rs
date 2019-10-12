//! A panic handler that infinitely waits.

use crate::{arch, println};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(args) = info.message() {
        println!("Kernel panic: {}", args);
    } else {
        println!("Kernel panic!");
    }

    arch::wait_forever()
}
