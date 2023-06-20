//! Universal Asynchronous Receiver/Transmitter.
use crate::{
    clocks::Clocks,
    glb::{self, Function, UartSignal},
    gpio::{Alternate, Pin},
    GLB, UART,
};
use base_address::BaseAddress;
use core::{cell::UnsafeCell, marker::PhantomData};
use embedded_time::rate::Baud;

/// UART alternate (type state).
pub struct Uart;

impl Alternate for Uart {
    const F: Function = Function::Uart;
}

/// Universal Asynchoronous Receiver/Transmitter registers.
#[repr(C)]
pub struct RegisterBlock {
    /// Transmit configuration.
    pub transmit_config: TRANSMIT_CONFIG,
    _reserved1: [u8; 0x04],
    /// Bit period in clocks.
    pub bit_period: BIT_PERIOD,
    /// Data format configuration.
    pub data_config: DATA_CONFIG,
    _reserved2: [u8; 0x20],
    /// Bus state.
    pub bus_state: BUS_STATE,
    _reserved3: [u8; 0x50],
    /// First-in first-out queue configuration 1.
    pub fifo_config_1: FIFO_CONFIG_1,
    /// Write data into first-in first-out queue.
    pub data_write: DATA_WRITE,
}

/// Transmit configuration register.
#[allow(non_camel_case_types)]
#[repr(transparent)]
pub struct TRANSMIT_CONFIG(UnsafeCell<u32>);

/// Configuration structure for transmit feature.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct TransmitConfig(u32);

impl TRANSMIT_CONFIG {
    /// Read transmit configuration.
    #[inline]
    pub fn read(&self) -> TransmitConfig {
        TransmitConfig(unsafe { self.0.get().read_volatile() })
    }
    /// Write transmit configuration.
    #[inline]
    pub fn write(&self, val: TransmitConfig) {
        unsafe { self.0.get().write_volatile(val.0) }
    }
}

impl TransmitConfig {
    const ENABLE: u32 = 1 << 0;
    const CTS: u32 = 1 << 1;
    const FREERUN: u32 = 1 << 2;
    const LIN_TRANSMIT: u32 = 1 << 3;
    const PARITY_ENABLE: u32 = 1 << 4;
    const PARITY_MODE: u32 = 1 << 5;
    const IR_TRANSMIT: u32 = 1 << 6;
    const IR_INVERSE: u32 = 1 << 7;
    const WORD_LENGTH: u32 = 0b111 << 8;
    const STOP_BITS: u32 = 0b11 << 11;
    const LIN_BREAK_BITS: u32 = 0b111 << 13;
    const TRANSFER_LENGTH: u32 = 0xff << 16;

