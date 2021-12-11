#![no_std]
#![no_main]

use core::panic::PanicInfo;
use hash_os::println;




/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
