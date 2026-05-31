use arduino_hal::pac::EXINT;

pub struct InitForD2Toggle {
    _priv: (),
}

/// Init external interrupts
///
/// Turns on the INT0 interrupt, on any change of the D2 pin
pub fn init(exint: EXINT) -> (InitForD2Toggle,) {
    // Set to measure any change
    exint.eicra().write(|w| w.isc0().val_0x01());
    // Activate INT0
    exint.eimsk().write(|w| w.int0().set_bit());

    drop(exint);
    (InitForD2Toggle { _priv: () },)
}
