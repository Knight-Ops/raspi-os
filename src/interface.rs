//! Trait definitions for coupling `kernel` and `BSP` code.
//!
//! ```
//!         +-------------------+
//!         | Interface (Trait) |
//!         |                   |
//!         +--+-------------+--+
//!            ^             ^
//!            |             |
//!            |             |
//! +----------+--+       +--+----------+
//! | Kernel code |       |  BSP Code   |
//! |             |       |             |
//! +-------------+       +-------------+
//! ```

/// System console operations.
pub mod console {
    use core::fmt;
    /// Console write functions.
    ///
    /// `core::fmt::Write` is exactly what we need. Re-export it here because
    /// implementing `console::Write` gives a better hint to the reader about
    /// the intention.
    pub trait Write {
        fn write_char(&self, c: char);
        fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
    }

    /// Console read functions.
    pub trait Read {
        fn read_char(&self) -> char {
            ' '
        }
    }

    pub trait Statistics {
        fn chars_written(&self) -> usize {
            0
        }

        fn chars_read(&self) -> usize {
            0
        }
    }

    // This is a full console
    pub trait All = Write + Read + Statistics;
}

pub mod sync {
    pub trait Mutex {
        type Data;

        fn lock<R>(&mut self, f: impl FnOnce(&mut Self::Data) -> R) -> R;
    }
}

pub mod driver {
    pub type Result = core::result::Result<(), ()>;

    pub trait DeviceDriver {
        // Return a compatibility string for identifying the driver.
        fn compatible(&self) -> &str;

        // Called by the kernel to bring the device up
        fn init(&self) -> Result {
            Ok(())
        }
    }
}