    /// Enable transmit.
    #[inline]
    pub const fn enable_txd(self) -> Self {
        Self(self.0 | Self::ENABLE)
    }
    /// Disable transmit.
    #[inline]
    pub const fn disable_txd(self) -> Self {
        Self(self.0 & !Self::ENABLE)
    }
    /// Check if transmit is enabled.
    #[inline]
    pub const fn is_txd_enabled(self) -> bool {
        self.0 & Self::ENABLE != 0
    }
    /// Enable Clear-to-Send signal.
    #[inline]
    pub const fn enable_cts(self) -> Self {
        Self(self.0 | Self::CTS)
    }
    /// Disable Clear-to-Send signal.
    #[inline]
    pub const fn disable_cts(self) -> Self {
        Self(self.0 & !Self::CTS)
    }
    /// Check if Clear-to-Send signal is enabled.
    #[inline]
    pub const fn is_cts_enabled(self) -> bool {
        self.0 & Self::CTS != 0
    }
    /// Enable free-run mode.
    #[inline]
    pub const fn enable_freerun(self) -> Self {
        Self(self.0 | Self::FREERUN)
    }
    /// Disable free-run mode.
    #[inline]
    pub const fn disable_freerun(self) -> Self {
        Self(self.0 & !Self::FREERUN)
    }
    /// Check if free-run mode is enabled.
    #[inline]
    pub const fn is_freerun_enabled(self) -> bool {
        self.0 & Self::FREERUN != 0
    }
    /// Enable LIN protocol transmission.
    #[inline]
    pub const fn enable_lin_transmit(self) -> Self {
        Self(self.0 | Self::LIN_TRANSMIT)
    }
    /// Disable LIN protocol transmission.
    #[inline]
    pub const fn disable_lin_transmit(self) -> Self {
        Self(self.0 & !Self::LIN_TRANSMIT)
    }
    /// Check if LIN protocol transmission is enabled.
    #[inline]
    pub const fn is_lin_transmit_enabled(self) -> bool {
        self.0 & Self::LIN_TRANSMIT != 0
    }
    /// Set parity check mode.
    #[inline]
    pub const fn set_parity(self, parity: Parity) -> Self {
        match parity {
            Parity::Even => Self((self.0 | Self::PARITY_ENABLE) & !Self::PARITY_MODE),
            Parity::Odd => Self(self.0 | Self::PARITY_ENABLE | Self::PARITY_MODE),
            Parity::None => Self(self.0 & !Self::PARITY_ENABLE),
        }
    }
    /// Get parity check mode.
    #[inline]
    pub const fn get_parity(self) -> Parity {
        if self.0 & Self::PARITY_ENABLE == 0 {
            Parity::None
        } else if self.0 & Self::PARITY_MODE == 0 {
            Parity::Even
        } else {
            Parity::Odd
        }
    }
    /// Enable IR transmission.
    #[inline]
    pub const fn enable_ir_transmit(self) -> Self {
        Self(self.0 | Self::IR_TRANSMIT)
    }
    /// Disable IR transmission.
    #[inline]
    pub const fn disable_ir_transmit(self) -> Self {
        Self(self.0 & !Self::IR_TRANSMIT)
    }
    /// Check if IR transmission is enabled.
    #[inline]
    pub const fn is_ir_transmit_enabled(self) -> bool {
        self.0 & Self::IR_TRANSMIT != 0
    }
    /// Invert transmit signal output in IR mode.
    #[inline]
    pub const fn enable_ir_inverse(self) -> Self {
        Self(self.0 | Self::IR_INVERSE)
    }
    /// Don't invert transmit signal output in IR mode.
    #[inline]
    pub const fn disable_ir_inverse(self) -> Self {
        Self(self.0 & !Self::IR_INVERSE)
    }
    /// Check if transmit signal output in IR mode is inverted.
    #[inline]
    pub const fn is_ir_inverse_enabled(self) -> bool {
        self.0 & Self::IR_INVERSE != 0
    }
    /// Set word length.
    #[inline]
    pub const fn set_word_length(self, val: WordLength) -> Self {
        let val = match val {
            WordLength::Five => 4,
            WordLength::Six => 5,
            WordLength::Seven => 6,
            WordLength::Eight => 7,
        };
        Self(self.0 & !Self::WORD_LENGTH | val << 8)
    }
    /// Get word length.
    #[inline]
    pub const fn word_length(self) -> WordLength {
        let val = (self.0 & Self::WORD_LENGTH) >> 8;
        match val {
            4 => WordLength::Five,
            5 => WordLength::Six,
            6 => WordLength::Seven,
            7 => WordLength::Eight,
            _ => unreachable!(),
        }
    }
    /// Set stop bit configuration.
    #[inline]
    pub const fn set_stop_bits(self, val: StopBits) -> Self {
        let val = match val {
            StopBits::ZeroPointFive => 0,
            StopBits::One => 1,
            StopBits::OnePointFive => 2,
            StopBits::Two => 3,
        };
        Self(self.0 & !Self::STOP_BITS | val << 11)
    }
    /// Get stop bit configuration.
    #[inline]
    pub const fn stop_bits(self) -> StopBits {
        let val = (self.0 & Self::STOP_BITS) >> 11;
        match val {
            0 => StopBits::ZeroPointFive,
            1 => StopBits::One,
            2 => StopBits::OnePointFive,
            3 => StopBits::Two,
            _ => unreachable!(),
        }
    }
    /// Set synchronize interval under LIN mode.
    ///
    /// # Parameters
    ///
    /// * `bits` - Interval in bits, the value should be 0 ~ 7.
    #[inline]
    pub const fn set_lin_break_bits(self, bits: u8) -> Self {
        Self(self.0 & !Self::LIN_BREAK_BITS | (bits as u32) << 13)
    }
    /// Get synchronize interval under LIN mode.
    ///
    /// Return value is 0 ~ 7, represent in bits.
    #[inline]
    pub const fn lin_break_bits(self) -> u8 {
        ((self.0 & Self::LIN_BREAK_BITS) >> 13) as u8
    }
    /// Trigger interrupt when specified length of data is send.
    ///
    /// NOTE: This bit is not valid when it is running under free-run mode.
    #[inline]
    pub const fn set_transfer_length(self, length: u16) -> Self {
        Self(self.0 & !Self::TRANSFER_LENGTH | (length as u32) << 16)
    }
    /// Get the length of data that triggers the interrupt.
    #[inline]
    pub const fn transfer_length(self) -> u16 {
        ((self.0 & Self::TRANSFER_LENGTH) >> 16) as u16
    }
}

