#![feature(lang_items,asm)]
#![no_std]
#![no_main]

#[lang = "panic_fmt"]
pub extern fn rust_begin_panic(
    _msg: core::fmt::Arguments,
    _file: &'static str,
    _line: u32) -> ! {
    loop {}
}

extern {
    fn _stack_top();
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


fn disable_watchdog() {
    /* https://www.pjrc.com/teensy/K20P64M72SF1RM.pdf
     * section 23.3.1 unlocking and updating the watchdog */
    unsafe {
        let mut wdog_unlock = 0x4005_200E;
        let mut wdog_stctrlh = 0x4005_2000;
        core::ptr::write_volatile(&mut wdog_unlock, 0xC520);
        core::ptr::write_volatile(&mut wdog_unlock, 0xD928);
        // wait 2 clock cycles to read
        asm!("nop" : : : "memory");
        asm!("nop" : : : "memory");
        // update the watchdog config to disabled
        let mut frame = core::ptr::read_volatile(&wdog_stctrlh);
        frame &= !(0x00000001);
        core::ptr::write_volatile(&mut wdog_stctrlh, frame);
    }
}

fn enable_clock() {
    /* only enable system clock on port C for now
     * https://www.pjrc.com/teensy/K20P64M72SF1RM.pdf
     * section 12.2.12 system clock gating control register 5 */
    unsafe {
        let mut sim_scgc5 = 0x4004_8038;
        let mut scgc = core::ptr::read_volatile(&sim_scgc5);
        scgc |= 0x00000800;
        core::ptr::write_volatile(&mut sim_scgc5, scgc);
    }
}

fn set_pin_mode() {
    unsafe {
        let mut portc_pcr5 = 0x4004_B014;
        let mut pcr : u32 = core::ptr::read_volatile(&portc_pcr5);
        // hard code to gpio
        pcr &= 0xFFFFF8FF;
        let mut mode = 1;
        mode &= 0x00000007;
        mode <<= 8;
        pcr |= mode;
        core::ptr::write_volatile(&mut portc_pcr5, pcr);
    }
}

fn turn_on_led() {
    unsafe {
        // set GPIO to output mode
        let mut gpioc_pddr = 0x400F_F094;
        let mut x : u32 = core::ptr::read_volatile(&gpioc_pddr);
        x |= 0xFFFFFFFF;
        core::ptr::write_volatile(&mut gpioc_pddr, x);
        let mut gpioc_psor = 0x400F_F084;
        let mut y : u32 = core::ptr::read_volatile(&gpioc_psor);
        // the highs are high
        y |= 0xFFFFFFFF;
        core::ptr::write_volatile(&mut gpioc_psor, y);
    }
}

pub extern fn main() {
    disable_watchdog();
    enable_clock();
    set_pin_mode();
    turn_on_led();
    loop {}
}
