use crate::{arch, arch::sync::NullLock, interface};
use core::ops;
use register::{mmio::*, register_bitfields};

const RECEIVE_FIFO_SIZE: u32 = 20480;
const NON_PERIDOIC_FIFO_SIZE: u32 = 20480;
const PERIODIC_FIFO_SIZE: u32 = 20480;
const CONTROL_MESSAGE_TIMEOUT: u32 = 10;
const HOSTPORTMASK: u32 = !0x2E;
const DWC_NUM_CHANNELS: u32 = 8;
const USB2_MAX_PACKET_SIZE: u32 = 1024;

register_bitfields! {
    u32,

    CHANNEL_INTERRUPTS [
        TRANSFER_COMPLETE OFFSET(0) NUMBITS(1) [],
        HALT OFFSET(1) NUMBITS(1) [],
        AHB_ERROR OFFSET(2) NUMBITS(1) [],
        STALL OFFSET(3) NUMBITS(1) [],
        NEGATIVE_ACKNOWLEDGEMENT OFFSET(4) NUMBITS(1) [],
        ACKNOWLEDGEMENT OFFSET(5) NUMBITS(1) [],
        NOT_YET OFFSET(6) NUMBITS(1) [],
        TRANSACTION_ERROR OFFSET(7) NUMBITS(1) [],
        BABBLE_ERROR OFFSET(8) NUMBITS(1) [],
        FRAME_OVERRUN OFFSET(9) NUMBITS(1) [],
        DATA_TOGGLE_ERROR OFFSET(10) NUMBITS(1) [],
        BUFFER_NOT_AVAILABLE OFFSET(11) NUMBITS(1) [],
        EXCESSIVE_TRANSMISSION OFFSET(12) NUMBITS(1) [],
        FRAME_LIST_ROLLOVER OFFSET(13) NUMBITS(1) []
    ],

    FIFO_SIZE [
        START_ADDRESS OFFSET(0) NUMBITS(16) [],
        DEPTH OFFSET(16) NUMBITS(16) []
    ],

    CORE_OTG_CONTROL [
        SESREQSCS OFFSET(0) NUMBITS(1) [],
        SESREQ OFFSET(1) NUMBITS(1) [],
        VBVALIDOVEN OFFSET(2) NUMBITS(1) [],
        VBVALIDOVVAL OFFSET(3) NUMBITS(1) [],
        AVALIDOVEN OFFSET(4) NUMBITS(1) [],
        AVALIDOVVAL OFFSET(5) NUMBITS(1) [],
        BVALIDOVEN OFFSET(6) NUMBITS(1) [],
        BVALIDOVVAL OFFSET(7) NUMBITS(1) [],
        HSTNEGCSC OFFSET(8) NUMBITS(1) [],
        HNPREQ OFFSET(9) NUMBITS(1) [],
        HOST_SET_HNP_ENABLE OFFSET(10) NUMBITS(1) [],
        DEVHNPEN OFFSET(11) NUMBITS(1) [],
        CONIDSTS OFFSET(16) NUMBITS(1) [],
        DBNCTIME OFFSET(17) NUMBITS(1) [],
        A_SESSION_VALID OFFSET(18) NUMBITS(1) [],
        B_SESSION_VALID OFFSET(19) NUMBITS(1) [],
        OTG_VERSION OFFSET(20) NUMBITS(1) [],
        MULTVALIDBC OFFSET(22) NUMBITS(5) [],
        CHRIPEN OFFSET(27) NUMBITS(1) []
    ],

    CORE_OTG_INTERRUPT [
        SESSION_END_DETECTED OFFSET(2) NUMBITS(1) [],
        SESSION_REQUEST_SUCCESS_STATUS_CHANGE OFFSET(8) NUMBITS(1) [],
        HOST_NEGOTIATION_SUCCESS_STATUS_CHANGE OFFSET(9) NUMBITS(1) [],
        HOST_NEGOTIATION_DETECTED OFFSET(17) NUMBITS(1) [],
        A_DEVICE_TIMEOUT_CHANGE OFFSET(18) NUMBITS(1) [],
        DEBOUNCE_DONE OFFSET(19) NUMBITS(1) []
    ],

    CORE_AHB [
        INTERRUPT_ENABLE OFFSET(0) NUMBITS(1) [],
        AXI_BURST_LENGTH OFFSET(1) NUMBITS(2) [
            Length4 = 0b00,
            Length3 = 0b01,
            Length2 = 0b10,
            Length1 = 0b11
        ],
        WAIT_FOR_AXI_WRITES OFFSET(4) NUMBITS(1) [],
        DMA_ENABLE OFFSET(5) NUMBITS(1) [],
        TRANSFER_EMPTY_LEVEL OFFSET(7) NUMBITS(1) [
            Empty = 1,
            Half = 0
        ],
        PERIODIC_TRANSFER_EMPTY_LEVEL OFFSET(8) NUMBITS(1) [
            Empty = 1,
            Half = 0
        ],
        REMMEMSUPP OFFSET(21) NUMBITS(1) [],
        NOTIALLDMAWRIT OFFSET(22) NUMBITS(1) [],
        DMA_REMAINDER_MODE OFFSET(23) NUMBITS(1) [
            Incremental = 0,
            Single = 1
        ]
    ],

    USB_CONTROL [
        TOUTCAL OFFSET(0) NUMBITS(3) [],
        PHY_INTERFACE OFFSET(3) NUMBITS(1) [],
        MODE_SELECT OFFSET(4) NUMBITS(1) [
            ULPI = 0,
            UTMI = 1
        ],
        FSINTF OFFSET(5) NUMBITS(1) [],
        PHYSEL OFFSET(6) NUMBITS(1) [],
        DDRSEL OFFSET(7) NUMBITS(1) [],
        SRP_CAPABLE OFFSET(8) NUMBITS(1) [],
        HNP_CAPABLE OFFSET(9) NUMBITS(1) [],
        USBTRDTIM OFFSET(10) NUMBITS(4) [],
        PHY_LPM_CLK_SEL OFFSET(15) NUMBITS(1) [],
        OTGUTMIFSSEL OFFSET(16) NUMBITS(1) [],
        ULPI_FSLS OFFSET(17) NUMBITS(1) [],
        ULPI_AUTO_RES OFFSET(18) NUMBITS(1) [],
        ULPI_CLK_SUS_M OFFSET(19) NUMBITS(1) [],
        ULPI_DRIVE_EXTERNAL_VBUS OFFSET(20) NUMBITS(1) [],
        ULPI_INT_VBUS_INDICATOR OFFSET(21) NUMBITS(1) [],
        TS_DLINE_PULSE_EANBLE OFFSET(22) NUMBITS(1) [],
        INDICATOR_COMPLEMENT OFFSET(23) NUMBITS(1) [],
        INDICATOR_PASS_THROUGH OFFSET(24) NUMBITS(1) [],
        ULPI_INT_PROT_DIS OFFSET(25) NUMBITS(1) [],
        IC_USB_CAPABLE OFFSET(26) NUMBITS(1) [],
        IC_TRAFFIC_PULL_REMOVE OFFSET(27) NUMBITS(1) [],
        TX_END_DELAY OFFSET(28) NUMBITS(1) [],
        FORCE_HOST_MODE OFFSET(29) NUMBITS(1) [],
        FORCE_DEV_MODE OFFSET(30) NUMBITS(1) []
    ],

    CORE_RESET [
        CORE_SOFT OFFSET(0) NUMBITS(1) [],
        HCLK_SOFT OFFSET(1) NUMBITS(1) [],
        HOST_FRAME_COUNTER OFFSET(2) NUMBITS(1) [],
        IN_TOKEN_QUEUE_FLUSH OFFSET(3) NUMBITS(1) [],
        RECEIVE_FIFO_FLUSH OFFSET(4) NUMBITS(1) [],
        TRANSMIT_FIFO_FLUSH OFFSET(5) NUMBITS(1) [],
        TRANSMIT_FIFO_FLUSH_NUMBER OFFSET(6) NUMBITS(5) [],
        DMA_REQUEST_SIGNAL OFFSET(30) NUMBITS(1) [],
        AHB_MASTER_IDLE OFFSET(31) NUMBITS(1) []
    ],

    CORE_INTERRUPTS [
        CURRENT_MODE OFFSET(0) NUMBITS(1) [],
        MODE_MISMATCH OFFSET(1) NUMBITS(1) [],
        OTG OFFSET(2) NUMBITS(1) [],
        DMA_START_OF_FRAME OFFSET(3) NUMBITS(1) [],
        RECEIVE_STATUS_LEVEL OFFSET(4) NUMBITS(1) [],
        NP_TRANSMIT_FIFO_EMPTY OFFSET(5) NUMBITS(1) [],
        GINNAKEFF OFFSET(6) NUMBITS(1) [],
        GOUTNAKEFF OFFSET(7) NUMBITS(1) [],
        ULPICK OFFSET(8) NUMBITS(1) [],
        I2C OFFSET(9) NUMBITS(1) [],
        EARLY_SUSPEND OFFSET(10) NUMBITS(1) [],
        USB_SUSPEND OFFSET(11) NUMBITS(1) [],
        USB_RESET OFFSET(12) NUMBITS(1) [],
        ENUMERATION_DONE OFFSET(13) NUMBITS(1) [],
        ISOCHRONOUS_OUT_DROP OFFSET(14) NUMBITS(1) [],
        EOPFRAME OFFSET(15) NUMBITS(1) [],
        RESTORE_DONE OFFSET(16) NUMBITS(1) [],
        END_POINT_MISMATCH OFFSET(17) NUMBITS(1) [],
        IN_END_POINT OFFSET(18) NUMBITS(1) [],
        OUT_END_POINT OFFSET(19) NUMBITS(1) [],
        INCOMPLETE_ISOCHRONOUS_IN OFFSET(20) NUMBITS(1) [],
        INCOMPLETE_ISOCHRONOUS_OUT OFFSET(21) NUMBITS(1) [],
        FETSETUP OFFSET(22) NUMBITS(1) [],
        RESET_DETECT OFFSET(23) NUMBITS(1) [],
        PORT OFFSET(24) NUMBITS(1) [],
        HOST_CHANNEL OFFSET(25) NUMBITS(1) [],
        HP_TRANSMIT_FIFO_EMPTY OFFSET(26) NUMBITS(1) [],
        LOW_POWER_MODE_TRANSMIT_RECEIVED OFFSET(27) NUMBITS(1) [],
        CONNECTION_ID_STATUS_CHANGE OFFSET(28) NUMBITS(1) [],
        DISCONNECT OFFSET(29) NUMBITS(1) [],
        SESSION_REQUEST OFFSET(30) NUMBITS(1) [],
        WAKEUP OFFSET(31) NUMBITS(1) []
    ],

    NON_PERIODIC_FIFO_STATUS [
        SPACE_AVAILABLE OFFSET(0) NUMBITS(16) [],
        QUEUE_SPACE_AVAILABLE OFFSET(16) NUMBITS(8) [],
        TERMINATE OFFSET(24) NUMBITS(1) [],
        TOKEN_TYPE OFFSET(25) NUMBITS(2) [
            InOut = 0,
            ZeroLengthOut = 1,
            PingCompleteSplit = 2,
            ChannelHalt = 3
        ],
        CHANNEL OFFSET(27) NUMBITS(4) [],
        ODD OFFSET(31) NUMBITS(1) []
    ],

    //TODO This is a hack to get a u128 with field access for this structure
    // it will work for now, but it isn't pretty
    CORE_HARDWARE0 [
        DIRECTION0 OFFSET(0) NUMBITS(2) [],
        DIRECTION1 OFFSET(2) NUMBITS(2) [],
        DIRECTION2 OFFSET(4) NUMBITS(2) [],
        DIRECTION3 OFFSET(6) NUMBITS(2) [],
        DIRECTION4 OFFSET(8) NUMBITS(2) [],
        DIRECTION5 OFFSET(10) NUMBITS(2) [],
        DIRECTION6 OFFSET(12) NUMBITS(2) [],
        DIRECTION7 OFFSET(14) NUMBITS(2) [],
        DIRECTION8 OFFSET(16) NUMBITS(2) [],
        DIRECTION9 OFFSET(18) NUMBITS(2) [],
        DIRECTION10 OFFSET(20) NUMBITS(2) [],
        DIRECTION11 OFFSET(22) NUMBITS(2) [],
        DIRECTION12 OFFSET(24) NUMBITS(2) [],
        DIRECTION13 OFFSET(26) NUMBITS(2) [],
        DIRECTION14 OFFSET(28) NUMBITS(2) [],
        DIRECTION15 OFFSET(30) NUMBITS(2) []
    ],
    CORE_HARDWARE1 [
        OPERATING_MODE OFFSET(0) NUMBITS(3) [
            HNP_SRP_CAPABLE = 0,
            SRP_ONLY_CAPABLE = 1,
            NO_HNP_SRP_CAPABLE = 2,
            SRP_CAPABLE_DEVICE = 3,
            NO_SRP_CAPABLE_DEVICE = 4,
            SRP_CAPABLE_HOST = 5,
            NO_SRP_CAPABLE_HOST = 6
        ],
        ARCHITECTURE OFFSET(3) NUMBITS(2) [
            SlaveOnly = 0,
            ExternalDma = 1,
            InternalDma = 2
        ],
        POINTTO_POINT OFFSET(5) NUMBITS(1) [],
        HIGH_SPEED_PHYSICAL OFFSET(6) NUMBITS(2) [
            NotSupported = 0,
            Utmi = 1,
            Ulpi = 2,
            UtmiUlpi = 3
        ],
        FULL_SPEED_PHYSICAL OFFSET(8) NUMBITS(2) [
            Physical0 = 0,
            Dedicated = 1,
            Physical2 = 2,
            Physical3 = 3
        ],
        DEVICE_END_POINT_COUNT OFFSET(10) NUMBITS(4) [],
        HOST_CHANNEL_COUNT OFFSET(14) NUMBITS(4) [],
        SUPPORTS_PERIODIC_ENDPOINTS OFFSET(18) NUMBITS(1) [],
        DYNAMIC_FIFO OFFSET(19) NUMBITS(1) [],
        MULTI_PROC_INT OFFSET(20) NUMBITS(1) [],
        NON_PERIODIC_QUEUE_DEPTH OFFSET(22) NUMBITS(2) [],
        HOST_PERIODIC_QUEUE_DEPTH OFFSET(24) NUMBITS(2) [],
        DEVICE_TOKEN_QUEUE_DEPTH OFFSET(26) NUMBITS(5) [],
        ENABLE_IC_USB OFFSET(31) NUMBITS(1) []
    ],
    CORE_HARDWARE2 [
        TRANSFER_SIZE_CONTROL_WIDTH OFFSET(0) NUMBITS(4) [],
        PACKET_SIZE_CONTROL_WIDTH OFFSET(4) NUMBITS(3) [],
        OTG_FUNC OFFSET(7) NUMBITS(1) [],
        I2C OFFSET(8) NUMBITS(1) [],
        VENDOR_CONTROL_INTERFACE OFFSET(9) NUMBITS(1) [],
        OPTIONAL_FEATURES OFFSET(10) NUMBITS(1) [],
        SYNCHRONOUS_RESET_TYPE OFFSET(11) NUMBITS(1) [],
        ADP_SUPPORT OFFSET(12) NUMBITS(1) [],
        OTG_ENABLE_HSIC OFFSET(13) NUMBITS(1) [],
        BC_SUPPORT OFFSET(14) NUMBITS(1) [],
        LOW_POWER_MODE_ENABLED OFFSET(15) NUMBITS(1) [],
        FIFO_DEPTH OFFSET(16) NUMBITS(16) []
    ],
    CORE_HARDWARE3 [
        PERIODIC_IN_ENDPOINT_COUNT OFFSET(0) NUMBITS(4) [],
        POWER_OPTIMISATION OFFSET(4) NUMBITS(1) [],
        MINIMUM_AHB_FREQUENCY OFFSET(5) NUMBITS(1) [],
        PARTIAL_POWER_OFF OFFSET(6) NUMBITS(1) [],
        UTMI_PHYSICAL_DATA_WIDTH OFFSET(14) NUMBITS(2) [
            Width8bit = 0,
            Width16bit = 1,
            Width8or16bit = 2
        ],
        MODE_CONTROL_ENDPOINT_COUNT OFFSET(16) NUMBITS(4) [],
        VALID_FILTER_IDDIG_ENABLED OFFSET(20) NUMBITS(1) [],
        VBUS_VALID_FILTER_ENABLED OFFSET(21) NUMBITS(1) [],
        VALID_FILTER_A_ENABLED OFFSET(22) NUMBITS(1) [],
        VALID_FILTER_B_ENABLED OFFSET(23) NUMBITS(1) [],
        SESSION_END_FILTER_ENABLED OFFSET(24) NUMBITS(1) [],
        DED_FIFO_EN OFFSET(25) NUMBITS(1) [],
        IN_ENDPOINT_COUNT OFFSET(26) NUMBITS(4) [],
        DMA_DESCRIPTION OFFSET(30) NUMBITS(1) [],
        DMA_DYNAMIC_DESCRIPTION OFFSET(31) NUMBITS(1) []
    ],

    HOST_CONFIG [
        CLOCK_RATE OFFSET(0) NUMBITS(2) [],
        FSLS_ONLY OFFSET(2) NUMBITS(1) [],
        EN_32KHZ_SUSP OFFSET(7) NUMBITS(1) [],
        RES_VAL_PERIOD OFFSET(8) NUMBITS(8) [],
        ENABLE_DMA_DESCRIPTOR OFFSET(23) NUMBITS(1) [],
        FRAME_LIST_ENTRIES OFFSET(24) NUMBITS(2) [],
        PERIODIC_SCHEDULE_ENABLE OFFSET(26) NUMBITS(1) [],
        PERIODIC_SCHEDULE_STATUS OFFSET(27) NUMBITS(1) [],
        RESERVED28_30 OFFSET(28) NUMBITS(3) [],
        MODE_CHG_TIME OFFSET(31) NUMBITS(1) []
    ],

    HOST_FRAME_INTERVAL [
        INTERVAL OFFSET(0) NUMBITS(16) [],
        DYNAMIC_FRAME_RELOAD OFFSET(16) NUMBITS(1) []
    ],

    HOST_FRAME_CONTROL [
        FRAME_NUMBER OFFSET(0) NUMBITS(16) [],
        FRAME_REMAINING OFFSET(16) NUMBITS(16) []
    ],

    HOST_FIFO_STATUS [
        SPACE_AVAILABLE OFFSET(0) NUMBITS(16) [],
        QUEUE_SPACE_AVAILABLE OFFSET(16) NUMBITS(8) [],
        TERMINATE OFFSET(24) NUMBITS(1) [],
        TOKEN_TYPE OFFSET(25) NUMBITS(2) [
            ZeroLength = 0,
            Ping = 1,
            Disable = 2
        ],
        CHANNEL OFFSET(27) NUMBITS(4) [],
        ODD OFFSET(31) NUMBITS(1) []
    ],

    HOST_PORT [
        CONNECT OFFSET(0) NUMBITS(1) [],
        CONNECT_CHANGED OFFSET(1) NUMBITS(1) [],
        ENABLE OFFSET(2) NUMBITS(1) [],
        ENABLE_CHANGED OFFSET(3) NUMBITS(1) [],
        OVER_CURRENT OFFSET(4) NUMBITS(1) [],
        OVER_CURRENT_CHANGED OFFSET(5) NUMBITS(1) [],
        RESUME OFFSET(6) NUMBITS(1) [],
        SUSPEND OFFSET(7) NUMBITS(1) [],
        RESET OFFSET(8) NUMBITS(1) [],
        PORT_LINE_STATUS OFFSET(10) NUMBITS(2) [],
        POWER OFFSET(12) NUMBITS(1) [],
        TEST_CONTROL OFFSET(13) NUMBITS(4) [],
        SPEED OFFSET(17) NUMBITS(2) [
            USB_SPEED_HIGH = 0,
            USB_SPEED_FULL = 1,
            USB_SPEED_LOW = 2
        ]
    ],

    HOST_CHANNEL_CHARACTERISTICS [
        MAX_PACKET_SIZE OFFSET(0) NUMBITS(11) [],
        ENDPOINT_NUMBER OFFSET(11) NUMBITS(4) [],
        ENDPOINT_DIRECTION OFFSET(15) NUMBITS(1) [
            In = 0,
            Out = 1
        ],
        LOW_SPEED OFFSET(17) NUMBITS(1) [
            True = 1,
            False = 0
        ],
        ENDPOINT_TYPE OFFSET(18) NUMBITS(2) [],
        PACKETS_PER_FRAME OFFSET(20) NUMBITS(2) [],
        DEVICE_ADDRESS OFFSET(22) NUMBITS(7) [],
        ODD_FRAME OFFSET(29) NUMBITS(1) [],
        CHANNEL_DISABLE OFFSET(30) NUMBITS(1) [
            True = 1,
            False = 0
        ],
        CHANNEL_ENABLE OFFSET(31) NUMBITS(1) [
            True = 1,
            False = 0
        ]
    ],

    HOST_CHANNEL_SPLIT_CONTROL [
        PORT_ADDRESS OFFSET(0) NUMBITS(7) [],
        HUB_ADDRESS OFFSET(7) NUMBITS(7) [],
        TRANSACTION_POSITION OFFSET(14) NUMBITS(2) [
            Begin = 2,
            End = 1,
            Middle = 0,
            All = 3
        ],
        COMPLETE_SPLIT OFFSET(16) NUMBITS(1) [
            Split = 1,
            Normal = 0
        ],
        SPLIT_ENABLE OFFSET(31) NUMBITS(1) [
            True = 1,
            False = 0
        ]
    ],

    HOST_TRANSFER_SIZE [
        SIZE OFFSET(0) NUMBITS(19) [],
        PACKET_COUNT OFFSET(19) NUMBITS(10) [],
        PACKET_ID OFFSET(29) NUMBITS(2) [
            USB_PID_DATA0 = 0,
            USB_PID_DATA1 = 2,
            USB_PID_DATA2 = 1,
            USB_PID_SETUP = 3
            // USB_MDATA = 3 TODO Why is this defined twice
        ],
        DO_PING OFFSET(31) NUMBITS(1) []
    ],

    POWER_REG [
        STOP_P_CLOCK OFFSET(0) NUMBITS(1) [],
        GATE_H_CLOCK OFFSET(1) NUMBITS(1) [],
        POWER_CLAMP OFFSET(2) NUMBITS(1) [],
        POWER_DOWN_MODULES OFFSET(3) NUMBITS(1) [],
        PHY_SUSPENDED OFFSET(4) NUMBITS(1) [],
        ENABLE_SLEEP_CLOCK_GATING OFFSET(5) NUMBITS(1) [],
        PHY_SLEEPING OFFSET(6) NUMBITS(1) [],
        DEEP_SLEEP OFFSET(7) NUMBITS(1) []
    ],

    USB_SEND_CONTROL [
        SPLIT_TRIES OFFSET(0) NUMBITS(8) [],
        PACKET_TRIES OFFSET(8) NUMBITS(8) [],
        GLOBAL_TRIES OFFSET(16) NUMBITS(8) [],
        RESERVED OFFSET(24) NUMBITS(3) [],
        LONGER_DELAY OFFSET(27) NUMBITS(1) [],
        ACTION_RESEND_SPLIT OFFSET(28) NUMBITS(1) [],
        ACTION_RETRY OFFSET(29) NUMBITS(1) [],
        ACTION_FATAL_ERROR OFFSET(30) NUMBITS(1) [],
        SUCCESS OFFSET(31) NUMBITS(1) []
    ]
}

