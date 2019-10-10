pub mod mmio {
    pub const BASE: usize = 0x3F00_0000;
    pub const VIDEOCORE_MBOX: usize = BASE + 0x0000_B880;
    pub const GPIO_BASE: usize = BASE + 0x0020_0000;
    pub const Uart_BASE: usize = BASE + 0x0020_1000;
    pub const MiniUart_BASE: usize = BASE + 0x0021_5000;
}
