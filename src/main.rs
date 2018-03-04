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
    let (wdog,sim,ledpin) = unsafe {
        (watchdog::Watchdog::new(),
         sim::Sim::new(),
         port::Port::new(port::PortName::C).pin(5))
    };

    let (clock_pin, data_pin) = unsafe {
        (
            port::Port::new(port::PortName::C).pin(6),
            port::Port::new(port::PortName::C).pin(7)
        )
    };

    wdog.disable();
    sim.enable_clock(sim::Clock::PortC);


    let mut ledgpio = ledpin.make_gpio();
    ledgpio.output();
    ledgpio.high();

    let mut clock = clock_pin.make_gpio();
    clock.output();

    let mut data = data_pin.make_gpio();
    data.output();
    data.high();


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

    shift_out(&mut data, &mut clock, 255);
    shift_out(&mut data, &mut clock, 10);
    shift_out(&mut data, &mut clock, 8);
    loop {}
}
