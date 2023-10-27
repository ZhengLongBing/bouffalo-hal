// Build this example with:
// rustup target install riscv64imac-unknown-none-elf
// cargo build --target riscv64imac-unknown-none-elf --release -p jtag-demo

#![no_std]
#![no_main]

use base_address::Static;
use bl_rom_rt::entry;
use bl_soc::{gpio::Pads, prelude::*};
use panic_halt as _;

#[entry]
fn main() -> ! {
    let gpio: Pads<Static<0x20000000>> = unsafe { core::mem::transmute(()) };
    // enable jtag
    gpio.io0.into_jtag_d0();
    gpio.io1.into_jtag_d0();
    gpio.io2.into_jtag_d0();
    gpio.io3.into_jtag_d0();

    let mut led = gpio.io8.into_floating_output();
    loop {
        led.set_low().ok();
        unsafe { riscv::asm::delay(100_000) };
        led.set_high().ok();
        unsafe { riscv::asm::delay(100_000) };
    }
}
