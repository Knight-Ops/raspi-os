use crate::arch;

pub fn wait_cyles(cycles: usize) {
    for _ in (0..cycles) {
        arch::nop();
    }
}
