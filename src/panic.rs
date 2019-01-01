use ::CONTEXT;
use core::panic::PanicInfo;

#[panic_handler]
#[no_mangle]
pub fn panic(info: &PanicInfo) -> ! {
    kprintln!(CONTEXT, "KERNEL PANIC: {:?}", info);
    loop {}
}
