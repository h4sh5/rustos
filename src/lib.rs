#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(asm)]

pub mod gdt;
pub mod interrupts;
// pub mod serial;
pub mod vga_buffer;
pub mod cmd;
pub mod strutils;
pub mod memory;

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

/// read msr , 11 is if APIC is supported 
/// http://perfinsp.sourceforge.net/msr.html
pub fn get_msr(msr:u32) {
    let lo:u32;
    let hi:u32;
    unsafe {
        asm!(
            // "mov rcx, {0}",
            // rdmsr rcx, rax, rdi  ; rcx is the input, rax (eax) and rdi (edx) are outputs
            "rdmsr",
            // "mov {1}, eax",
            // "mov {2}, edx",
            // "add {0}, {number}",
            in("rcx") msr,
            out("eax") lo,
            out("edx") hi,
            options(nostack)
            // out(reg) lo,
            // out(reg) hi,
            
        );
    }
    println!("{:#08x}", (hi|lo) as u64);
    println!("{:#064b}", (hi|lo) as u64);

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


    //------------- memory stuff
    use x86_64::VirtAddr;
    use x86_64::structures::paging::Page;
    use x86_64::structures::paging::PageTableFlags as Flags;
    use crate::memory::{self, BootInfoFrameAllocator};
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };


    // map an unused (virtual) page
    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    //-------------

    // mapping back the kernel page
    memory::create_mapping(page, &mut mapper, &mut frame_allocator, 0x200000, 0x240000, Flags::PRESENT);


    print!("{}", crate::cmd::PROMPT);

    
    hlt_loop();
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