enum CoreFifoFlush {
    FlushNonPeriodic = 0,
    FlushPeriodic1 = 1,
    FlushPeriodic2 = 2,
    FlushPeriodic3 = 3,
    FlushPeriodic4 = 4,
    FlushPeriodic5 = 5,
    FlushPeriodic6 = 6,
    FlushPeriodic7 = 7,
    FlushPeriodic8 = 8,
    FlushPeriodic9 = 9,
    FlushPeriodic10 = 10,
    FlushPeriodic11 = 11,
    FlushPeriodic12 = 12,
    FlushPeriodic13 = 13,
    FlushPeriodic14 = 14,
    FlushPeriodic15 = 15,
    FlushAll = 16,
}

#[allow(non_snake_case)]
#[repr(C)]
#[repr(align(4))]
struct CoreNonPeriodicInfo {
    Size: ReadWrite<u32, FIFO_SIZE::Register>,
    Status: ReadOnly<u32, NON_PERIODIC_FIFO_STATUS::Register>,
}

#[allow(non_snake_case)]
#[repr(C)]
#[repr(align(4))]
struct CorePeriodicInfo {
    HostSize: ReadWrite<u32, FIFO_SIZE::Register>,
    DataSize: [ReadWrite<u32, FIFO_SIZE::Register>; 15],
}