/// Bit period configuration register.
#[allow(non_camel_case_types)]
#[repr(transparent)]
pub struct BIT_PERIOD(UnsafeCell<u32>);

impl BIT_PERIOD {
    /// Read data configuration.
    #[inline]
    pub fn read(&self) -> BitPeriod {
        BitPeriod(unsafe { self.0.get().read_volatile() })
    }
    /// Write data configuration.
    #[inline]
    pub fn write(&self, val: BitPeriod) {
        unsafe { self.0.get().write_volatile(val.0) }
    }
}
/// Configuration structure for bit period.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct BitPeriod(u32);

impl BitPeriod {
    const TRANSMIT: u32 = 0xff;
    const RECEIVE: u32 = 0xff << 16;

    /// Set transmit time interval.
    #[inline]
    pub const fn set_transmit_time_interval(self, val: u16) -> Self {
        Self(self.0 & !Self::TRANSMIT | val as u32)
    }
    /// Get transmit time interval.
    #[inline]
    pub const fn transmit_time_interval(self) -> u16 {
        (self.0 & Self::TRANSMIT) as u16
    }
    /// Set receive time interval.
    #[inline]
    pub const fn set_receive_time_interval(self, val: u16) -> Self {
        Self(self.0 & !Self::RECEIVE | ((val as u32) << 16))
    }
    /// Get receive time interval.
    #[inline]
    pub const fn receive_time_interval(self) -> u16 {
        ((self.0 & Self::RECEIVE) >> 16) as u16
    }
}

/// Data configuration register.
#[allow(non_camel_case_types)]
#[repr(transparent)]
pub struct DATA_CONFIG(UnsafeCell<u32>);

/// Configuration structure for data format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct DataConfig(u32);

impl DATA_CONFIG {
    /// Read data configuration.
    #[inline]
    pub fn read(&self) -> DataConfig {
        DataConfig(unsafe { self.0.get().read_volatile() })
    }
    /// Write data configuration.
    #[inline]
    pub fn write(&self, val: DataConfig) {
        unsafe { self.0.get().write_volatile(val.0) }
    }
}

impl DataConfig {
    const BIT_ORDER: u32 = 1 << 0;

    /// Set the bit order in each data word.
    #[inline]
    pub const fn set_bit_order(self, val: BitOrder) -> Self {
        match val {
            BitOrder::LsbFirst => Self(self.0 & !Self::BIT_ORDER),
            BitOrder::MsbFirst => Self(self.0 | Self::BIT_ORDER),
        }
    }
    /// Get the bit order in each data word.
    #[inline]
    pub const fn bit_order(self) -> BitOrder {
        if self.0 & Self::BIT_ORDER == 0 {
            BitOrder::LsbFirst
        } else {
            BitOrder::MsbFirst
        }
    }
}

/// Bus state register.
#[allow(non_camel_case_types)]
#[repr(transparent)]
pub struct BUS_STATE(UnsafeCell<u32>);

/// Configuration structure for bus state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct BusState(u32);

impl BUS_STATE {
    /// Read bus state.
    #[inline]
    pub fn read(&self) -> BusState {
        BusState(unsafe { self.0.get().read_volatile() })
    }
    /// Write bus state.
    #[inline]
    pub fn write(&self, val: BusState) {
        unsafe { self.0.get().write_volatile(val.0) }
    }
}

impl BusState {
    const TRANSMIT_BUSY: u32 = 1 << 0;
    const RECEIVE_BUSY: u32 = 1 << 1;

    /// Get if UART transmit bus is busy.
    #[inline]
    pub const fn transmit_busy(self) -> bool {
        self.0 & Self::TRANSMIT_BUSY != 0
    }
    /// Get if UART receive bus is busy.
    #[inline]
    pub const fn receive_busy(self) -> bool {
        self.0 & Self::RECEIVE_BUSY != 0
    }
}

