#![feature(asm)]
#![no_std]

use core::fmt;
use core::fmt::Write;

extern crate spin;
use spin::{Mutex};

extern crate x86;
use x86::shared::io::{inb, outb};

extern crate common;
use common::{Queue,InterruptData};

pub const COM1 : u16 = 0x3F8;

struct SerialPortRaw {
    base_address: u16,
    in_queue: Queue<u8>,
    out_queue: Queue<u8>,
}

impl SerialPortRaw {
    pub unsafe fn read_in_bytes(&mut self) {
        loop {
            let b = inb(self.base_address);
            self.in_queue.enqueue(b).unwrap();
            if inb(self.base_address + SerialPort::LSR) & 0x01 == 0x0 {
                break;
            }
        }
    }

    pub unsafe fn write_out_bytes(&mut self) {
        while inb(self.base_address + SerialPort::LSR) & 0x02 != 0x0 {
            if let Some(b) = self.out_queue.try_dequeue() {
                outb(self.base_address, b);
            } else {
                break;
            }
        }
    }

    pub fn set_baud_rate_internal<'a>(&self, baud_rate: u32) {
        let divisor = 115200 / baud_rate;
        unsafe {
            let r = inb(self.base_address + SerialPort::LCR);
            outb(self.base_address + SerialPort::LCR, 0x80 | r);
            outb(self.base_address, (divisor & 0xFF) as u8);
            outb(self.base_address + 1, (divisor >> 8) as u8);
            outb(self.base_address + SerialPort::LCR, 0x7F | r);
        }
    }

}

pub struct SerialPort {
    raw: InterruptData<Mutex<SerialPortRaw>>
}

impl SerialPort {
    pub fn create(base_address: u16) -> SerialPort {
        SerialPort { 
            raw: InterruptData::new(Mutex::new(
                SerialPortRaw {
                    base_address: base_address,
                    in_queue: Queue::new(),
                    out_queue: Queue::new(),
                }))
        }
    }

    // See https://en.wikibooks.org/wiki/Serial_Programming/8250_UART_Programming#UART_Registers
    const IER: u16 = 1;
    const IIR: u16 = 2;
    const FCR: u16 = 2;
    const LCR: u16 = 3;
    const LSR: u16 = 5;

    pub fn init(&self) {
        let port = self.raw.enter();
        let port = port.lock();
        unsafe {
            outb(port.base_address + SerialPort::IER, 0); // disable interrupts
            outb(port.base_address + SerialPort::LCR, 3); // 8N1
            port.set_baud_rate_internal(115200);
            // outb(port.base_address + SerialPort::FCR, 0);
            
            outb(port.base_address + SerialPort::FCR, 
                0x01 | // enable FIFO
                0x06 | // empty rx tx FIFOs
                0x20 | // enable 64-byte FIFO
                0x80   // 32-byte trigger
                );
        }
    }

    pub fn enable_interrupts(&self) {
        let port = self.raw.enter();
        let port = port.lock();
        unsafe {
            outb(port.base_address + SerialPort::IER, 
                 0x3);  // received data, transmit empty
        }
    }       

    pub fn set_baud_rate(&self, baud_rate: u32) -> () {
        let port = self.raw.enter();
        let port = port.lock();
        port.set_baud_rate_internal(baud_rate);
    }

    pub fn try_receive(&self) -> Option<u8> {
        let port = self.raw.enter();
        let mut port = port.lock();
        port.in_queue.try_dequeue()
    }

    pub fn try_write(&self, b: u8) -> Result<(),()> {
        let port = self.raw.enter();
        let mut port = port.lock();
        if port.out_queue.count > 0 {
            return port.out_queue.enqueue(b);
        }

        unsafe {
            if inb(port.base_address + SerialPort::LSR) & 0x20 == 0x20 {
            outb(port.base_address, b);
                Ok(())
            } else {
                Err(())    
            }
        }
    }

    pub fn on_interrupt(&self) -> () {
        let port = self.raw.enter();
        let mut port = port.lock();
        loop {
            unsafe {
                let flags = inb(port.base_address + SerialPort::IIR);
                if flags & 0x1 == 1 {
                    break;
                }

                match (flags >> 1) & 0x7 {
                    1 => {
                        port.write_out_bytes();
                    },
                    2 | 6  => {
                        port.read_in_bytes();
                    },
                    i => panic!("Unknown interrupt source: {:x}", i),
                }
            }
        }
    }
}

pub struct SerialPortWriter<'a> {
    port: &'a SerialPort
}

impl<'a> SerialPortWriter<'a> {
    pub fn from_port(port: &SerialPort) -> SerialPortWriter {
        SerialPortWriter {
            port: port
        }
    }
}

impl<'a> Write for SerialPortWriter<'a> {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        for b in s.bytes() {
            while Err(()) == self.port.try_write(b) { 
                self.port.on_interrupt();
            }
        }

        Ok(())
    }
}