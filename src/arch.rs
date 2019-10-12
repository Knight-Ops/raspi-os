// Here we export the required architecture for the board

#[cfg(feature = "bsp_rpi3")]
pub mod aarch64;
#[cfg(feature = "bsp_rpi3")]
pub use aarch64::*;