/// FIFO configuration register 1.
#[allow(non_camel_case_types)]
#[repr(transparent)]
pub struct FIFO_CONFIG_1(UnsafeCell<u32>);

/// Configuration structure for FIFO configuration 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct FifoConfig1(u32);

impl FIFO_CONFIG_1 {
    /// Read FIFO config.
    #[inline]
    pub fn read(&self) -> FifoConfig1 {
        FifoConfig1(unsafe { self.0.get().read_volatile() })
    }
    /// Write FIFO config.
    #[inline]
    pub fn write(&self, val: FifoConfig1) {
        unsafe { self.0.get().write_volatile(val.0) }
    }
}

impl FifoConfig1 {
    const TRANSMIT_COUNT: u32 = 0x3f;
    const RECEIVE_COUNT: u32 = 0x3f << 8;
    const TRANSMIT_THRESHOLD: u32 = 0x1f << 16;
    const RECEIVE_THRESHOLD: u32 = 0x1f << 24;

    /// Get count of available data in transmit FIFO.
    #[inline]
    pub const fn transmit_count(self) -> u8 {
        (self.0 & Self::TRANSMIT_COUNT) as u8
    }
    /// Get count of available data in receive FIFO.
    #[inline]
    pub const fn receive_count(self) -> u8 {
        ((self.0 & Self::RECEIVE_COUNT) >> 8) as u8
    }
    /// Set transmit FIFO threshold.
    #[inline]
    pub const fn set_transmit_threshold(self, val: u8) -> Self {
        Self(self.0 & !Self::TRANSMIT_THRESHOLD | ((val as u32) << 16))
    }
    /// Get transmit FIFO threshold.
    #[inline]
    pub const fn transmit_threshold(self) -> u8 {
        ((self.0 & Self::TRANSMIT_THRESHOLD) >> 16) as u8
    }
    /// Set receive FIFO threshold.
    #[inline]
    pub const fn set_receive_threshold(self, val: u8) -> Self {
        Self(self.0 & !Self::RECEIVE_THRESHOLD | ((val as u32) << 24))
    }
    /// Get receive FIFO threshold.
    #[inline]
    pub const fn receive_threshold(self) -> u8 {
        ((self.0 & Self::RECEIVE_THRESHOLD) >> 24) as u8
    }
}

/// Multiplex to Request-to-Send (type state).
pub struct MuxRts<const U: usize>;

/// Multiplex to Clear-to-Send (type state).
pub struct MuxCts<const U: usize>;

/// Multiplex to Transmit (type state).
pub struct MuxTxd<const U: usize>;

/// Multiplex to Receive (type state).
pub struct MuxRxd<const U: usize>;

impl<const U: usize> MuxRts<U> {
    #[inline]
    fn to_signal() -> UartSignal {
        match U {
            0 => UartSignal::Rts0,
            1 => UartSignal::Rts1,
            2 => UartSignal::Rts2,
            _ => unreachable!(),
        }
    }
}

impl<const U: usize> MuxCts<U> {
    #[inline]
    fn to_signal() -> UartSignal {
        match U {
            0 => UartSignal::Cts0,
            1 => UartSignal::Cts1,
            2 => UartSignal::Cts2,
            _ => unreachable!(),
        }
    }
}

impl<const U: usize> MuxTxd<U> {
    #[inline]
    fn to_signal() -> UartSignal {
        match U {
            0 => UartSignal::Txd0,
            1 => UartSignal::Txd1,
            2 => UartSignal::Txd2,
            _ => unreachable!(),
        }
    }
}

impl<const U: usize> MuxRxd<U> {
    #[inline]
    fn to_signal() -> UartSignal {
        match U {
            0 => UartSignal::Rxd0,
            1 => UartSignal::Rxd1,
            2 => UartSignal::Rxd2,
            _ => unreachable!(),
        }
    }
}

/// Global peripheral UART signal multiplexer.
pub struct UartMux<A: BaseAddress, const I: usize, M> {
    base: GLB<A>,
    _mode: PhantomData<M>,
}

