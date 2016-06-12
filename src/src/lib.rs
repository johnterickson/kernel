#![feature(lang_items)]
#![feature(asm)]
#![no_std]

extern crate rlibc;

#[macro_use]
extern crate vga;

extern crate interrupts;
extern crate keyboard;
extern crate pic;

pub mod support; // For Rust lang items

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    pic::remap();

    vga::clear_console();
    vga::initialize_cursor();

    interrupts::install();
    interrupts::enable();

    vga::clear_console();
    kprintln!("Kernel initialized.");

    loop { }
}
