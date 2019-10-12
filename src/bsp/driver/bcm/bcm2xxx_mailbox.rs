use crate::{arch, arch::sync::NullLock, interface};
use core::ops;
use core::sync::atomic::{compiler_fence, Ordering};
use register::{
    mmio::{ReadOnly, WriteOnly},
    register_bitfields,
};

pub mod bcm2837_mail;
pub use bcm2837_mail::*;

register_bitfields! {
    u32,

    STATUS [
        FULL  OFFSET(31) NUMBITS(1) [],
        EMPTY OFFSET(30) NUMBITS(1) []
    ]
}

#[allow(non_snake_case)]
#[repr(C)]
struct RegisterBlock {
    READ: [ReadOnly<u32>; 4],                // 0x00
    PEEK: ReadOnly<u32>,                     // 0x10
    SENDER: ReadOnly<u32>,                   // 0x14
    STATUS: ReadOnly<u32, STATUS::Register>, // 0x18
    CONFIG: ReadOnly<u32>,                   // 0x1C
    WRITE: [WriteOnly<u32>; 4],              // 0x20
}

// Custom errors
#[derive(Debug)]
pub enum MboxError {
    ResponseError,
    UnknownError,
}
type Result<T> = ::core::result::Result<T, MboxError>;

// Responses
enum Response {
    Success = 0x8000_0000,
    Error = 0x8000_0001, // error parsing request buffer (partial response)
    UnknownError = 0x0,
}

impl Response {
    fn from(value: u32) -> Response {
        match value {
            0x8000_0000 => Response::Success,
            0x8000_0001 => Response::Error,
            _ => Response::UnknownError,
        }
    }
}

// Public interface to the mailbox
#[repr(C)]
struct MboxInner {
    // This is a really ugly solution to a Mailbox buffer
    // We probably should make this a structure, but we don't
    // have access to dynamically sized Vec, or Box with no_std
    // currently in this phase of init, so it will have to work
    // for now, if we abstract it aware, the user shouldn't care
    base_addr: usize,
}

/// Deref to RegisterBlock
///
/// Allows writing
/// ```
/// self.STATUS.read()
/// ```
/// instead of something along the lines of
/// ```
/// unsafe { (*Mbox::ptr()).STATUS.read() }
/// ```
impl ops::Deref for MboxInner {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl MboxInner {
    pub const fn new(base_addr: usize) -> MboxInner {
        MboxInner { base_addr }
    }

    /// Returns a pointer to the register block
    fn ptr(&self) -> *const RegisterBlock {
        self.base_addr as *const _
    }

    /// Make a mailbox call. Returns Err(MboxError) on failure, Ok(()) success
    pub fn call(&self, mail: &mut Mail, channel: Channel) -> Result<()> {
        // wait until we can write to the mailbox
        loop {
            if !self.STATUS.is_set(STATUS::FULL) {
                break;
            }

            arch::nop();
        }

        let buf_ptr = mail.buffer.as_ptr() as u32;

        // write the address of our message to the mailbox with channel identifier
        self.WRITE[0].set((buf_ptr & !0xF) | ((channel as u32) & 0xF));

        // now wait for the response
        loop {
            // is there a response?
            loop {
                if !self.STATUS.is_set(STATUS::EMPTY) {
                    break;
                }

                arch::nop();
            }

            let resp: u32 = self.READ[0].get();

            // is it a response to our message?
            if ((resp & 0xF) == channel as u32) && ((resp & !0xF) == buf_ptr) {
                // is it a valid successful response?
                return match Response::from(mail.buffer[1]) {
                    Response::Success => Ok(()),
                    Response::Error => Err(MboxError::ResponseError),
                    Response::UnknownError => Err(MboxError::UnknownError),
                };
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// OS interface implementations
////////////////////////////////////////////////////////////////////////////////
use interface::sync::Mutex;

pub struct Mbox {
    inner: NullLock<MboxInner>,
}

impl Mbox {
    pub const unsafe fn new(base_addr: usize) -> Mbox {
        Mbox {
            inner: NullLock::new(MboxInner::new(base_addr)),
        }
    }

    pub fn call(&self, mail: &mut Mail, channel: Channel) -> Result<()> {
        let mut r = &self.inner;
        r.lock(|inner| inner.call(mail, channel))
    }
}

impl interface::driver::DeviceDriver for Mbox {
    fn compatible(&self) -> &str {
        "BCM2XXX Mailbox"
    }
}
