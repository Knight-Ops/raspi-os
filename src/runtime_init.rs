pub trait RuntimeInit {
    unsafe fn init(&self) -> ! {
        extern "C" {
            static mut __bss_start: u64;
            static mut __bss_end: u64;
        }

        r0::zero_bss(&mut __bss_start, &mut __bss_end);

        crate::kernel_entry()
    }
}

struct Secret;

impl RuntimeInit for Secret {}

pub fn get() -> &'static dyn RuntimeInit {
    &Secret
}
