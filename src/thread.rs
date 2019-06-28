extern crate core;
extern crate interrupts;
extern crate x86;

use ::CONTEXT;

#[repr(C)]
#[repr(align(16))]
#[derive(Debug)]
pub struct ThreadState {
    regs: interrupts::ProcessorState,
    rip: *mut usize,
}

const STACK_SIZE : usize = 32 * 1024;

#[repr(C)]
#[repr(align(16))]
pub struct Thread {
    state_ptr: *mut ThreadState,
    pub id: usize,
    stack: [u8; STACK_SIZE],
    pub name: &'static str,
}

pub struct ThreadContext<'a> {
    prev_thread: &'a mut Thread,
    this_thread: &'a mut Thread,
}

impl<'a> ThreadContext<'a> {
    pub fn yield_to(&mut self) -> () {
        self.this_thread.switch_to(self.prev_thread);
    }
}

type ThreadFunc = fn(&mut ThreadContext, usize)->();

struct FnPtr {
    pub f: ThreadFunc,
}

impl core::fmt::Debug for Thread {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        let rip = if self.state_ptr as usize == 0 { 0 } else { self.state().rip as usize};
        write!(f, "[Id:{:x}:{} &ctxt:{:x} rip:{:x}]", self.id, self.name, self.state_ptr as usize, rip)
    }
}

impl Thread {
    pub fn new(id: usize, name: &'static str) -> Thread {
        Thread {
            state_ptr: core::ptr::null_mut(),
            id: id,
            stack: [0x0u8; STACK_SIZE],
            name: name,
        }    
    }

    fn state_mut(&mut self) -> &mut ThreadState {
        unsafe {
            &mut *self.state_ptr
        }
    }

    fn state(&self) -> &ThreadState {
        unsafe {
            &*self.state_ptr
        }
    }

    fn prepare(&mut self, prev_thread: &mut Thread, f: ThreadFunc, arg: usize) {
        unsafe {
            let stack_needed = core::mem::size_of::<ThreadState>() as usize;
            let stack_needed = ((stack_needed + 15) / 16) * 16; // round up
            let stack_start_offset = (self.stack.len() as usize) - stack_needed;
            self.state_ptr = (&mut self.stack[0] as *mut u8).offset(stack_start_offset as isize) as *mut ThreadState;
            self.state_mut().regs.rsp = core::ptr::null_mut();
            self.state_mut().regs.rdi = f as *mut usize;
            self.state_mut().regs.rsi = prev_thread as *const Thread as *mut usize;
            self.state_mut().regs.rdx = self as *mut Thread as *mut usize;
            self.state_mut().regs.rcx = arg as *mut usize;
            self.state_mut().rip = Thread::thread_start as *mut usize;
        }
        kprintln!(CONTEXT, "Prepare: {:?}", self);
    }

    #[naked]
    #[inline(never)]
    unsafe extern "sysv64" fn thread_start(f_ptr: FnPtr, prev_thread: &mut Thread, this_thread: &mut Thread, arg: usize) {
        // let ip = f_ptr.f as *const u8;
        kprintln!(CONTEXT, "thread_start arg:{:x} prev:{:?} current:{:?}", arg, prev_thread, this_thread);
        let mut context = ThreadContext
        {
            prev_thread: prev_thread,
            this_thread: this_thread,
        };
        (f_ptr.f)(&mut context, arg);
        unreachable!("Thread over!");
    }

    // Note: the calling convention seems to be ignored for x64
    // and is always https://en.wikipedia.org/wiki/X86_calling_conventions#System_V_AMD64_ABI
    // RDI, RSI, RDX, RCX, R8, R9
    // If the callee wishes to use registers RBP, RBX, and R12–R15, 
    // it must restore their original values before returning control to the caller. 
    // All other registers must be saved by the caller if it wishes to preserve their values.[

    #[naked]
    #[inline(never)]
    pub extern "sysv64" fn switch_to(&mut self, next: &Thread) {
        // ::toggle_single_step();
        unsafe {
            kprintln!(CONTEXT, "switch_to enter cur:{:?} next:{:?} next.regs:{:?}", self, next, *next.state_ptr);
            
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

                mov rax, rsp
                push rax // capture rsp

                // everything is now stored
                // save the stack pointer
                mov [rdi], rsp

                // switch to the other stack
                mov rsp, [rsi]

                // null out context ptr as we are now active in that context
                xor rax, rax
                mov [rsi], rax 

                pop rax // skip dummy rsp

                // // restore state
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

                int 3

                ret
                " 
                : // no outputs
                : "{rdi}"(&self.state_ptr), "{rsi}"(&next.state_ptr)//, s"(body as fn()) 
                : // no clobbers
                : "volatile", "intel");
            
        }
        
        kprintln!(CONTEXT, "switch_to exit cur:{:?} next:{:?}", &self, next); 
    }
}

pub struct Scheduler {
    free_index: usize,
    scheduler_thread: Thread,
    threads: [Option<Thread>; 8],
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            free_index: 0,
            scheduler_thread: Thread::new(1, "main"),
            threads: [None, None, None, None, None, None, None, None]
        }
    }

    pub fn create_thread(&mut self, name: &'static str, f: ThreadFunc, arg: usize) -> usize {
        assert!(self.free_index < 8);
        let new_id = self.free_index + 2;
        let t = Thread::new(new_id, name);
        self.threads[self.free_index] = Some(t);
        self.threads[self.free_index].as_mut().unwrap().prepare(&mut self.scheduler_thread, f, arg);
        self.free_index += 1;
        new_id
    }

    pub fn run(&mut self) -> ! {
        loop {
            for t in self.threads.into_iter() {
                if let Some(t) = t {
                    kprintln!(CONTEXT, "Switching to thread {}: {}", t.id, t.name);
                    //::toggle_single_step();
                    self.scheduler_thread.switch_to(&t);
                    // ::toggle_single_step();
                } else {
                    break;
                }
            }
            unsafe {
                x86::shared::halt();
            }
        }
    }
}