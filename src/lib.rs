#![no_std]
#![feature(abi_x86_interrupt)]


pub mod gdt;
pub mod interrupts;
// pub mod serial;
pub mod vga_buffer;
pub mod cmd;
pub mod strutils;

use lazy_static::lazy_static;
use bootloader::{BootInfo,entry_point};
use spin::Mutex;



lazy_static! {

    pub static ref OSINFO: Mutex<OSInfoStore> = Mutex::new(OSInfoStore {
        bootinfo: unsafe { &mut *(0x0 as *mut BootInfo) },
    });

}

pub struct OSInfoStore {
    // TODO: track row position too?
    bootinfo: &'static BootInfo,
}


entry_point!(kernel_main);

#[no_mangle]
//pub extern "C" fn start() -> {
pub fn kernel_main(boot_info: &'static BootInfo) -> ! {

    // hash_os::init(); // init os
    init();

    println!("Hello to #OS prerelease.\nThe QEMU escape key is {}", "ctrl-alt-G");


    use x86_64::registers::control::Cr3;
    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at: {:?}", level_4_page_table.start_address());

    OSINFO.lock().bootinfo = &boot_info;
    // println!("boot info: {:#?}", boot_info);

    // insert breakpoint, should trigger interrupt
    // x86_64::instructions::interrupts::int3(); 

    // println!("It did not crash!");
    print!("{}", crate::cmd::PROMPT);

    
    loop {}
}


pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() }; // new
    x86_64::instructions::interrupts::enable();     // should be sti - enable interrupt
}