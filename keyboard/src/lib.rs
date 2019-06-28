#![no_std]

extern crate common;
use common::{InterruptData,Queue};

extern crate spin;
use spin::Mutex;

extern crate x86;
use x86::shared::io::{inb};

#[derive(Clone,Copy)]
pub struct ScanCode(u8);

struct KeyboardData {
    in_queue: Queue<char>,
    lshift: bool,
    rshift: bool,
}

impl KeyboardData {
    pub fn new() -> KeyboardData {
        KeyboardData {
            in_queue: Queue::<char>::new(),
            lshift: false,
            rshift: false,
        }
    }

    /// Decode a code in the PS/2 scan code set 1 (legacy set).
    ///
    /// Difference between set 1 and sets 2 & 3:
    ///   http://wiki.osdev.org/%228042%22_PS/2_Controller#Translation
    ///
    /// Reference table:
    ///   http://www.computer-engineering.org/ps2keyboard/scancodes1.html
    fn from_scancode(&self, code: ScanCode) -> Option<char> {
        let mut printable = match code.0 {
            0x1e => 'a',
            0x30 => 'b',
            0x2e => 'c',
            0x20 => 'd',
            0x12 => 'e',
            0x21 => 'f',
            0x22 => 'g',
            0x23 => 'h',
            0x17 => 'i',
            0x24 => 'j',
            0x25 => 'k',
            0x26 => 'l',
            0x32 => 'm',
            0x31 => 'n',
            0x18 => 'o',
            0x19 => 'p',
            0x10 => 'q',
            0x13 => 'r',
            0x1f => 's',
            0x14 => 't',
            0x16 => 'u',
            0x2f => 'v',
            0x11 => 'w',
            0x2d => 'x',
            0x15 => 'y',
            0x2c => 'z',
            0x0b => '0',
            0x02 => '1',
            0x03 => '2',
            0x04 => '3',
            0x05 => '4',
            0x06 => '5',
            0x07 => '6',
            0x08 => '7',
            0x09 => '8',
            0x0a => '9',
            0x29 => '`',
            0x0c => '-',
            0x0d => '=',
            28 => '\n',
            0x2b => '\\',
            0x39 => ' ',
            0x1a => '[',
            0x1b => ']',
            0x27 => ';',
            0x28 => '\'',
            0x33 => ',',
            0x34 => '.',
            0x35 => '/',
            0x37 => '*', // Keypad
            0x4a => '-', // Keypad
            0x4e => '+', // Keypad
            _ => { return None; }
        };

        if self.lshift || self.rshift {
            printable = match printable {
                a @ 'a' ..= 'z' => (a as u8 - ('a' as u8 - 'A' as u8)) as char,
                a @ _ => a,
            };
        }

        Some(printable)
    }
}

pub struct Keyboard {
    data: InterruptData<Mutex<KeyboardData>>
}

impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard {
            data: InterruptData::new(Mutex::new(KeyboardData::new()))
        }
    }

    fn get_hex(i: u8) -> char {
        (match i {
            0 ..= 9 => i + ('0' as u8),
            10 ..= 15 => i - 10 + ('A' as u8),
            _ => panic!("Not a hex number!")
        }) as char
    }

    pub fn isr(&self) {
        let data = self.data.enter();
        let mut data = data.lock();
        
        let scancode = ScanCode(unsafe { inb(0x60) });
        if let Some(c) = data.from_scancode(scancode) {
            let _result = data.in_queue.enqueue(c);
        } else if scancode.0 == 0x2a {
            data.lshift = true;
        } else if scancode.0 == 0x36 {
            data.rshift = true;
        } else if scancode.0 == 0xaa {
            data.lshift = false;
        } else if scancode.0 == 0xb6 {
            data.rshift = false;
        } else if scancode.0 > 0x7f {
            // ignore other key releases
        } else {
            let _result = data.in_queue.enqueue('[');
            let _result = data.in_queue.enqueue(Keyboard::get_hex(scancode.0 / 16));
            let _result = data.in_queue.enqueue(Keyboard::get_hex(scancode.0 % 16));
            let _result = data.in_queue.enqueue(']');            
        }
    }

    pub fn try_dequeue(&self) -> Option<char> {
        let data = self.data.enter();
        let mut data = data.lock();

        data.in_queue.try_dequeue()
    }
}
