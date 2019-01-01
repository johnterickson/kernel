#![feature(asm)]
#![no_std]

extern crate x86;

pub const COM1 : u16 = 0x3F8;

pub struct Queue<T : Copy+Default> {
    buffer: [T; 32],
    first_free_index: usize,
    pub count: usize,
}

impl<T> Queue<T> where T: Copy+Default {
    pub fn new() -> Queue<T> {
        Queue {
            buffer: [Default::default(); 32],
            first_free_index: 0,
            count: 0
        }
    }

    pub fn enqueue(&mut self, item: T) -> Result<(),()> {
        if self.count == self.buffer.len() {
            Err(())
        } else {
            self.buffer[self.first_free_index] = item;
            self.first_free_index = (self.first_free_index + 1) % self.buffer.len();
            self.count += 1;
            Ok(())
        }
    }

    fn first_value_index(&self) -> usize {
        (self.buffer.len() + self.first_free_index - self.count) % self.buffer.len()
    }

    pub fn try_dequeue(&mut self) -> Option<T> {
        if self.count == 0 {
            None
        } else {
            let item = self.buffer[self.first_value_index()];
            self.count -= 1;
            Some(item)
        }
    }
}

#[test]
fn queue_in_out() {
    let mut q = Queue::<u8>::new();

    assert_eq!(None, q.try_dequeue());

    for i in 1..8 {
        q.enqueue(i).unwrap();
    }
    for i in 1..8 {
        assert_eq!(i, q.try_dequeue().unwrap());
    }    

    assert_eq!(None, q.try_dequeue());

    for i in 1..8 {
        q.enqueue(i).unwrap();
    }
    for i in 1..8 {
        assert_eq!(i, q.try_dequeue().unwrap());
    }    

    assert_eq!(None, q.try_dequeue());
}

pub struct InterruptData<T> {
    data: T,
}

impl<T> InterruptData<T> {
    pub fn new(data: T) -> InterruptData<T> {
        InterruptData { data }
    }

    pub fn enter<'a>(&'a self) -> InterruptDataGuard<'a, T> {
        let previous_flags = x86::shared::flags::flags();
        let should_enable = previous_flags & x86::shared::flags::FLAGS_IF == x86::shared::flags::FLAGS_IF;
        if should_enable {
            unsafe {
                x86::shared::irq::disable();
            }
        }
        InterruptDataGuard { 
            should_enable: should_enable,
            inner: &self.data,
        }
    }
}

pub struct InterruptDataGuard<'a, T: 'a> {
    should_enable : bool,
    inner: &'a T,
}

impl<'a, T> core::ops::Deref for InterruptDataGuard<'a, T>
{
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T { &*self.inner }
}

impl<'a, T> Drop for InterruptDataGuard<'a, T> {
    fn drop(&mut self) {
        unsafe {
            if self.should_enable {
                x86::shared::irq::enable();
            }
        }
    }
}