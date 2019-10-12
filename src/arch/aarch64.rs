pub mod sync;
use crate::bsp;
use core::sync::atomic::{compiler_fence, Ordering};
use cortex_a::{asm, regs::*};

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

    if bsp::BOOT_CORE_ID == MPIDR_EL1.get() & bsp::CORE_MASK {
        SP.set(bsp::BOOT_CORE_STACK_START);

        compiler_fence(Ordering::SeqCst);

        // This is a hack to not drop back into assembly. Without this get, the compiler fence was not properly
        // preventing stack allocations prior to the stack being set up.
        if SP.get() != 0 {
            runtime_init::init()
        } else {
            wait_forever()
        }
    } else {
        // if not core0, infinitely wait for events
        wait_forever()
    }
}

////////////////////////////////////////////////////////////////////////////////
// Implementation of the kernel's architecture abstraction code
////////////////////////////////////////////////////////////////////////////////

pub use asm::nop;

#[inline(always)]
pub fn wait_forever() -> ! {
    loop {
        asm::wfe();
    }
}
