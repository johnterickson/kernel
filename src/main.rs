#![feature(asm)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(naked_functions)]
#![no_std]
#![no_main]

extern crate common;
extern crate keyboard;
#[macro_use]
extern crate interrupts;
extern crate pic;
extern crate serial;
extern crate vga;

#[macro_use]
extern crate intermezzos;
extern crate bootloader;
extern crate x86;

#[macro_use]
extern crate lazy_static;
extern crate spin;

#[cfg(not(test))]
pub mod panic;
mod thread;

use core::intrinsics;
use core::sync::atomic::{AtomicUsize,Ordering};
use interrupts::{Idt, IdtRef};
use keyboard::Keyboard;
use spin::Mutex;
use serial::{SerialPort,COM1};
use thread::*;
use vga::Vga;
use x86::bits64::irq::IdtEntry;

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
    kprintln!(CONTEXT, "Initializing APIC...");

    pic::remap();

    kprintln!(CONTEXT, "Initializing COM1...");
    CONTEXT.com1.init();

    kprintln!(CONTEXT, "Configuring interrupts...");

    CONTEXT.idt.set_handler(0, make_idt_entry!(isr0, |state| {
        kprintln!(CONTEXT, "Divide by zero: {:?}", state);
        loop {}
    }));
    CONTEXT.idt.set_handler(1, make_idt_entry!(isr1, |state| {
        kprintln!(CONTEXT, "Trap: {:?}", state);
        pic::eoi_for(1);
    }));
    CONTEXT.idt.set_handler(2, make_idt_entry!(isr2, |state| {
        kprint!(CONTEXT, "NMI: {:?}", state);
        loop {}
    }));
    CONTEXT.idt.set_handler(3, make_idt_entry!(isr3, |state| {
        kprintln!(CONTEXT, "Breakpoint: {:?}", state);
        pic::eoi_for(3);
    }));
    CONTEXT.idt.set_handler(4, make_idt_entry!(isr4, |state| {
        kprint!(CONTEXT, "Overflow: {:?}", state);
        loop {}
    }));
    CONTEXT.idt.set_handler(5, make_idt_entry!(isr5, |state| {
        kprint!(CONTEXT, "Bounds: {:?}", state);
        loop {}
    }));
    CONTEXT.idt.set_handler(6, make_idt_entry!(isr6, |state| {
        kprint!(CONTEXT, "Invalid opcode: {:?}", state);
        loop {}
    }));
    CONTEXT.idt.set_handler(7, make_idt_entry!(isr7, |state| {
        kprint!(CONTEXT, "Device not available: {:?}", state);
        loop {}
    }));
    CONTEXT.idt.set_handler(8, make_idt_entry!(isr8, |state| {
        kprint!(CONTEXT, "Double fault: {:?}", state);
        loop {}
    }));
    CONTEXT.idt.set_handler(9, make_idt_entry!(isr9, |state| {
        kprint!(CONTEXT, "Coprocessor segment overrun: {:?}", state);
        loop {}
    }));
    CONTEXT.idt.set_handler(10, make_idt_entry!(isr10, |state| {
        kprint!(CONTEXT, "Invalid TSS: {:?}", state);
        loop {}
    }));
    CONTEXT.idt.set_handler(11, make_idt_entry!(isr11, |state| {
        kprint!(CONTEXT, "Segment not present: {:?}", state);
        loop {}
    }));
    CONTEXT.idt.set_handler(12, make_idt_entry!(isr12, |state| {
        kprint!(CONTEXT, "Stack segment fault: {:?}", state);
        loop {}
    }));
    CONTEXT.idt.set_handler(13, make_idt_entry!(isr13, |state| {
        kprint!(CONTEXT, "General protection fault: {:?}", state);
        loop {}
    }));
    CONTEXT.idt.set_handler(14, make_idt_entry!(isr14, |state| {
        kprint!(CONTEXT, "Page fault: {:?}", state);
        loop {}
    }));

    // IRQ0 (0) on PIC1 (32), so IDT index is 32
    // Keyboard uses IRQ1 and PIC1 has been remapped to 0x20 (32); therefore
    // the index in the IDT for IRQ1 will be 32 + 1 = 33
    CONTEXT.idt.set_handler(32, make_idt_entry!(isr32, |_state| {
        CONTEXT.on_tick();
        pic::eoi_for(32);
    }));
    CONTEXT.idt.set_handler(33, make_idt_entry!(isr33, |_state| {
        CONTEXT.keyboard.isr();
        pic::eoi_for(33);
    }));
    CONTEXT.idt.set_handler(35, make_idt_entry!(isr35, |_state| {
        kprint!(CONTEXT, "COM2/4 ISR");
        pic::eoi_for(35);
    }));
    CONTEXT.idt.set_handler(36, make_idt_entry!(isr36, |_state| {
        // kprintln!(CONTEXT, "COM1/3 ISR enter");
        CONTEXT.com1.on_interrupt();
        // kprintln!(CONTEXT, "COM1/3 ISR exit");
        pic::eoi_for(36);
    }));

    kprintln!(CONTEXT, "Configuring COM1...");
    CONTEXT.com1.enable_interrupts();
    pic::enable_irq(4);

    kprintln!(CONTEXT, "Kernel initialized.");
    kprintln!(CONTEXT, "Pic mask: {:x}", pic::get_mask());
    kprintln!(CONTEXT, "Enabling interrupts.");
    CONTEXT.idt.enable_interrupts();

    // let mut main_thread = Scheduler::new();
    //main_thread.create_thread("echo", echo, 0);
    //main_thread.create_thread("clock", clock, 0);
    //main_thread.create_thread("keyboard", keyboard, 0);

    kprintln!(CONTEXT, "Beginning main loop.");
    // main_thread.run();

    let mut last_displayed = 0;
    loop
    {
        //kprintln!(CONTEXT, "Reading COM1.");
        while let Some(b) = CONTEXT.com1.try_receive() {
            match b as char {
                 'Q' => {
                    shutdown();
                },
                _ => kprint!(CONTEXT, "{}", b as char),
            };
            
            while CONTEXT.com1.try_write(b) != Ok(()) { }
        }

        //kprintln!(CONTEXT, "Checking timer.");
        let ticks = CONTEXT.ticks();
        if ticks - last_displayed > 0 {
            kprint_header!(CONTEXT, "ticks: {}\n", ticks);
            last_displayed = ticks;
        }

        //kprintln!(CONTEXT, "Checking keyboard.");
        while let Some(b) = CONTEXT.keyboard.try_dequeue() {
            while CONTEXT.com1.try_write(b as u8) != Ok(()) { }
            match b {
                'Q' => {
                    shutdown();
                },
                _ => {
                    kprint!(CONTEXT, "{}", b);
                }
            };
        }

        unsafe {
            x86::shared::halt();
        }
    }
}