impl<A: BaseAddress, const I: usize, M> UartMux<A, I, M> {
    /// Configure the internal UART signal to Request-to-Send (RTS).
    #[inline]
    pub fn into_request_to_send<const U: usize>(self) -> UartMux<A, I, MuxRts<U>> {
        let config = self.base.uart_mux_group[I >> 3]
            .read()
            .set_signal(I & 0x7, MuxRts::<U>::to_signal());
        self.base.uart_mux_group[I >> 3].write(config);
        UartMux {
            base: self.base,
            _mode: PhantomData,
        }
    }
    /// Configure the internal UART signal to Transmit (TXD).
    #[inline]
    pub fn into_transmit<const U: usize>(self) -> UartMux<A, I, MuxTxd<U>> {
        let config = self.base.uart_mux_group[I >> 3]
            .read()
            .set_signal(I & 0x7, MuxTxd::<U>::to_signal());
        self.base.uart_mux_group[I >> 3].write(config);
        UartMux {
            base: self.base,
            _mode: PhantomData,
        }
    }
    /// Configure the internal UART signal to Receive (RXD).
    #[inline]
    pub fn into_receive<const U: usize>(self) -> UartMux<A, I, MuxRxd<U>> {
        let config = self.base.uart_mux_group[I >> 3]
            .read()
            .set_signal(I & 0x7, MuxRxd::<U>::to_signal());
        self.base.uart_mux_group[I >> 3].write(config);
        UartMux {
            base: self.base,
            _mode: PhantomData,
        }
    }
    /// Configure the internal UART signal to Clear-to-Send (CTS).
    #[inline]
    pub fn into_clear_to_send<const U: usize>(self) -> UartMux<A, I, MuxCts<U>> {
        let config = self.base.uart_mux_group[I >> 3]
            .read()
            .set_signal(I & 0x7, MuxCts::<U>::to_signal());
        self.base.uart_mux_group[I >> 3].write(config);
        UartMux {
            base: self.base,
            _mode: PhantomData,
        }
    }
}

/// Available UART signal multiplexers.
pub struct UartMuxes<A: BaseAddress> {
    /// Multiplexer of UART signal 0.
    pub sig0: UartMux<A, 0, MuxRts<0>>,
    /// Multiplexer of UART signal 1.
    pub sig1: UartMux<A, 1, MuxRts<0>>,
    /// Multiplexer of UART signal 2.
    pub sig2: UartMux<A, 2, MuxRts<0>>,
    /// Multiplexer of UART signal 3.
    pub sig3: UartMux<A, 3, MuxRts<0>>,
    /// Multiplexer of UART signal 4.
    pub sig4: UartMux<A, 4, MuxRts<0>>,
    /// Multiplexer of UART signal 5.
    pub sig5: UartMux<A, 5, MuxRts<0>>,
    /// Multiplexer of UART signal 6.
    pub sig6: UartMux<A, 6, MuxRts<0>>,
    /// Multiplexer of UART signal 7.
    pub sig7: UartMux<A, 7, MuxRts<0>>,
    /// Multiplexer of UART signal 8.
    pub sig8: UartMux<A, 8, MuxRts<0>>,
    /// Multiplexer of UART signal 9.
    pub sig9: UartMux<A, 9, MuxRts<0>>,
    /// Multiplexer of UART signal 10.
    pub sig10: UartMux<A, 10, MuxRts<0>>,
    /// Multiplexer of UART signal 11.
    pub sig11: UartMux<A, 11, MuxRts<0>>,
}

/// Check if target gpio `Pin` is internally connected to UART signal index `I`.
pub trait HasUartSignal<const I: usize> {}

