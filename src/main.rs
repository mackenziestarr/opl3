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
mod teensy;
mod opl3;

use opl3::Opl3;

const CLOCK_SPEED_MHZ: u32 = 72;
const CYCLES_PER_MICROSECOND: u32 = 72;

enum Time {
    Microseconds
}

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

    sim.enable_clock(sim::Clock::PortA);
    sim.enable_clock(sim::Clock::PortB);
    sim.enable_clock(sim::Clock::PortC);
    sim.enable_clock(sim::Clock::PortD);
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

    let mut opl3 = Opl3 {
        cs : make_output_from_pin(teensy::gpio(2)),
        rd : make_output_from_pin(teensy::gpio(3)),
        wr : make_output_from_pin(teensy::gpio(4)),
        ic : make_output_from_pin(teensy::gpio(5)),
        a0 : make_output_from_pin(teensy::gpio(9)),
        a1 : make_output_from_pin(teensy::gpio(10)),
        d0 : make_output_from_pin(teensy::gpio(14)),
        d1 : make_output_from_pin(teensy::gpio(15)),
        d2 : make_output_from_pin(teensy::gpio(16)),
        d3 : make_output_from_pin(teensy::gpio(17)),
        d4 : make_output_from_pin(teensy::gpio(18)),
        d5 : make_output_from_pin(teensy::gpio(19)),
        d6 : make_output_from_pin(teensy::gpio(20)),
        d7 : make_output_from_pin(teensy::gpio(21)),
    };

    let (mut led, mut clock, mut data) = (
        make_output_from_pin(teensy::gpio(13)),
        make_output_from_pin(teensy::gpio(11)),
        make_output_from_pin(teensy::gpio(12))
    );

    let mut uart = unsafe {
        let rx = teensy::gpio(0).make_rx();
        let tx = teensy::gpio(1).make_tx();
        uart::Uart::new(0, Some(rx), Some(tx), (468, 24)) // 9600 baud
    };

    // verify things are happening
    led.high();
    writeln!(uart, "Hello World").unwrap();
    opl3.init();

    // enable rhythm mode
    opl3.write(0xbd, 0x20);

    loop {
        opl3.write(0xbd, 0x20);
        sleep(10000, Time::Microseconds);
    }
}


fn sleep(value: u32, unit: Time) {
    for _i in 0..(value * CYCLES_PER_MICROSECOND) {
        unsafe {
            asm!("nop" : : : "memory");
        }
    }
}

fn make_output_from_pin(pin: port::Pin) -> port::Gpio {
    let mut gpio = pin.make_gpio();
    gpio.output();
    gpio
}
