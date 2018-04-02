#![feature(lang_items,asm)]
#![no_std]
#![no_main]

mod watchdog;
mod sim;
mod port;

#[link_section = ".vectors"]
#[no_mangle]
pub static _VECTORS: [unsafe extern fn(); 2] = [
    _stack_top,
    main,
];

#[link_section = ".flashconfig"]
#[no_mangle]
pub static _FLASHCONFIG: [u8; 16] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xDE, 0xF9, 0xFF, 0xFF
];

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_panic(
    _msg: core::fmt::Arguments,
    _file: &'static str,
    _line: u32) -> ! {
    loop {}
}

extern {
    fn _stack_top();
}


extern fn main() {
    let (wdog,sim) = unsafe {
        (watchdog::Watchdog::new(),
         sim::Sim::new())
    };

    wdog.disable();
    sim.enable_clock(sim::Clock::PortC);


    fn make_output(pin_num: usize) -> port::Gpio {
        let pin = unsafe {
            port::Port::new(port::PortName::C).pin(pin_num)
        };
        let mut gpio = pin.make_gpio();
        gpio.output();
        gpio
    }

    let (mut a0, mut a1, mut led, mut clock, mut data) = (
        make_output(3),
        make_output(4),
        make_output(5),
        make_output(6),
        make_output(7)
    );

    led.high();

    fn shift_out(data: &mut port::Gpio, clock: &mut port::Gpio, value: u8) {
        // clear shift register out
        for _ in 0..8 {
            data.low();
            clock.high();
            clock.low();
        }
        for i in 0..8 {
            match value & (1 << (7 - i)) {
                1 ... core::u8::MAX => data.high(),
                0 => data.low(),
                _ => ()
            }
            clock.high();
            clock.low();
        }
    }

    fn sleep() {
        for _i in 0..1_000_000 {
            unsafe {
                asm!("nop" : : : "memory");
            }
        }
    }


    fn opl3_write(
        a0: &mut port::Gpio,
        a1: &mut port::Gpio,
        data: &mut port::Gpio,
        clock: &mut port::Gpio,
        address: u8,
        value: u8
    ) {
        // write address
        a0.low();
        a1.low();
        shift_out(data, clock, address);
        sleep();

        // write data
        a0.high();
        shift_out(data, clock, value);
        sleep();
    }

    opl3_write(&mut a0, &mut a1, &mut data, &mut clock, 0xbd, 0x20);
    sleep();

    loop {
        sleep();
        opl3_write(&mut a0, &mut a1, &mut data, &mut clock, 0xbd, 0x34);
        sleep();
    }
}
