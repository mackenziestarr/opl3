use core;
use volatile::Volatile;
use bit_field::BitField;

pub enum PortName {
    A,
    B,
    C,
    D,
    E
}

#[repr(C,packed)]
pub struct Port {
    pcr: [Volatile<u32>; 32],
    gpclr: u32,
    gpchr: u32,
    reserved_0: [u8; 24],
    isfr: u32,
}

#[repr(C,packed)]
pub struct Pin {
    port: *mut Port,
    pin: usize
}

#[repr(C,packed)]
struct GpioBitband {
    pdor: [u32; 32],
    psor: [u32; 32],
    pcor: [u32; 32],
    ptor: [u32; 32],
    pdir: [u32; 32],
    pddr: [u32; 32]
}

#[repr(C,packed)]
pub struct Gpio {
    gpio: *mut GpioBitband,
    pin: usize
}

impl Pin {
    pub fn make_gpio(self) -> Gpio {
        unsafe {
            let port = &mut *self.port;
            port.set_pin_mode(self.pin, 1);
            Gpio::new(port.name(), self.pin)
        }
    }
}

impl Gpio {
    pub unsafe fn new(port: PortName, pin: usize) -> Gpio {
        let gpio = match port {
            PortName::A => 0x43FE0000 as *mut GpioBitband,
            PortName::B => 0x43FE0800 as *mut GpioBitband,
            PortName::C => 0x43FE1000 as *mut GpioBitband,
            PortName::D => 0x43FE1800 as *mut GpioBitband,
            PortName::E => 0x43FE2000 as *mut GpioBitband
        };

        Gpio { gpio: gpio, pin: pin }
    }

    pub fn make_output(&mut self) -> &mut Gpio {
        self.output();
        self
    }

    pub fn output(&mut self) {
        unsafe {
            core::ptr::write_volatile(&mut (*self.gpio).pddr[self.pin], 1);
        }
    }

    pub fn high(&mut self) {
        unsafe {
            core::ptr::write_volatile(&mut (*self.gpio).psor[self.pin], 1);
        }
    }

    pub fn low(&mut self) {
        unsafe {
            core::ptr::write_volatile(&mut (*self.gpio).pcor[self.pin], 1);
        }
    }

}


impl Port {

    pub fn name(&self) -> PortName {
        let addr = (self as *const Port) as u32;
        match addr {
            0x40049000 => PortName::A,
            0x4004A000 => PortName::B,
            0x4004B000 => PortName::C,
            0x4004C000 => PortName::D,
            0x4004D000 => PortName::E,
            _ => unreachable!()
        }
    }

    pub unsafe fn pin(&mut self, p: usize) -> Pin {
        Pin { port: self, pin: p }
    }

    pub unsafe fn new(name: PortName) -> &'static mut Port {
        &mut * match name {
            PortName::A => 0x40049000 as *mut Port,
            PortName::B => 0x4004A000 as *mut Port,
            PortName::C => 0x4004B000 as *mut Port,
            PortName::D => 0x4004C000 as *mut Port,
            PortName::E => 0x4004D000 as *mut Port
        }
    }

    pub fn set_pin_mode(&mut self, p: usize, mut mode: u32) {
        self.pcr[p].update(|pcr| {
            pcr.set_bits(8..11, mode);
        });
    }
}

pub struct Tx(u8);
pub struct Rx(u8);

impl Pin {
    pub fn make_rx(self) -> Rx {
        unsafe {
            let port = &mut *self.port;
            match (port.name(), self.pin) {
                (PortName::B, 16) => {
                    port.set_pin_mode(self.pin, 3);
                    Rx(0)

                }
                _ => panic!("Invalid serial Rx pin")
            }
        }
    }

    pub fn make_tx(self) -> Tx {
        unsafe {
            let port = &mut *self.port;
            match (port.name(), self.pin) {
                (PortName::B, 17) => {
                    port.set_pin_mode(self.pin, 3);
                    Tx(0)
                },
                _ => panic!("Invalid serial Tx pin")
            }
        }
    }
}

impl Rx {
    pub fn uart(&self) -> u8 {
        self.0
    }
}

impl Tx {
    pub fn uart(&self) -> u8 {
        self.0
    }
}
