use crate::bsp;
use crate::{arch, arch::sync::NullLock, interface};
use core::fmt;
use core::ops;
use register::{mmio::*, register_bitfields};

register_bitfields! {
    u32,

    /// Synopsis System Timer Control / Status.
    CS [

        /// System Timer Match 3
        /// 0 = No Timer 3 match since last cleared.
        /// 1 = Timer 3 match detected.
        MATCH3 OFFSET(3) NUMBITS(1) [
            True = 1,
            False = 0
        ],

        /// System Timer Match 2
        /// 0 = No Timer 2 match since last cleared.
        /// 1 = Timer 2 match detected.
        MATCH2 OFFSET(2) NUMBITS(1) [
            True = 1,
            False = 0
        ],

        /// System Timer Match 1
        /// 0 = No Timer 1 match since last cleared.
        /// 1 = Timer 1 match detected.
        MATCH1 OFFSET(1) NUMBITS(1) [
            True = 1,
            False = 0
        ],

        /// System Timer Match 0
        /// 0 = No Timer 0 match since last cleared.
        /// 1 = Timer 0 match detected.
        MATCH0 OFFSET(0) NUMBITS(0) [
            True = 1,
            False = 0
        ]
    ]
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    /// Synopsis System Timer Control / Status.
    /// This register is used to record and clear timer channel comparator matches. The system timer match bits
    /// are routed to the interrupt controller where they can generate an interrupt.
    /// The M0-3 fields contain the free-running counter match status. Write a one to the relevant bit to clear the
    /// match detect status bit and the corresponding interrupt request line.
    CS: ReadWrite<u32, CS::Register>,
    /// Synopsis System Timer Counter Lower bits.
    /// The system timer free-running counter lower register is a read-only register that returns the current value
    /// of the lower 32-bits of the free running counter.
    CLO: ReadOnly<u32>,
    /// Synopsis System Timer Counter Higher bits.
    /// The system timer free-running counter higher register is a read-only register that returns the current value
    /// of the higher 32-bits of the free running counter.
    CHI: ReadOnly<u32>,
    /// Synopsis System Timer Compare.
    /// The system timer compare registers hold the compare value for each of the four timer channels.
    /// Whenever the lower 32-bits of the free-running counter matches one of the compare values the
    /// corresponding bit in the system timer control/status register is set.
    /// Each timer peripheral (minirun and run) has a set of four compare registers
    C0: ReadWrite<u32>,
    /// Synopsis System Timer Compare.
    /// The system timer compare registers hold the compare value for each of the four timer channels.
    /// Whenever the lower 32-bits of the free-running counter matches one of the compare values the
    /// corresponding bit in the system timer control/status register is set.
    /// Each timer peripheral (minirun and run) has a set of four compare registers
    C1: ReadWrite<u32>,
    /// Synopsis System Timer Compare.
    /// The system timer compare registers hold the compare value for each of the four timer channels.
    /// Whenever the lower 32-bits of the free-running counter matches one of the compare values the
    /// corresponding bit in the system timer control/status register is set.
    /// Each timer peripheral (minirun and run) has a set of four compare registers
    C2: ReadWrite<u32>,
    /// Synopsis System Timer Compare.
    /// The system timer compare registers hold the compare value for each of the four timer channels.
    /// Whenever the lower 32-bits of the free-running counter matches one of the compare values the
    /// corresponding bit in the system timer control/status register is set.
    /// Each timer peripheral (minirun and run) has a set of four compare registers
    C3: ReadWrite<u32>,
}

struct SysTimerInner {
    base_addr: usize,
}

impl ops::Deref for SysTimerInner {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl SysTimerInner {
    const fn new(base_addr: usize) -> SysTimerInner {
        SysTimerInner { base_addr }
    }

    fn ptr(&self) -> *const RegisterBlock {
        self.base_addr as *const _
    }

    fn init(&self) {
        self.C0.set(0);
        self.C1.set(0);
        self.C2.set(0);
        self.C3.set(0);
    }

    // We technically should not have to lock out reads in the NullLock (or other Mutex) since multiple-reads
    // not cause any issues. We are going to do it anyways (hit to performance) in order to make sure we avoid
    // any possible case of a read and write happening at the *exact* same time.

