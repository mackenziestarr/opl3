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
        5  => (port::PortName::D, 7),
        6  => (port::PortName::D, 4),
        7  => (port::PortName::D, 2),
        8  => (port::PortName::D, 3),
        9  => (port::PortName::C, 3),
        10 => (port::PortName::C, 4),
        11 => (port::PortName::C, 6),
        12 => (port::PortName::C, 7),
        13 => (port::PortName::C, 5),
        14 => (port::PortName::D, 1),
        15 => (port::PortName::C, 0),
        16 => (port::PortName::B, 0),
        17 => (port::PortName::B, 1),
        18 => (port::PortName::B, 3),
        19 => (port::PortName::B, 2),
        20 => (port::PortName::D, 5),
        21 => (port::PortName::D, 6),
        _ => panic!("not implemented")
    };
    unsafe {
        port::Port::new(port).pin(pin)
    }
}
