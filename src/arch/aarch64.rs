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
            crate::runtime_init::get().init();
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

/// Wait N microseconds
pub fn wait_usec(n: usize) {
    let frq = CNTFRQ_EL0.get() as usize;

    // Calculate number of ticks
    let tick_val = ((frq * n) / 1_000_000) as u32;

    // Set the compare value register
    CNTP_TVAL_EL0.set(tick_val);

    // Kick off the counting                        // Disable timer interrupt
    CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::SET + CNTP_CTL_EL0::IMASK::SET);

    loop {
        // ISTATUS will be one when cval ticks have passed. Continuously check it.
        if CNTP_CTL_EL0.is_set(CNTP_CTL_EL0::ISTATUS) {
            break;
        }
    }

    // Disable counting
    CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::CLEAR);
}

#[inline(always)]
/// Loop forever (Trap state)
pub fn wait_forever() -> ! {
    loop {
        asm::wfe();
    }
}
