#[cfg(feature = "bsp_rpi3")]
mod bcm;
#[cfg(feature = "bsp_rpi3")]
mod dwc_usb_2_0_hs_otg;

#[cfg(feature = "bsp_rpi3")]
pub use bcm::*;
#[cfg(feature = "bsp_rpi3")]
pub use dwc_usb_2_0_hs_otg::USB;
