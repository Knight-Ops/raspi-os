use crate::bsp::mailbox;
use crate::{arch, arch::sync::NullLock, interface};
use core::ops;
use core::sync::atomic::{compiler_fence, Ordering};

use super::MboxError;
type Result<T> = ::core::result::Result<T, MboxError>;

// Channels
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum Channel {
    PowerManagement,
    Framebuffer,
    VirtualUART,
    VCHIQ,
    Leds,
    Buttons,
    TouchScreen,
    Count,
    ArmToVCProperty,
    VCToArmProperty,
}

// Tags - Found https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface
#[repr(u32)]
pub enum Tag {
    GetFirmwareRevision = 0x00000001,
    GetBoardModel = 0x00010001,
    GetBoardRevision = 0x00010002,
    GetBoardMAC = 0x00010003,
    GetBoardSerial = 0x10004,
    GetARMMemory = 0x00010005,
    GetVCMemory = 0x00010006,
    GetClocks = 0x00010007,

    // Config
    GetCommandLine = 0x00050001,

    // Shared resource management
    GetDMAChannels = 0x00060001,

    // Power
    GetPowerState = 0x00020001,
    GetTiming = 0x00020002,
    SetPowerState = 0x00028001,

    // Clocks
    GetClockState = 0x00030001,
    GetClockRate = 0x00030002,
    GetMaxClockRate = 0x00030004,
    GetMinClockRate = 0x00030007,
    GetTurbo = 0x00030009,
    SetClockState = 0x00038001,
    SetClockRate = 0x00038002,
    SetTurbo = 0x00038009,

    // Voltage
    GetVoltage = 0x00030003,
    GetMaxVoltage = 0x00030005,
    GetTemperature = 0x00030006,
    GetMinVoltage = 0x00030008,
    GetMaxTemperature = 0x0003000a,
    SetVoltage = 0x00038003,

    // Memory
    AllocateMemory = 0x0003000c,
    LockMemory = 0x0003000d,
    UnlockMemory = 0x0003000e,
    ReleaseMemory = 0x0003000f,
    ExecuteCode = 0x00030010,
    GetDispmanxMemoryHandle = 0x00030014,
    GetEDIDBlock = 0x00030020,

    // FrameBuffer
    AllocateBuffer = 0x00040001,
    BlankScreen = 0x00040002,
    GetPhysicalHeightWidth = 0x00040003,
    GetVirtualHeightWidth = 0x00040004,
    GetDepth = 0x00040005,
    GetPixelOrder = 0x00040006,
    GetAlphaMode = 0x00040007,
    GetPitch = 0x00040008,
    GetVirtualOffset = 0x00040009,
    GetOverscan = 0x0004000a,
    GetPalette = 0x0004000b,
    TestPhysicalHeightWidth = 0x00044003,
    TestVirtualHeightWidth = 0x00044004,
    TestDepth = 0x00044005,
    TestPixelOrder = 0x00044006,
    TestAlphaMode = 0x00044007,
    TestVirtualOffset = 0x00044009,
    TestOverscan = 0x0004400a,
    TestPalette = 0x0004400b,
    ReleaseBuffer = 0x00048001,
    SetPhysicalHeightWidth = 0x00048003,
    SetVirtualHeightWidth = 0x00048004,
    SetDepth = 0x00048005,
    SetPixelOrder = 0x00048006,
    SetAlphaMode = 0x00048007,
    SetVirtualOffset = 0x00048009,
    SetOverscan = 0x0004800a,
    SetPalette = 0x0004800b,

    //Cursor
    SetCursorInfo = 0x00008010,
    SetCursorState = 0x00008011,

    End = 0,
}

#[repr(u32)]
pub enum Power {
    SDCard = 0,
    Uart0 = 1,
    Uart1 = 2,
    USBHCD = 3,
    I2C0 = 4,
    I2C1 = 5,
    I2C2 = 6,
    SPI = 7,
    CCP2TX = 8,
}

#[repr(u32)]
pub enum Clocks {
    _reserved = 0,
    EMMC = 1,
    UART = 2,
    ARM = 3,
    CORE = 4,
    V3D = 5,
    H264 = 6,
    ISP = 7,
    SDRAM = 8,
    PIXEL = 9,
    PWM = 0xa,
    EMMC2 = 0xC,
}

#[repr(u32)]
pub enum Voltage {
    _reserved = 0,
    CORE = 1,
    SDRAM_C = 2,
    SDRAM_P = 3,
    SDRAM_I = 4,
}

#[repr(u32)]
pub enum Request {
    Request = 0,
}