impl<A: BaseAddress> HasUartSignal<1> for Pin<A, 1, Uart> {}
impl<A: BaseAddress> HasUartSignal<2> for Pin<A, 2, Uart> {}
impl<A: BaseAddress> HasUartSignal<3> for Pin<A, 3, Uart> {}
impl<A: BaseAddress> HasUartSignal<4> for Pin<A, 4, Uart> {}
impl<A: BaseAddress> HasUartSignal<5> for Pin<A, 5, Uart> {}
impl<A: BaseAddress> HasUartSignal<6> for Pin<A, 6, Uart> {}
impl<A: BaseAddress> HasUartSignal<7> for Pin<A, 7, Uart> {}
impl<A: BaseAddress> HasUartSignal<8> for Pin<A, 8, Uart> {}
impl<A: BaseAddress> HasUartSignal<9> for Pin<A, 9, Uart> {}
impl<A: BaseAddress> HasUartSignal<10> for Pin<A, 10, Uart> {}
impl<A: BaseAddress> HasUartSignal<11> for Pin<A, 11, Uart> {}
impl<A: BaseAddress> HasUartSignal<0> for Pin<A, 12, Uart> {}
impl<A: BaseAddress> HasUartSignal<1> for Pin<A, 13, Uart> {}
impl<A: BaseAddress> HasUartSignal<2> for Pin<A, 14, Uart> {}
impl<A: BaseAddress> HasUartSignal<3> for Pin<A, 15, Uart> {}
impl<A: BaseAddress> HasUartSignal<4> for Pin<A, 16, Uart> {}
impl<A: BaseAddress> HasUartSignal<5> for Pin<A, 17, Uart> {}
impl<A: BaseAddress> HasUartSignal<6> for Pin<A, 18, Uart> {}
impl<A: BaseAddress> HasUartSignal<7> for Pin<A, 19, Uart> {}
impl<A: BaseAddress> HasUartSignal<8> for Pin<A, 20, Uart> {}
impl<A: BaseAddress> HasUartSignal<9> for Pin<A, 21, Uart> {}
impl<A: BaseAddress> HasUartSignal<10> for Pin<A, 22, Uart> {}
impl<A: BaseAddress> HasUartSignal<11> for Pin<A, 23, Uart> {}
impl<A: BaseAddress> HasUartSignal<0> for Pin<A, 24, Uart> {}
impl<A: BaseAddress> HasUartSignal<1> for Pin<A, 25, Uart> {}
impl<A: BaseAddress> HasUartSignal<2> for Pin<A, 26, Uart> {}
impl<A: BaseAddress> HasUartSignal<3> for Pin<A, 27, Uart> {}
impl<A: BaseAddress> HasUartSignal<4> for Pin<A, 28, Uart> {}
impl<A: BaseAddress> HasUartSignal<5> for Pin<A, 29, Uart> {}
impl<A: BaseAddress> HasUartSignal<6> for Pin<A, 30, Uart> {}
impl<A: BaseAddress> HasUartSignal<7> for Pin<A, 31, Uart> {}
impl<A: BaseAddress> HasUartSignal<8> for Pin<A, 32, Uart> {}
impl<A: BaseAddress> HasUartSignal<9> for Pin<A, 33, Uart> {}
impl<A: BaseAddress> HasUartSignal<10> for Pin<A, 34, Uart> {}
impl<A: BaseAddress> HasUartSignal<11> for Pin<A, 35, Uart> {}
impl<A: BaseAddress> HasUartSignal<0> for Pin<A, 36, Uart> {}
impl<A: BaseAddress> HasUartSignal<1> for Pin<A, 37, Uart> {}
impl<A: BaseAddress> HasUartSignal<2> for Pin<A, 38, Uart> {}
impl<A: BaseAddress> HasUartSignal<3> for Pin<A, 39, Uart> {}
impl<A: BaseAddress> HasUartSignal<4> for Pin<A, 40, Uart> {}
impl<A: BaseAddress> HasUartSignal<5> for Pin<A, 41, Uart> {}
impl<A: BaseAddress> HasUartSignal<6> for Pin<A, 42, Uart> {}
impl<A: BaseAddress> HasUartSignal<7> for Pin<A, 43, Uart> {}
impl<A: BaseAddress> HasUartSignal<8> for Pin<A, 44, Uart> {}
impl<A: BaseAddress> HasUartSignal<9> for Pin<A, 45, Uart> {}

/// Valid UART pins.
pub trait Pins<const U: usize> {
    /// Checks if this pin configuration includes Request-to-Send feature.
    const RTS: bool;
    /// Checks if this pin configuration includes Clear-to-Send feature.
    const CTS: bool;
    /// Checks if this pin configuration includes Transmit feature.
    const TXD: bool;
    /// Checks if this pin configuration includes Receive feature.
    const RXD: bool;
}

impl<A1, A2, const I: usize, const U: usize, const N: usize> Pins<U>
    for (Pin<A1, N, Uart>, UartMux<A2, I, MuxTxd<U>>)
where
    A1: BaseAddress,
    A2: BaseAddress,
    Pin<A2, N, Uart>: HasUartSignal<I>,
{
    const RTS: bool = false;
    const CTS: bool = false;
    const TXD: bool = true;
    const RXD: bool = false;
}

