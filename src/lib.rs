#![no_std]
#![feature(abi_x86_interrupt)]


// pub mod gdt;
pub mod interrupts;
// pub mod serial;
pub mod vga_buffer;


pub fn init() {
    // gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() }; // new
    x86_64::instructions::interrupts::enable();     // should be sti - enable interrupt
}