// Public interface to the mailbox
#[repr(C)]
#[repr(align(16))]
pub struct Mail {
    // This is a really ugly solution to a Mailbox buffer
    // We probably should make this a structure, but we don't
    // have access to dynamically sized Vec, or Box with no_std
    // currently in this phase of init, so it will have to work
    // for now, if we abstract it aware, the user shouldn't care
    pub buffer: [u32; 36],
}

impl Mail {
    pub fn new() -> Mail {
        Mail { buffer: [0; 36] }
    }

    pub fn get_board_serial(&mut self) -> Result<u64> {
        self.buffer[0] = 8 * 4;
        self.buffer[1] = Request::Request as u32;
        self.buffer[2] = Tag::GetBoardSerial as u32;
        self.buffer[3] = 8;
        self.buffer[4] = 0;
        self.buffer[5] = 0;
        self.buffer[6] = 0;
        self.buffer[7] = Tag::End as u32;

        compiler_fence(Ordering::Release);

        match mailbox().call(self, Channel::ArmToVCProperty) {
            Err(MboxError::ResponseError) => Err(MboxError::ResponseError),
            Err(MboxError::UnknownError) => Err(MboxError::UnknownError),
            Ok(()) => {
                let result: u64 = (((self.buffer[6] as u64) << 32) | (self.buffer[5] as u64));
                Ok(result)
            }
        }
    }

    pub fn get_firmware_revision(&mut self) -> Result<u32> {
        self.buffer[0] = 7 * 4;
        self.buffer[1] = Request::Request as u32;
        self.buffer[2] = Tag::GetFirmwareRevision as u32;
        self.buffer[3] = 4;
        self.buffer[4] = 0;
        self.buffer[5] = 0;
        self.buffer[6] = Tag::End as u32;

        compiler_fence(Ordering::Release);

        match mailbox().call(self, Channel::ArmToVCProperty) {
            Err(MboxError::ResponseError) => Err(MboxError::ResponseError),
            Err(MboxError::UnknownError) => Err(MboxError::UnknownError),
            Ok(()) => {
                let result: u32 = self.buffer[5];
                Ok(result)
            }
        }
    }

    pub fn get_board_model(&mut self) -> Result<u32> {
        self.buffer[0] = 7 * 4;
        self.buffer[1] = Request::Request as u32;
        self.buffer[2] = Tag::GetBoardModel as u32;
        self.buffer[3] = 4;
        self.buffer[4] = 0;
        self.buffer[5] = 0;
        self.buffer[6] = Tag::End as u32;

        compiler_fence(Ordering::Release);

        match mailbox().call(self, Channel::ArmToVCProperty) {
            Err(MboxError::ResponseError) => Err(MboxError::ResponseError),
            Err(MboxError::UnknownError) => Err(MboxError::UnknownError),
            Ok(()) => {
                let result: u32 = self.buffer[5];
                Ok(result)
            }
        }
    }

    pub fn get_board_revision(&mut self) -> Result<u32> {
        self.buffer[0] = 7 * 4;
        self.buffer[1] = Request::Request as u32;
        self.buffer[2] = Tag::GetBoardRevision as u32;
        self.buffer[3] = 4;
        self.buffer[4] = 0;
        self.buffer[5] = 0;
        self.buffer[6] = Tag::End as u32;

        compiler_fence(Ordering::Release);

        match mailbox().call(self, Channel::ArmToVCProperty) {
            Err(MboxError::ResponseError) => Err(MboxError::ResponseError),
            Err(MboxError::UnknownError) => Err(MboxError::UnknownError),
            Ok(()) => {
                let result: u32 = self.buffer[5];
                Ok(result)
            }
        }
    }

    pub fn get_board_mac(&mut self) -> Result<u64> {
        self.buffer[0] = 8 * 4;
        self.buffer[1] = Request::Request as u32;
        self.buffer[2] = Tag::GetBoardMAC as u32;
        self.buffer[3] = 8;
        self.buffer[4] = 0;
        self.buffer[5] = 0;
        self.buffer[6] = 0;
        self.buffer[7] = Tag::End as u32;

        compiler_fence(Ordering::Release);

        match mailbox().call(self, Channel::ArmToVCProperty) {
            Err(MboxError::ResponseError) => Err(MboxError::ResponseError),
            Err(MboxError::UnknownError) => Err(MboxError::UnknownError),
            Ok(()) => {
                let result: u64 = (((self.buffer[6] as u64) << 32) | (self.buffer[5] as u64));
                Ok(result)
            }
        }
    }

