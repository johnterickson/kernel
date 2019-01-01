#![feature(core_intrinsics)]
#![no_std]
#![no_main]

extern crate common;
extern crate keyboard;
extern crate interrupts;
extern crate serial;
extern crate vga;

#[macro_use]
extern crate intermezzos;
extern crate bootloader;

#[macro_use]
extern crate lazy_static;
extern crate spin;

#[cfg(not(test))]
pub mod panic;

use core::sync::atomic::{AtomicUsize,Ordering};
use interrupts::{Idt, IdtRef};
use keyboard::Keyboard;
use spin::Mutex;
use serial::{SerialPort,COM1};
use vga::Vga;

pub struct Context {
    pub vga: Mutex<Vga<&'static mut [u8]>>,
    pub idt: IdtRef<'static>,
    pub com1: SerialPort,
    pub keyboard: Keyboard,
    time: AtomicUsize,
}

impl Context {
    pub fn new(idt: &'static Idt) -> Context {
        let slice = unsafe {
            core::slice::from_raw_parts_mut(0xb8000 as *mut u8, 4000)
        };

        Context {
            vga: Mutex::new(Vga::new(slice)),
            idt: IdtRef::from_idt(idt),
            keyboard: Keyboard::new(),
            com1: SerialPort::create(COM1),
            time: AtomicUsize::new(0)
        }
    }

    pub fn ticks(&self) -> usize {
        self.time.load(Ordering::SeqCst)
    }

    pub fn on_tick(&self) {
        self.time.fetch_add(1, Ordering::SeqCst);
    }
}

lazy_static! {
    static ref IDT: Idt = {
        Idt::new()
    };
}

lazy_static! {
    static ref CONTEXT: Context = {
        Context::new(&IDT)
    };
}

#[no_mangle]
pub fn _start() -> ! {
    panic!("TODO: Implement OS.")

    // kprintln!(CONTEXT, "hello world");
    //loop {}
}
