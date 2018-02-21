use core;

pub enum PortName {
    C
}

#[repr(C,packed)]
pub struct Port {
    pcr: [u32; 32],
    gpclr: u32,
    gpchr: u32,
    reserved_0: [u8; 24],
    isfr: u32,
}

impl Port {
    pub unsafe fn new(name: PortName) -> &'static mut Port {
        &mut * match name {
            PortName::C => 0x4004B000 as *mut Port
        }
    }

    pub unsafe fn set_pin_mode(&mut self, p: usize, mut mode: u32) {
        let mut pcr = core::ptr::read_volatile(&self.pcr[p]);
        pcr &= 0xFFFFF8FF;
        mode &= 0x00000007;
        mode <<= 8;
        pcr |= mode;
        core::ptr::write_volatile(&mut self.pcr[p], pcr);
    }
}
