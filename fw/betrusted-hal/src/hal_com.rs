#[allow(dead_code)]

/// com_txrx is a polled-implementation of an atomic TX/RX swap operation
/// assumes that transaction is *not* in progress on entry to this function
/// this invariant is enforced by the function itself, but if another routine
/// is used to access the block make sure to leave that function with the
/// transaction finished.
pub fn com_txrx(p: &betrusted_pac::Peripherals, tx: u16) -> u16 {
    // load the TX register
    unsafe{ p.COM.tx.write(|w| w.bits(tx as u32)); } // transaction is automatically iniated on write

    // wait until the done register is set
    while !p.COM.status.read().tip().bit_is_set() { }

    // grab the RX value and return it
    let rx: u16 = p.COM.rx.read().bits() as u16;
    rx
}