use port;


// maps physical pins on teensy 3.2 to virtual port + pin pairs on the
// cortext m4
pub fn gpio(pin: u8) -> port::Pin {
    let (port, pin) = match pin {
        0  => (port::PortName::B, 16),
        1  => (port::PortName::B, 17),
        2  => (port::PortName::D, 0),
        3  => (port::PortName::A, 12),
        4  => (port::PortName::A, 13),
        9  => (port::PortName::C, 3),
        10 => (port::PortName::C, 4),
        11 => (port::PortName::C, 6),
        12 => (port::PortName::C, 7),
        13 => (port::PortName::C, 5),
        _ => panic!("not implemented")
    };
    unsafe {
        port::Port::new(port).pin(pin)
    }
}
