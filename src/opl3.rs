use port;

// @todo refactor out
enum Time {
    Microseconds
}

const CYCLES_PER_MICROSECOND: u32 = 72;

fn sleep(value: u32, unit: Time) {
    for _i in 0..(value * CYCLES_PER_MICROSECOND) {
        unsafe {
            asm!("nop" : : : "memory");
        }
    }
}

pub struct Opl3 {
    pub cs : port::Gpio,
    pub rd : port::Gpio,
    pub wr : port::Gpio,
    pub ic : port::Gpio,
    pub a0 : port::Gpio,
    pub a1 : port::Gpio,
    pub d0 : port::Gpio,
    pub d1 : port::Gpio,
    pub d2 : port::Gpio,
    pub d3 : port::Gpio,
    pub d4 : port::Gpio,
    pub d5 : port::Gpio,
    pub d6 : port::Gpio,
    pub d7 : port::Gpio
}

impl Opl3 {
    fn write_data(&mut self, value: u8) {
        if (value & 1) == 1 { self.d0.high() }
        if (value & 2) == 2 { self.d1.high() }
        if (value & 4) == 4 { self.d2.high() }
        if (value & 8) == 8 { self.d3.high() }
        if (value & 16) == 16 { self.d4.high() }
        if (value & 32) == 32 { self.d5.high() }
        if (value & 64) == 64 { self.d6.high() }
        if (value & 128) == 128 { self.d7.high() }
    }
    pub fn clear_data(&mut self) {
        self.d0.low();
        self.d1.low();
        self.d2.low();
        self.d3.low();
        self.d4.low();
        self.d5.low();
        self.d6.low();
        self.d7.low();
    }
    pub fn init(&mut self) {
        //reset
        self.ic.low();
        sleep(1000, Time::Microseconds);
        self.ic.high();

        //clear all registers
        for i in 0x00..=0xff {
            self.write(i, 0x00)
        }

        self.clear_data();
    }
    pub fn write(&mut self, address: u8, value: u8) {
        // write address
        self.a0.low();
        self.a1.low();
        sleep(3, Time::Microseconds);

        self.cs.low();
        sleep(3, Time::Microseconds);

        self.wr.low();
        self.rd.high();
        sleep(3, Time::Microseconds);

        self.write_data(address);
        sleep(3, Time::Microseconds);

        // write data
        self.a0.high();
        sleep(3, Time::Microseconds);

        self.write_data(value);
        sleep(3, Time::Microseconds);

        self.clear_data();
        self.cs.high();
    }
}