    // TODO this should maybe return its own struct instead of a tuple
    pub fn get_arm_memory(&mut self) -> Result<(u32, u32)> {
        self.buffer[0] = 8 * 4;
        self.buffer[1] = Request::Request as u32;
        self.buffer[2] = Tag::GetARMMemory as u32;
        self.buffer[3] = 8;
        self.buffer[4] = 0;
        self.buffer[5] = 0;
        self.buffer[6] = 0;
        self.buffer[7] = Tag::End as u32;

        compiler_fence(Ordering::Release);

        match mailbox().call(self, Channel::ArmToVCProperty) {
            Err(MboxError::ResponseError) => Err(MboxError::ResponseError),
            Err(MboxError::UnknownError) => Err(MboxError::UnknownError),
            Ok(()) => {
                let result: (u32, u32) = (self.buffer[5], self.buffer[6]);
                Ok(result)
            }
        }
    }

    // TODO this should maybe return its own struct instead of a tuple
    pub fn get_vc_memory(&mut self) -> Result<(u32, u32)> {
        self.buffer[0] = 8 * 4;
        self.buffer[1] = Request::Request as u32;
        self.buffer[2] = Tag::GetVCMemory as u32;
        self.buffer[3] = 8;
        self.buffer[4] = 0;
        self.buffer[5] = 0;
        self.buffer[6] = 0;
        self.buffer[7] = Tag::End as u32;

        compiler_fence(Ordering::Release);

        match mailbox().call(self, Channel::ArmToVCProperty) {
            Err(MboxError::ResponseError) => Err(MboxError::ResponseError),
            Err(MboxError::UnknownError) => Err(MboxError::UnknownError),
            Ok(()) => {
                let result: (u32, u32) = (self.buffer[5], self.buffer[6]);
                Ok(result)
            }
        }
    }

    //TODO Get Clocks
    //pub fn get_clocks(&mut self) -> Result<> {}

    //TODO
    // pub fn get_command_line(&mut self)

    pub fn get_dma_channels(&mut self) -> Result<u32> {
        self.buffer[0] = 7 * 4;
        self.buffer[1] = Request::Request as u32;
        self.buffer[2] = Tag::GetDMAChannels as u32;
        self.buffer[3] = 4;
        self.buffer[4] = 0;
        self.buffer[5] = 0;
        self.buffer[6] = Tag::End as u32;

        compiler_fence(Ordering::Release);

        match mailbox().call(self, Channel::ArmToVCProperty) {
            Err(MboxError::ResponseError) => Err(MboxError::ResponseError),
            Err(MboxError::UnknownError) => Err(MboxError::UnknownError),
            Ok(()) => {
                let result: u32 = self.buffer[5];
                Ok(result)
            }
        }
    }

    // TODO this should maybe return its own struct instead of a tuple
    pub fn get_power_state(&mut self, device: Power) -> Result<(u32, u32)> {
        self.buffer[0] = 8 * 4;
        self.buffer[1] = Request::Request as u32;
        self.buffer[2] = Tag::GetPowerState as u32;
        self.buffer[3] = 8;
        self.buffer[4] = 0;
        self.buffer[5] = device as u32;
        self.buffer[6] = 0;
        self.buffer[7] = Tag::End as u32;

        compiler_fence(Ordering::Release);

        match mailbox().call(self, Channel::ArmToVCProperty) {
            Err(MboxError::ResponseError) => Err(MboxError::ResponseError),
            Err(MboxError::UnknownError) => Err(MboxError::UnknownError),
            Ok(()) => {
                let result: (u32, u32) = (self.buffer[5], self.buffer[6]);
                Ok(result)
            }
        }
    }
    // TODO This should have a state that is it's own type probably
    pub fn get_timing(&mut self, device: Power, state: u32) -> Result<(u32, u32)> {
        self.buffer[0] = 8 * 4;
        self.buffer[1] = Request::Request as u32;
        self.buffer[2] = Tag::GetTiming as u32;
        self.buffer[3] = 8;
        self.buffer[4] = 0;
        self.buffer[5] = device as u32;
        self.buffer[6] = state;
        self.buffer[7] = Tag::End as u32;

        compiler_fence(Ordering::Release);

        match mailbox().call(self, Channel::ArmToVCProperty) {
            Err(MboxError::ResponseError) => Err(MboxError::ResponseError),
            Err(MboxError::UnknownError) => Err(MboxError::UnknownError),
            Ok(()) => {
                let result: (u32, u32) = (self.buffer[5], self.buffer[6]);
                Ok(result)
            }
        }
    }

