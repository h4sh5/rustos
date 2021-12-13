use crate::vga_buffer::{BUFFER_WIDTH};
use crate::strutils::{strcmpl};
use crate::{print, println, OSINFO};


pub const PROMPT: char = '>';

// pub fn bytes2chars(bs: &mut [char; BUFFER_WIDTH], chars: &mut [char; BUFFER_WIDTH]) {
// 	for i in 0..BUFFER_WIDTH {
// 		arr[i] = s[i];
// 	}
// }

pub fn handle_cmd(input: &[char; BUFFER_WIDTH]) {

	if strcmpl(input, "help", 4) {
		println!(concat!(
    		"help: show help\n",
    		"break: trigger breakpoint (c3)\n",
    		"pagefault: trigger pagefault\n",
    		"memregions: show mem regions from boot info\n",
    		)
		)

	}

	if strcmpl(input, "break", 5) {
		//insert breakpoint, should trigger interrupt
    	x86_64::instructions::interrupts::int3(); 
	}

	if strcmpl(input, "pagefault", 9) {
		// trigger a page fault
	    unsafe {
	        *(0xdeadbeef as *mut u64) = 42;
	    };

	}

	if strcmpl(input, "bootinfo", "bootinfo".chars().count()) {
		// read memory regions
		println!("{:?}", OSINFO.lock().bootinfo);

	}




}