//! This module contains methods and macros to create and register interrupt descriptors and
//! interrupt handlers

#![feature(asm)]
#![feature(naked_functions)]
#![feature(const_fn)]
#![no_std]

extern crate x86;
extern crate pic;
extern crate spin;

use spin::Mutex;
use x86::shared::dtables;
use x86::shared::dtables::DescriptorTablePointer;
use x86::bits64::irq::IdtEntry;

#[repr(C)]
#[derive(Debug)]
pub struct ProcessorState {
    pub rsp: *mut usize,
    pub rax: *mut usize,
    pub rbx: *mut usize,
    pub rcx: *mut usize,
    pub rdx: *mut usize,
    pub rdi: *mut usize,
    pub rsi: *mut usize,
    pub r8: *mut usize,
    pub r9: *mut usize,
    pub r10: *mut usize,
    pub r11: *mut usize,
    pub r12: *mut usize,
    pub r13: *mut usize,
    pub r14: *mut usize,
    pub r15: *mut usize,
    pub rbp: *mut usize,
}

impl Default for ProcessorState {
    fn default() -> Self {
        ProcessorState {
            rsp: 0 as *mut usize,
            rax: 0 as *mut usize,
            rbx: 0 as *mut usize,
            rcx: 0 as *mut usize,
            rdx: 0 as *mut usize,
            rdi: 0 as *mut usize,
            rsi: 0 as *mut usize,
            r8: 0 as *mut usize,
            r9: 0 as *mut usize,
            r10: 0 as *mut usize,
            r11: 0 as *mut usize,
            r12: 0 as *mut usize,
            r13: 0 as *mut usize,
            r14: 0 as *mut usize,
            r15: 0 as *mut usize,
            rbp: 0 as *mut usize,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct InterruptState {
    pub regs: ProcessorState,
    pub rip: *mut usize,
    pub cs: *mut usize,
    pub flags: *mut usize,
}

/// Creates an IDT entry.
///
/// Creates an IDT entry that executes the expression in `body`.
#[macro_export]
macro_rules! make_idt_entry {
    ($name:ident, $body:expr) => {{
        extern "C" fn body(state: &mut interrupts::InterruptState) {
            $body(state)
        }

        #[naked]
        unsafe extern fn $name() {
            asm!("
                  push rbp
                  push r15
                  push r14
                  push r13
                  push r12
                  push r11
                  push r10
                  push r9
                  push r8
                  push rsi
                  push rdi
                  push rdx
                  push rcx
                  push rbx
                  push rax
                  mov rdi, rsp
                  push rdi
                  sub rdi, 8
                  
                  cli

                  call $0

                  sti

                  add rsp, 8
                  pop rax
                  pop rbx
                  pop rcx
                  pop rdx
                  pop rdi
                  pop rsi
                  pop r8
                  pop r9
                  pop r10
                  pop r11
                  pop r12
                  pop r13
                  pop r14
                  pop r15
                  pop rbp
                  iretq" 
                  : // no outputs 
                  : "s"(body as extern "C" fn(&mut interrupts::InterruptState)) 
                  : // no clobbers
                  : "volatile", "intel");
            intrinsics::unreachable();
        }

        use x86::shared::paging::VAddr;
        use x86::shared::PrivilegeLevel;

        let handler = VAddr::from_usize($name as usize);

        // last is "block", idk
        IdtEntry::new(handler, 0x8, PrivilegeLevel::Ring0, false)
    }};
}

/// The Interrupt Descriptor Table
///
/// The CPU will look at this table to find the appropriate interrupt handler.
//static IDT: Mutex<[IdtEntry; 256]> = Mutex::new([IdtEntry::MISSING; 256]);

pub struct Idt {
    entries: Mutex<[IdtEntry; 256]>,
}

impl Idt {
    pub fn new() -> Idt {
        Idt {
            entries: Mutex::new([IdtEntry::MISSING; 256]),
        }
    }
}

/// Pointer to the Interrupt Descriptor Table
pub struct IdtRef<'a> {
    ptr: DescriptorTablePointer<IdtEntry>,
    idt: &'a Idt,
}

unsafe impl<'a> Send for IdtRef<'a> {}
unsafe impl<'a> Sync for IdtRef<'a> {}

impl<'a> IdtRef<'a> {
    /// Creates a new pointer struct to the IDT.
    pub fn from_idt(idt: &'a Idt) -> IdtRef {
        let r = IdtRef {
            ptr: DescriptorTablePointer::new_idtp(&idt.entries.lock()[..]),
            idt: idt,
        };

        // This block is safe because by referencing IDT above, we know that we've constructed an
        // IDT.
        unsafe { dtables::lidt(&r.ptr) };

        r
    }

    /// Sets an IdtEntry as a handler for interrupt specified by `index`.
    pub fn set_handler(&self, index: usize, entry: IdtEntry) {
        self.idt.entries.lock()[index] = entry;
    }

    /// Enables interrupts.
    pub fn enable_interrupts(&self) {
        // This unsafe fn is okay becuase, by virtue of having an IdtRef, we know that we have a
        // valid Idt.
        unsafe {
            x86::shared::irq::enable();
        }
    }

      /// Enables interrupts.
    pub fn disable_interrupts(&self) {
        // This unsafe fn is okay becuase, by virtue of having an IdtRef, we know that we have a
        // valid Idt.
        unsafe {
            x86::shared::irq::disable();
        }
    }
}