impl<
        A1,
        A2,
        A3,
        A4,
        const I1: usize,
        const I2: usize,
        const U: usize,
        const N1: usize,
        const N2: usize,
    > Pins<U>
    for (
        (Pin<A1, N1, Uart>, UartMux<A2, I1, MuxTxd<U>>),
        (Pin<A3, N2, Uart>, UartMux<A4, I2, MuxRxd<U>>),
    )
where
    A1: BaseAddress,
    A2: BaseAddress,
    A3: BaseAddress,
    A4: BaseAddress,
    Pin<A2, N1, Uart>: HasUartSignal<I1>,
    Pin<A4, N2, Uart>: HasUartSignal<I2>,
{
    const RTS: bool = false;
    const CTS: bool = false;
    const TXD: bool = true;
    const RXD: bool = true;
}

impl<
        A1,
        A2,
        A3,
        A4,
        const I1: usize,
        const I2: usize,
        const U: usize,
        const N1: usize,
        const N2: usize,
    > Pins<U>
    for (
        (Pin<A1, N1, Uart>, UartMux<A2, I1, MuxTxd<U>>),
        (Pin<A3, N2, Uart>, UartMux<A4, I2, MuxCts<U>>),
    )
where
    A1: BaseAddress,
    A2: BaseAddress,
    A3: BaseAddress,
    A4: BaseAddress,
    Pin<A2, N1, Uart>: HasUartSignal<I1>,
    Pin<A4, N2, Uart>: HasUartSignal<I2>,
{
    const RTS: bool = false;
    const CTS: bool = true;
    const TXD: bool = true;
    const RXD: bool = false;
}

impl<
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        A8,
        const I1: usize,
        const I2: usize,
        const I3: usize,
        const I4: usize,
        const U: usize,
        const N1: usize,
        const N2: usize,
        const N3: usize,
        const N4: usize,
    > Pins<U>
    for (
        (Pin<A1, N1, Uart>, UartMux<A2, I1, MuxTxd<U>>),
        (Pin<A3, N2, Uart>, UartMux<A4, I2, MuxRxd<U>>),
        (Pin<A5, N3, Uart>, UartMux<A6, I3, MuxRts<U>>),
        (Pin<A7, N4, Uart>, UartMux<A8, I4, MuxCts<U>>),
    )
where
    A1: BaseAddress,
    A2: BaseAddress,
    A3: BaseAddress,
    A4: BaseAddress,
    A5: BaseAddress,
    A6: BaseAddress,
    A7: BaseAddress,
    A8: BaseAddress,
    Pin<A2, N1, Uart>: HasUartSignal<I1>,
    Pin<A4, N2, Uart>: HasUartSignal<I2>,
    Pin<A6, N3, Uart>: HasUartSignal<I3>,
    Pin<A8, N4, Uart>: HasUartSignal<I4>,
{
    const RTS: bool = false;
    const CTS: bool = true;
    const TXD: bool = true;
    const RXD: bool = false;
}

/// Data writing register.
#[allow(non_camel_case_types)]
#[repr(transparent)]
pub struct DATA_WRITE(UnsafeCell<u8>);

/// Write data into first-in first-out queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct DataWrite(u8);

impl DATA_WRITE {
    /// Write a byte to first-in first-out queue.
    #[inline]
    pub fn write_u8(&self, val: u8) {
        unsafe { self.0.get().write_volatile(val) }
    }
}

/// Managed serial peripheral.
pub struct Serial<A: BaseAddress, PINS> {
    uart: UART<A>,
    pins: PINS,
}

impl<A: BaseAddress, PINS> Serial<A, PINS> {
    /// Creates a serial instance with same baudrate for transmit and receive.
    #[inline]
    pub fn new<const U: usize>(
        uart: UART<A>,
        config: Config,
        baudrate: Baud,
        pins: PINS,
        clocks: &Clocks,
        glb: &GLB<impl BaseAddress>,
    ) -> Self
    where
        PINS: Pins<U>,
    {
        // Enable clock
        let val = glb.uart_config.read().enable_clock();
        glb.uart_config.write(val);

        // Calculate transmit interval
        let uart_clock = clocks.uart_clock();
        let interval = uart_clock.0 / baudrate.0;
        if !(1..=65535).contains(&interval) {
            panic!("Impossible baudrate!");
        }
        let val = BitPeriod(0)
            .set_transmit_time_interval(interval as u16)
            .set_receive_time_interval(interval as u16);
        uart.bit_period.write(val);

        // Write bit order
        let val = DataConfig(0).set_bit_order(config.bit_order);
        uart.data_config.write(val);

        // Config transmit
        let mut val = TransmitConfig(0)
            .enable_freerun()
            .set_parity(config.parity)
            .set_stop_bits(config.stop_bits)
            .set_word_length(config.word_length);
        if PINS::TXD {
            val = val.enable_txd();
        }
        if PINS::CTS {
            val = val.enable_cts();
        }
        uart.transmit_config.write(val);

        Self { uart, pins }
    }