    // TODO this should maybe return its own struct instead of a tuple
    // TODO This should have a state that is it's own type probably
    pub fn set_power_state(&mut self, device: Power, state: u32) -> Result<(u32, u32)> {
        self.buffer[0] = 8 * 4;
        self.buffer[1] = Request::Request as u32;
        self.buffer[2] = Tag::SetPowerState as u32;
        self.buffer[3] = 8;
        self.buffer[4] = 0;
        self.buffer[5] = device as u32;
        self.buffer[6] = state;
        self.buffer[7] = Tag::End as u32;

        compiler_fence(Ordering::Release);

        match mailbox().call(self, Channel::ArmToVCProperty) {
            Err(MboxError::ResponseError) => Err(MboxError::ResponseError),
            Err(MboxError::UnknownError) => Err(MboxError::UnknownError),
            Ok(()) => {
                let result: (u32, u32) = (self.buffer[5], self.buffer[6]);
                Ok(result)
            }
        }
    }

    // TODO this should maybe return its own struct instead of a tuple
    pub fn get_clock_state(&mut self, clock: Clocks) -> Result<(u32, u32)> {
        self.buffer[0] = 8 * 4;
        self.buffer[1] = Request::Request as u32;
        self.buffer[2] = Tag::GetClockState as u32;
        self.buffer[3] = 8;
        self.buffer[4] = 0;
        self.buffer[5] = clock as u32;
        self.buffer[6] = 0;
        self.buffer[7] = Tag::End as u32;

        compiler_fence(Ordering::Release);

        match mailbox().call(self, Channel::ArmToVCProperty) {
            Err(MboxError::ResponseError) => Err(MboxError::ResponseError),
            Err(MboxError::UnknownError) => Err(MboxError::UnknownError),
            Ok(()) => {
                let result: (u32, u32) = (self.buffer[5], self.buffer[6]);
                Ok(result)
            }
        }
    }

    // TODO this should maybe return its own struct instead of a tuple
    // TODO This should have a state that is it's own type probably
    pub fn set_clock_state(&mut self, clock: Clocks, state: u32) -> Result<(u32, u32)> {
        self.buffer[0] = 8 * 4;
        self.buffer[1] = Request::Request as u32;
        self.buffer[2] = Tag::SetClockState as u32;
        self.buffer[3] = 8;
        self.buffer[4] = 0;
        self.buffer[5] = clock as u32;
        self.buffer[6] = state;
        self.buffer[7] = Tag::End as u32;

        compiler_fence(Ordering::Release);

        match mailbox().call(self, Channel::ArmToVCProperty) {
            Err(MboxError::ResponseError) => Err(MboxError::ResponseError),
            Err(MboxError::UnknownError) => Err(MboxError::UnknownError),
            Ok(()) => {
                let result: (u32, u32) = (self.buffer[5], self.buffer[6]);
                Ok(result)
            }
        }
    }

    // TODO this should maybe return its own struct instead of a tuple
    pub fn get_clock_rate(&mut self, clock: Clocks) -> Result<(u32, u32)> {
        self.buffer[0] = 8 * 4;
        self.buffer[1] = Request::Request as u32;
        self.buffer[2] = Tag::GetClockRate as u32;
        self.buffer[3] = 8;
        self.buffer[4] = 0;
        self.buffer[5] = clock as u32;
        self.buffer[6] = 0;
        self.buffer[7] = Tag::End as u32;

        compiler_fence(Ordering::Release);

        match mailbox().call(self, Channel::ArmToVCProperty) {
            Err(MboxError::ResponseError) => Err(MboxError::ResponseError),
            Err(MboxError::UnknownError) => Err(MboxError::UnknownError),
            Ok(()) => {
                let result: (u32, u32) = (self.buffer[5], self.buffer[6]);
                Ok(result)
            }
        }
    }

    // TODO this should maybe return its own struct instead of a tuple
    // TODO This should have a state that is it's own type probably
    pub fn set_clock_rate(
        &mut self,
        clock: Clocks,
        clock_speed: u32,
        skip_setting_turbo: u32,
    ) -> Result<(u32, u32)> {
        self.buffer[0] = 9 * 4;
        self.buffer[1] = Request::Request as u32;
        self.buffer[2] = Tag::SetClockRate as u32;
        self.buffer[3] = 12;
        self.buffer[4] = 0;
        self.buffer[5] = clock as u32;
        self.buffer[6] = clock_speed;
        self.buffer[7] = skip_setting_turbo;
        self.buffer[8] = Tag::End as u32;

        compiler_fence(Ordering::Release);

        match mailbox().call(self, Channel::ArmToVCProperty) {
            Err(MboxError::ResponseError) => Err(MboxError::ResponseError),
            Err(MboxError::UnknownError) => Err(MboxError::UnknownError),
            Ok(()) => {
                let result: (u32, u32) = (self.buffer[5], self.buffer[6]);
                Ok(result)
            }
        }
    }

    // TODO All of the other functionality of the mailbox is yet to be implemented.
}
