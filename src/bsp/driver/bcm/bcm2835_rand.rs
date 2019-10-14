use crate::bsp;
use crate::{arch, arch::sync::NullLock, interface};
use core::fmt;
use core::ops;
use register::{mmio::*, register_bitfields};

register_bitfields! {
    u32,

    CTRL [
        ENABLE OFFSET(0) NUMBITS(1) [
            True = 1,
            False = 0
        ]
    ],

    INT_MASK [
        INT_OFF OFFSET(0) NUMBITS(1) [
            True = 1,
            False = 0
        ]
    ]
}

const RNG_WARMUP_COUNT: u32 = 0x40_000;

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    CTRL: ReadWrite<u32, CTRL::Register>,
    STATUS: ReadWrite<u32>,
    DATA: ReadOnly<u32>,
    FF_THRESHOLD: ReadWrite<u32>,
    INT_MASK: ReadWrite<u32, INT_MASK::Register>,
}

struct RngInner {
    base_addr: usize,
}

impl ops::Deref for RngInner {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl RngInner {
    pub const fn new(base_addr: usize) -> RngInner {
        RngInner { base_addr }
    }

    fn ptr(&self) -> *const RegisterBlock {
        self.base_addr as *const _
    }

    fn init(&self) {
        self.INT_MASK.modify(INT_MASK::INT_OFF::True);

        self.STATUS.set(RNG_WARMUP_COUNT);
        self.CTRL.modify(CTRL::ENABLE::True);
    }

    fn rand(&self, min: usize, max: usize) -> usize {
        loop {
            if (self.STATUS.get() >> 24) != 0 {
                break;
            }

            arch::nop();
        }

        // here we have a u32
        let l = self.DATA.get();
        let r = self.DATA.get();

        let rand = ((l as usize) << 32) | (r as usize);

        rand % (max - min) + min
    }
}

////////////////////////////////////////////////////////////////////////////////
// OS interface implementations
////////////////////////////////////////////////////////////////////////////////
use interface::sync::Mutex;

pub struct Rng {
    inner: NullLock<RngInner>,
}

impl Rng {
    pub const fn new(base_addr: usize) -> Rng {
        Rng {
            inner: NullLock::new(RngInner::new(base_addr)),
        }
    }

    pub fn rand(&self, min: usize, max: usize) -> usize {
        let mut r = &self.inner;
        r.lock(|inner| inner.rand(min, max))
    }
}

impl interface::driver::DeviceDriver for Rng {
    fn compatible(&self) -> &str {
        "BCM2835 Hardware Random Number Generator"
    }

    fn init(&self) -> interface::driver::Result {
        let mut r = &self.inner;
        r.lock(|inner| inner.init());

        Ok(())
    }
}