fn shutdown() {
    unsafe {
        // https://wiki.osdev.org/Shutdown
        // In newer versions of QEMU, you can pass -device isa-debug-exit,iobase=0xf4,iosize=0x04 on the command-line, and do: 
        x86::shared::io::outb(0xf4, 0x00);  
    }
}

pub fn echo(ctxt: &mut ThreadContext, _arg: usize) {
    loop { 
        while let Some(b) = CONTEXT.com1.try_receive() {
            match b as char {
                _ => kprint!(CONTEXT, "{}", b as char),
            };
            
            while CONTEXT.com1.try_write(b) != Ok(()) { }
        }
        ctxt.yield_to();
    }
}

pub fn clock(ctxt: &mut ThreadContext, _arg: usize) {
    let mut last_displayed = 0;
    loop { 
        let ticks = CONTEXT.ticks();
        if ticks - last_displayed > 00 {
            kprint_header!(CONTEXT, "ticks: {}\n", ticks);
            last_displayed = ticks;
        }
        kprintln!(CONTEXT, "Yielding from clock.");
        ctxt.yield_to();
        kprintln!(CONTEXT, "Returned to clock.");

    }
}

pub fn keyboard(ctxt: &mut ThreadContext, _arg: usize) {
    loop { 
        while let Some(b) = CONTEXT.keyboard.try_dequeue() {
            while CONTEXT.com1.try_write(b as u8) != Ok(()) { }
            match b {
                'Q' => {
                    unsafe {
                        // https://wiki.osdev.org/Shutdown
                        // In newer versions of QEMU, you can pass -device isa-debug-exit,iobase=0xf4,iosize=0x04 on the command-line, and do: 
                        x86::shared::io::outb(0xf4, 0x00);  
                    }
                }
                _ => {
                    kprint!(CONTEXT, "{}", b);
                }
            };
        }
        ctxt.yield_to();
    }
}