    fn get_cs_register(&self) -> u32 {
        self.CS.get()
    }

    fn get_systimer(&self) -> u64 {
        let mut high = self.CHI.get();
        let mut low = self.CLO.get();

        if high != self.CHI.get() {
            high = self.CHI.get();
            low = self.CLO.get();
        }

        ((high as u64) << 32) | (low as u64)
    }

    /// Get C0 Register Value
    fn get_c0_register(&self) -> u32 {
        self.C0.get()
    }

    /// Set C0 Register Value
    fn set_c0_register(&mut self, val: u32) {
        self.C0.set(val);
    }

    /// Get C1 Register Value
    fn get_c1_register(&self) -> u32 {
        self.C1.get()
    }

    /// Set C1 Register Value
    fn set_c1_register(&mut self, val: u32) {
        self.C1.set(val);
    }

    /// Get C2 Register Value
    fn get_c2_register(&self) -> u32 {
        self.C2.get()
    }

    /// Set C2 Register Value
    fn set_c2_register(&mut self, val: u32) {
        self.C2.set(val);
    }

    /// Get C3 Register Value
    fn get_c3_register(&self) -> u32 {
        self.C3.get()
    }

    /// Set C3 Register Value
    fn set_c3_register(&mut self, val: u32) {
        self.C3.set(val);
    }
}

////////////////////////////////////////////////////////////////////////////////
// OS interface implementations
////////////////////////////////////////////////////////////////////////////////
use interface::sync::Mutex;

pub struct SysTimer {
    inner: NullLock<SysTimerInner>,
}

impl SysTimer {
    pub const fn new(base_addr: usize) -> SysTimer {
        SysTimer {
            inner: NullLock::new(SysTimerInner::new(base_addr)),
        }
    }

    pub fn get_cs_register(&self) -> u32 {
        let mut r = &self.inner;
        r.lock(|inner| inner.get_cs_register())
    }

    pub fn get_systimer(&self) -> u64 {
        let mut r = &self.inner;
        r.lock(|inner| inner.get_systimer())
    }

    pub fn wait_usec(&self, n: u64) {
        let t = self.get_systimer();

        if t > 0 {
            loop {
                if self.get_systimer() > (t + n) {
                    break;
                }
            }
        }
    }

    /// Get C0 Register Value
    pub fn get_c0_register(&self) -> u32 {
        let mut r = &self.inner;
        r.lock(|inner| inner.C0.get())
    }

    /// Set C0 Register Value
    pub fn set_c0_register(&mut self, val: u32) {
        let mut r = &self.inner;
        r.lock(|inner| inner.C0.set(val));
    }

    /// Get C1 Register Value
    pub fn get_c1_register(&self) -> u32 {
        let mut r = &self.inner;
        r.lock(|inner| inner.C1.get())
    }

    /// Set C1 Register Value
    pub fn set_c1_register(&mut self, val: u32) {
        let mut r = &self.inner;
        r.lock(|inner| inner.C1.set(val));
    }

    /// Get C2 Register Value
    pub fn get_c2_register(&self) -> u32 {
        let mut r = &self.inner;
        r.lock(|inner| inner.C2.get())
    }

    /// Set C2 Register Value
    pub fn set_c2_register(&mut self, val: u32) {
        let mut r = &self.inner;
        r.lock(|inner| inner.C2.set(val));
    }

    /// Get C3 Register Value
    pub fn get_c3_register(&self) -> u32 {
        let mut r = &self.inner;
        r.lock(|inner| inner.C3.get())
    }

    /// Set C3 Register Value
    pub fn set_c3_register(&mut self, val: u32) {
        let mut r = &self.inner;
        r.lock(|inner| inner.C3.set(val));
    }
}

impl interface::driver::DeviceDriver for SysTimer {
    fn compatible(&self) -> &str {
        "BCM2835 System Timer"
    }

    fn init(&self) -> interface::driver::Result {
        let mut r = &self.inner;
        r.lock(|inner| inner.init());

        Ok(())
    }
}
