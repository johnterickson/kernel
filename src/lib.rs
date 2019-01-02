#![feature(asm)]
#![feature(const_fn)]
#![feature(naked_functions)]
#![no_std]

#[macro_export]
macro_rules! kprintln {
    ($ctx:ident, $fmt:expr) => (kprint!($ctx, concat!($fmt, "\n")));
    ($ctx:ident, $fmt:expr, $($arg:tt)*) => (kprint!($ctx, concat!($fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! kprint {
    ($ctx:ident, $($arg:tt)*) => ({
        use core::fmt::Write;
        let mut vga = $ctx.vga.lock();
        vga.write_fmt(format_args!($($arg)*)).unwrap();
        vga.flush();
    });
}

#[macro_export]
macro_rules! kprint_header {
    ($ctx:ident, $($arg:tt)*) => ({
        use core::fmt::Write;
        let mut vga = $ctx.vga.lock();
        let old_position = vga.position;
        vga.invert();
        vga.position = 0;
        vga.write_fmt(format_args!($($arg)*)).unwrap();
        vga.invert();
        vga.position = old_position;
        vga.flush();
    });
}
