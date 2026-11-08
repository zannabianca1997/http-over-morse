//! Wire
//!
//! Made up of a listening pin and a writing pin

use core::convert::Infallible;

use embedded_hal::digital::{InputPin, OutputPin};

use crate::{
    led::DebugLed,
    pins::{Pin2, Pin3, Pin4, Pin13},
};

trait WirePins {
    type Rx: InputPin<Error = Infallible>;
    fn rx(&self) -> &Self::Rx;
    type Tx: OutputPin<Error = Infallible>;
    fn tx(&self) -> &Self::Tx;
    fn tx_mut(&mut self) -> &mut Self::Tx;
}

pub struct WireA {
    rx: Pin3,
    tx: Pin4,
}

impl WirePins for WireA {
    type Rx = Pin3;
    fn rx(&self) -> &Self::Rx {
        &self.rx
    }
    type Tx = Pin4;
    fn tx(&self) -> &Self::Tx {
        &self.tx
    }
    fn tx_mut(&mut self) -> &mut Self::Tx {
        &mut self.tx
    }
}

impl WireA {
    pub(crate) fn new(rx: Pin3, tx: Pin4) -> Self {
        Self { rx, tx }
    }
}

pub struct WireB {
    rx: Pin2,
    tx: Pin13,
}

impl WirePins for WireB {
    type Rx = Pin2;
    fn rx(&self) -> &Self::Rx {
        &self.rx
    }
    type Tx = Pin13;
    fn tx(&self) -> &Self::Tx {
        &self.tx
    }
    fn tx_mut(&mut self) -> &mut Self::Tx {
        &mut self.tx
    }
}

impl WireB {
    pub(crate) fn new(rx: Pin2, tx: Pin13) -> Self {
        Self { rx, tx }
    }

    /// Downgrade the wire to the led alone
    pub fn downgrade_to_led(self) -> DebugLed {
        DebugLed::new(self.tx)
    }
}

pub trait Wire: WirePins {}

impl<T: WirePins> Wire for T {}
