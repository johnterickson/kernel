#![no_std]
#![feature(lang_items)]

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // writeln!(host_stderr, "{}", info).ok();
    loop {}
}

#[lang = "eh_personality"] extern fn rust_eh_personality() {}

// // Define a function that is imported into the module.
// // By default, the "env" namespace is used.
// extern "C" {
//     fn print_str(ptr: *const u8, len: usize);
// }

// // Define a string that is accessible within the wasm
// // linear memory.
// static HELLO: &'static str = "Hello, World!";

// // Export a function named "hello_wasm". This can be called
// // from the embedder!
// #[no_mangle]
// pub extern fn hello_wasm() {
//     // Call the function we just imported and pass in
//     // the offset of our string and its length as parameters.
//     unsafe {
//       print_str(HELLO.as_ptr(), HELLO.len());
//     }
// }

// Export a function named "hello_wasm". This can be called
// from the embedder!
#[no_mangle]
pub extern fn wasm_add(a: i32, b: i32) -> i32 {
    a + b
}