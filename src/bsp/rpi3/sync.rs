use crate::interface;
use core::cell::UnsafeCell;

pub struct NullLock<T: ?Sized> {
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for NullLock<T> {}
unsafe impl<T: ?Sized + Send> Sync for NullLock<T> {}

impl<T> NullLock<T> {
    pub const fn new(data: T) -> NullLock<T> {
        NullLock {
            data: UnsafeCell::new(data),
        }
    }
}

impl<T> interface::sync::Mutex for &NullLock<T> {
    type Data = T;

    fn lock<R>(&mut self, f: impl FnOnce(&mut Self::Data) -> R) -> R {
        f(unsafe { &mut *self.data.get() })
    }
}
