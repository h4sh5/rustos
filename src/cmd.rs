use crate::vga_buffer::{BUFFER_WIDTH};
use crate::strutils::{strcmpl};
use crate::{println, OSINFO};

pub const PROMPT: char = '>';

// pub fn bytes2chars(bs: &mut [char; BUFFER_WIDTH], chars: &mut [char; BUFFER_WIDTH]) {
// 	for i in 0..BUFFER_WIDTH {
// 		arr[i] = s[i];
// 	}
// }

const IA32_APIC_BASE_MSR:u32 = 0x1B;

pub fn handle_cmd(input: &[char; BUFFER_WIDTH]) {

	if strcmpl(input, "help", 4) {
		println!(concat!(
    		"help: show help\n",
    		"break: trigger breakpoint (c3)\n",
    		"pagefault: trigger pagefault\n",
    		"bootinfo: show boot info\n",
    		// "snph: trigger segment_not_present_handler\n",
    		"msr_acpi: get MSR IA32_APIC_BASE_MSR\n",
    		"The QEMU escape key is Ctrl-Alt-G\n",
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

	// "bootinfo".chars().count() will generate count at compile time, found out by RE
	if strcmpl(input, "bootinfo", "bootinfo".chars().count()) {
		// read memory regions
		println!("{:?}", OSINFO.lock().bootinfo);

	}

	if strcmpl(input, "msr_acpi", "msr_acpi".chars().count()) {
		// read memory regions

		// println!("{:?}", OSINFO.lock().bootinfo);
		crate::get_msr(IA32_APIC_BASE_MSR);

	}

	
}