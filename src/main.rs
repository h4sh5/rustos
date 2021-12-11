#![no_std]
#![no_main]

use core::panic::PanicInfo;
use hash_os::println;


#[no_mangle]
pub extern "C" fn _start() -> ! {

    hash_os::init(); // init os

    println!("Hello World{}", "!");

    // insert breakpoint, should trigger interrupt
    x86_64::instructions::interrupts::int3(); 

    println!("It did not crash!");

    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