    /// Release serial instance and return its peripheral and pins.
    #[inline]
    pub fn free(self, glb: &GLB<impl BaseAddress>) -> (UART<A>, PINS) {
        let val = glb.uart_config.read().disable_clock();
        glb.uart_config.write(val);

        (self.uart, self.pins)
    }
}

impl embedded_hal::serial::Error for Error {
    fn kind(&self) -> embedded_hal::serial::ErrorKind {
        use embedded_hal::serial::ErrorKind;
        match self {
            Error::Framing => ErrorKind::FrameFormat,
            Error::Noise => ErrorKind::Noise,
            Error::Overrun => ErrorKind::Overrun,
            Error::Parity => ErrorKind::Parity,
        }
    }
}

impl<A: BaseAddress, PINS> embedded_hal::serial::ErrorType for Serial<A, PINS> {
    type Error = Error;
}

impl<A: BaseAddress, PINS> embedded_hal::serial::Write for Serial<A, PINS> {
    fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        for &word in buffer {
            self.uart.data_write.write_u8(word);
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

// requires to set `.set_function(Function::Uart)` before use.
const UART_GPIO_CONFIG: glb::GpioConfig = glb::GpioConfig::RESET_VALUE
    .enable_input()
    .enable_output()
    .enable_schmitt()
    .set_drive(glb::Drive::Drive0)
    .set_pull(glb::Pull::Up);

impl<A: BaseAddress, const N: usize, M: Alternate> Pin<A, N, M> {
    /// Configures the pin to operate as UART signal.
    #[inline]
    pub fn into_uart(self) -> Pin<A, N, Uart> {
        let config = UART_GPIO_CONFIG.set_function(Function::Uart);
        self.base.gpio_config[N].write(config);
        Pin {
            base: self.base,
            _mode: PhantomData,
        }
    }
}

/// Serial configuration.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Config {
    /// Data bit order.
    pub bit_order: BitOrder,
    /// Parity settings.
    pub parity: Parity,
    /// Serial stop bits.
    pub stop_bits: StopBits,
    /// Data word length.
    pub word_length: WordLength,
}

/// Order of the bits transmitted and received on the wire.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BitOrder {
    /// Each byte is sent out LSB-first.
    LsbFirst,
    /// Each byte is sent out MSB-first.
    MsbFirst,
}

/// Parity check.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Parity {
    /// No parity check.
    None,
    /// Even parity bit.
    Even,
    /// Odd parity bit.
    Odd,
}

/// Stop bits.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StopBits {
    /// 0.5 stop bits.
    ZeroPointFive,
    /// 1 stop bit.
    One,
    /// 1.5 stop bits.
    OnePointFive,
    /// 2 stop bits.
    Two,
}

/// Word length.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WordLength {
    /// Five bits per word.
    Five,
    /// Six bits per word.
    Six,
    /// Seven bits per word.
    Seven,
    /// Eight bits per word.
    Eight,
}

/// Serial error.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Framing error.
    Framing,
    /// Noise error.
    Noise,
    /// RX buffer overrun.
    Overrun,
    /// Parity check error.
    Parity,
}

#[cfg(test)]
mod tests {
    use super::RegisterBlock;
    use memoffset::offset_of;

    #[test]
    fn struct_register_block_offset() {
        assert_eq!(offset_of!(RegisterBlock, transmit_config), 0x0);
        assert_eq!(offset_of!(RegisterBlock, bit_period), 0x08);
        assert_eq!(offset_of!(RegisterBlock, data_config), 0x0c);
        assert_eq!(offset_of!(RegisterBlock, bus_state), 0x30);
        assert_eq!(offset_of!(RegisterBlock, fifo_config_1), 0x84);
        assert_eq!(offset_of!(RegisterBlock, data_write), 0x88);
    }
}