#[allow(non_snake_case)]
#[repr(C)]
#[repr(align(4))]
struct HostChannel {
    Characteristics: ReadWrite<u32, HOST_CHANNEL_CHARACTERISTICS::Register>,
    SplitCtrl: ReadWrite<u32, HOST_CHANNEL_SPLIT_CONTROL::Register>,
    Interrupt: ReadWrite<u32, CHANNEL_INTERRUPTS::Register>,
    InterruptMask: ReadWrite<u32, CHANNEL_INTERRUPTS::Register>,
    TransferSize: ReadWrite<u32, HOST_TRANSFER_SIZE::Register>,
    DmaAddr: ReadWrite<u32>,
    __reserved18: u32,
    __reserved1c: u32,
}

enum ClockRate {
    Clock30_60MHz = 0,
    Clock48MHz = 1,
    Clock6MHz = 2,
}

#[allow(non_snake_case)]
#[repr(C)]
struct RegisterBlock {
    DwcCoreOtgControl: ReadWrite<u32, CORE_OTG_CONTROL::Register>, // 0x00
    DwcCoreOtgInterrupt: ReadWrite<u32, CORE_OTG_INTERRUPT::Register>, // 0x04
    DwcCoreAhb: ReadWrite<u32, CORE_AHB::Register>,                // 0x08
    DwcCoreControl: ReadWrite<u32, USB_CONTROL::Register>,         // 0x0C
    DwcCoreReset: ReadWrite<u32, CORE_RESET::Register>,            // 0x10
    DwcCoreInterrupt: ReadWrite<u32, CORE_INTERRUPTS::Register>,   // 0x14
    DwcCoreInterruptMask: ReadWrite<u32, CORE_INTERRUPTS::Register>, // 0x18
    __reserved0: [u32; 2],                                         // 0x1c, 0x20
    DwcCoreReceivesize: ReadWrite<u32>,                            // 0x24
    DwcCoreNonPeriodicInfo: CoreNonPeriodicInfo,                   // 0x28, 0x2c
    __reserved1: [u32; 3],                                         // 0x30, 0x34, 0x38
    DwcCoreUserID: ReadWrite<u32>,                                 // 0x3c
    DwcCoreVendorID: ReadWrite<u32>,                               // 0x40
    DwcCoreHardware0: ReadWrite<u32, CORE_HARDWARE0::Register>,    // 0x44
    DwcCoreHardware1: ReadWrite<u32, CORE_HARDWARE1::Register>,    // 0x48
    DwcCoreHardware2: ReadWrite<u32, CORE_HARDWARE2::Register>,    // 0x4C
    DwcCoreHardware3: ReadWrite<u32, CORE_HARDWARE3::Register>,    // 0x50
    __reserved2: [u32; 43],                                        // 0x54
    DwcCorePeriodicInfo: CorePeriodicInfo,                         // 0x100
    __reserved3: [u32; 176],
    DwcHostConfig: ReadWrite<u32, HOST_CONFIG::Register>, // 0x400
    DwcHostFrameInterval: ReadWrite<u32, HOST_FRAME_INTERVAL::Register>, // 0x404
    DwcHostFrameControl: ReadWrite<u32, HOST_FRAME_CONTROL::Register>, // 0x408
    __reserved4: u32,
    DwcHostFifoStatus: ReadWrite<u32, HOST_FIFO_STATUS::Register>, // 0x410
    DwcHostInterrupt: ReadWrite<u32>,                              // 0x414
    DwcHostInterruptMask: ReadWrite<u32>,                          // 0x418
    DwcHostFrameList: ReadWrite<u32>,                              // 0x41C
    __reserved5: [u32; 8],                                         // 0x420
    DwcHostPort: ReadWrite<u32, HOST_PORT::Register>,              // 0x440
    __reserved6: [u32; 47],
    DwcHostChannel: HostChannel,
    __reserved7: [u32; 568],
    DwcPowerAndClock: ReadWrite<u32, POWER_REG::Register>, // 0xE00
}

struct USBInner {
    base_addr: usize,
}

impl ops::Deref for USBInner {
    type Target = RegisterBlock;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl USBInner {
    const fn new(base_addr: usize) -> USBInner {
        USBInner { base_addr }
    }

    fn ptr(&self) -> *const RegisterBlock {
        self.base_addr as *const _
    }
}

////////////////////////////////////////////////////////////////////////////////
// OS interface implementations
////////////////////////////////////////////////////////////////////////////////

pub struct USB {
    inner: NullLock<USBInner>,
}

impl USB {
    pub const fn new(base_addr: usize) -> USB {
        USB {
            inner: NullLock::new(USBInner::new(base_addr)),
        }
    }
}

impl interface::driver::DeviceDriver for USB {
    fn compatible(&self) -> &str {
        "DWC USB 2.0 OTG Driver"
    }

    fn init(&self) -> interface::driver::Result {
        // TODO
        Ok(())
    }
}
