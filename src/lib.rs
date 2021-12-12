#![no_std]
#![feature(abi_x86_interrupt)]


// pub mod gdt;
pub mod interrupts;
// pub mod serial;
pub mod vga_buffer;
// pub mod main;

#[no_mangle]
pub extern "C" fn _start() -> ! {

    // hash_os::init(); // init os
    init();

    println!("Hello to #OS prerelease.\nThe QEMU escape key is {}", "ctrl-alt-G");

    // insert breakpoint, should trigger interrupt
    // x86_64::instructions::interrupts::int3(); 

    // println!("It did not crash!");

    loop {}
}

pub fn init() {
    // gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() }; // new
    x86_64::instructions::interrupts::enable();     // should be sti - enable interrupt
}