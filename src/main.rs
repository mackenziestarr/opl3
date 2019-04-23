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

    fn make_output_from_pin(pin: port::Pin) -> port::Gpio {
        let mut gpio = pin.make_gpio();
        gpio.output();
        gpio
    }

    let (mut cs, mut rd, mut wr, mut a0, mut a1, mut led, mut clock, mut data) = (
        make_output_from_pin(teensy::gpio(2)),
        make_output_from_pin(teensy::gpio(3)),
        make_output_from_pin(teensy::gpio(4)),
        make_output_from_pin(teensy::gpio(9)),
        make_output_from_pin(teensy::gpio(10)),
        make_output_from_pin(teensy::gpio(13)),
        make_output_from_pin(teensy::gpio(11)),
        make_output_from_pin(teensy::gpio(12))
    );

    led.high();

    let mut uart = unsafe {
        let rx = teensy::gpio(0).make_rx();
        let tx = teensy::gpio(1).make_tx();
        uart::Uart::new(0, Some(rx), Some(tx), (468, 24)) // 9600 baud
    };

    writeln!(uart, "Hello World").unwrap();

    fn shift_out(data: &mut port::Gpio, clock: &mut port::Gpio, value: u8, uart: &mut uart::Uart) {
        // clear shift register out
        for _ in 0..8 {
            data.low();
            clock.low();
            clock.high();
        }
        for i in 0 .. 8 {
            match value & (1 << (7 - i)) {
                1 ... core::u8::MAX => {
                    write!(uart, "1").unwrap();
                    data.high()
                },
                0 => {
                    write!(uart, "0").unwrap();
                    data.low()
                }
                _ => ()
            }
            clock.low();
            clock.high();
        }
        writeln!(uart, "").unwrap();
    }

    fn sleep() {
        for _i in 0..1_000_000 {
            unsafe {
                asm!("nop" : : : "memory");
            }
        }
    }

    fn opl3_write(
        cs: &mut port::Gpio,
        rd: &mut port::Gpio,
        wr: &mut port::Gpio,
        a0: &mut port::Gpio,
        a1: &mut port::Gpio,
        data: &mut port::Gpio,
        clock: &mut port::Gpio,
        address: u8,
        value: u8,
        uart: &mut uart::Uart
    ) {
        cs.low();
        rd.high();
        wr.low();

        // write address
        a0.low();
        a1.low();
        shift_out(data, clock, address, uart);
        sleep();

        // write data
        a0.high();
        shift_out(data, clock, value, uart);

        cs.high()
    }

    opl3_write(&mut cs, &mut rd, &mut wr, &mut a0, &mut a1, &mut data, &mut clock, 0xbd, 1 << 4, uart);
    loop {
        opl3_write(&mut cs, &mut rd, &mut wr, &mut a0, &mut a1, &mut data, &mut clock, 0xbd, 1 << 3, uart);
        sleep();
        sleep()
    }
        // sleep();
        // opl3_write(&mut a0, &mut a1, &mut data, &mut clock, 0xbd, 0x34);
        // sleep();
}
