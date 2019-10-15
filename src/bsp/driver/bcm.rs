// BCM SoC drivers

mod bcm2835_rand;
mod bcm2835_systimer;
mod bcm2837_gpio;
mod bcm2xxx_aux;
mod bcm2xxx_mailbox;
mod bcm2xxx_mini_uart;
mod bcm2xxx_uart;

pub use bcm2835_rand::Rng;
pub use bcm2835_systimer::SysTimer;
pub use bcm2837_gpio::GPIO;
pub use bcm2xxx_aux::AuxRegisters;
pub use bcm2xxx_mailbox::Mail;
pub use bcm2xxx_mailbox::Mbox;
pub use bcm2xxx_mini_uart::MiniUart;
pub use bcm2xxx_uart::Uart;

// Here we get all the pub structs/enums from bcm2xxx_mailbox::bcm2837_mail so that we can type check
// our various function calls elsewhere.
use bcm2xxx_mailbox::*;
