#![feature(lang_items,asm)]
#![no_std]
#![no_main]

extern crate volatile;
extern crate bit_field;

use core::fmt::Write;
use core::panic::PanicInfo;

mod watchdog;
mod mcg;
mod sim;
mod port;
mod osc;
mod uart;

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


#[panic_handler]
pub extern fn panic_fmt(_info: &PanicInfo) -> ! {
    // do something here
    loop{}
}

extern {
    fn _stack_top();
}


extern fn main() {
    let (wdog, sim, mcg, osc) = unsafe {
        (watchdog::Watchdog::new(),
         sim::Sim::new(),
         mcg::Mcg::new(),
         osc::Osc::new())
    };

    wdog.disable();

    // Enable the crystal oscillator with 10pf of capacitance
    osc.enable(10);

    sim.enable_clock(sim::Clock::PortC);
    sim.enable_clock(sim::Clock::PortB);
    sim.enable_clock(sim::Clock::Uart0);

    sim.set_dividers(1, 2, 3);

    if let mcg::Clock::Fei(mut fei) = mcg.clock() {
        // Our 16MHz xtal is "very fast", and needs to be divided
        // by 512 to be in the acceptable FLL range.
        fei.enable_xtal(mcg::OscRange::VeryHigh);
        let fbe = fei.use_external(512);
        // PLL is 27/6 * xtal == 72MHz
        let pbe = fbe.enable_pll(27, 6);
        pbe.use_pll();
    } else {
        panic!("Somehow the clock wasn't in FEI mode");
    }

    fn make_output(port: port::PortName, pin_num: usize) -> port::Gpio {
        let pin = unsafe {
            port::Port::new(port).pin(pin_num)
        };
        let mut gpio = pin.make_gpio();
        gpio.output();
        gpio
    }

    let (mut a0, mut a1, mut led, mut clock, mut data) = (
        make_output(port::PortName::C, 3),
        make_output(port::PortName::C, 4),
        make_output(port::PortName::C, 5),
        make_output(port::PortName::C, 6),
        make_output(port::PortName::C, 7)
    );

    led.high();

    let mut uart = unsafe {
        let rx = port::Port::new(port::PortName::B).pin(16).make_rx();
        let tx = port::Port::new(port::PortName::B).pin(17).make_tx();
        uart::Uart::new(0, Some(rx), Some(tx), (468, 24))
    };

    writeln!(uart, "sup").unwrap();

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
        for _i in 0..1_000_000_0 {
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

    // opl3_write(&mut a0, &mut a1, &mut data, &mut clock, 0xbd, 0x20);
    // sleep();
    shift_out(&mut data, &mut clock, 0x22);
    loop {
        // sleep();
        // opl3_write(&mut a0, &mut a1, &mut data, &mut clock, 0xbd, 0x34);
        // sleep();
    }
}
