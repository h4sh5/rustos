
// #![no_std]

use crate::{gdt, print, println};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::vga_buffer::{backspace, cursor_left, cursor_right, 
    WRITER, BUFFER_WIDTH};

use crate::cmd::{PROMPT, handle_cmd};
use crate::OSINFO;
use crate::hlt_loop;

use core::result::Result::Ok;
use core::option::Option::Some;

pub const PIC_1_OFFSET: u8 = 32;  /// 0x20 (anything > 0x20 belongs to the APIC)
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8; // keyboard: 40 - 0x28

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}


impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = { /// there are 255 entries
        let mut idt = InterruptDescriptorTable::new();

        /// set up every IRQ w/ a default handler
        for i in 0x21..0xff {
            idt[i].set_handler_fn(default_exception_handler);
        }

        // just for some hardware
        idt[44].set_handler_fn(int_44_handler);

        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);



        // rest of the handlers to prevent double faults
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.non_maskable_interrupt.set_handler_fn(non_maskable_interrupt_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded.set_handler_fn(bound_range_exceeded_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available.set_handler_fn(device_not_available_handler);
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        idt.x87_floating_point.set_handler_fn(x87_floating_point_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.simd_floating_point.set_handler_fn(simd_floating_point_handler);
        idt.virtualization.set_handler_fn(virtualization_handler);
        // idt.vmm_communication_exception.set_handler_fn(vmm_communication_exception_handler);
        idt.security_exception.set_handler_fn(security_exception_handler);

        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn default_exception_handler(_stack_frame: InterruptStackFrame) {
    // println!("default_exception_handler");
}

extern "x86-interrupt" fn int_44_handler(_stack_frame: InterruptStackFrame) {
    println!("interrupt 44");
}


extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT (code {})\n{:#?}",error_code, stack_frame);
}



extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    // crate::kernel_main(OSINFO.lock().bootinfo);

    // read from it to dump some bytes
    // TODO: convert this dump functionality into a macro?
    unsafe {
        for i in 0..32 {
            let ptr = stack_frame.instruction_pointer.as_u64() + i;
            print!("{:#04x} ", *(ptr as *mut u8));
        }
    }
    hlt_loop();
}


/// dfeault interrupt handler just to prevent segment not present exceptions
extern "x86-interrupt" fn default_interrupt_handler(_stack_frame: InterruptStackFrame) {

}

/// timer handler, maybe shouldn't do anything?
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!("_");// 0x8 is backspace
    for _i in 0..20000 { // to generate "blink" effect!

    }

    backspace(true);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}


/// read keyboard input and do stuff (selector/entry 40)
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyCode};
    use spin::Mutex;
    use x86_64::instructions::port::Port;
    

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
        );
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    // scancode 14 is backspace and 83 is delete
    // unicde 0x08 is backspace and 0x7f is delete
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {

        if let Some(key) = keyboard.process_keyevent(key_event) {
            // print!("{:?}{:?} ", scancode, key);
            // if scancode == 

            match key {
                DecodedKey::Unicode(character)  => {

                    // TODO: handle backspace and del properly in vga_buffer
                    if character == '\x08' {
                        // print!("[BAK]");
                        backspace(true);
                    } else if character == '\x7f'  {
                        // print!("[DEL]");
                        backspace(false);
                    } else if character == '\n' {
                        // run command
                        let mut line: [char; BUFFER_WIDTH] = [0 as char;80];
                        WRITER.lock().get_prev_line(&mut line);
                        
                        println!();
                        print!("<");
                        for c in line {
                           
                            if c != '\0' {
                                 print!("{}", c);
                            }

                        }
                        handle_cmd(&line);
                        print!("{}",PROMPT);
                    } else {
                        
                        print!("{}", character);
                    }
                }

                DecodedKey::RawKey(key) => {
                    match key {
                        KeyCode::ArrowLeft => cursor_left(),
                        KeyCode::ArrowRight => cursor_right(),
                        _=> print!("({:?})", key),
                    }
                    

                }
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}


extern "x86-interrupt" fn divide_error_handler (
    stack_frame: InterruptStackFrame) {

    println!("EXCEPTION: Divide Error");
    println!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn debug_handler (
    stack_frame: InterruptStackFrame) {

    println!("EXCEPTION: Debug Error");
    println!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn non_maskable_interrupt_handler (
    stack_frame: InterruptStackFrame) {

    println!("EXCEPTION: non_maskable_interrupt Error");
    println!("{:#?}", stack_frame);
}


extern "x86-interrupt" fn overflow_handler (
    stack_frame: InterruptStackFrame) {

    println!("EXCEPTION: Overflow Error");
    println!("{:#?}", stack_frame);
}


extern "x86-interrupt" fn bound_range_exceeded_handler (
    stack_frame: InterruptStackFrame) {

    println!("EXCEPTION: bound_range_exceeded");
    println!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn invalid_opcode_handler (
    stack_frame: InterruptStackFrame) {

    println!("EXCEPTION: invalid_opcode");
    println!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn device_not_available_handler (
    stack_frame: InterruptStackFrame) {

    println!("EXCEPTION: device_not_available");
    println!("{:#?}", stack_frame);
}



extern "x86-interrupt" fn invalid_tss_handler (
    stack_frame: InterruptStackFrame, ec:u64) {

    println!("EXCEPTION: invalid_tss: errcode {}",ec);
    println!("{:#?}", stack_frame);
}



extern "x86-interrupt" fn segment_not_present_handler (
    stack_frame: InterruptStackFrame, ec:u64) {

    println!("EXCEPTION: segment_not_present: errcode {}", ec);
    println!("External?:{} Table:{} Index:{}", ec & 1, ec & 0b110, (ec & 0b111111000) >> 3);
    println!("{:#?}", stack_frame);
    // read from it to dump some bytes
    unsafe {
        for i in 0..32 {
            let ptr = stack_frame.instruction_pointer.as_u64() + i;
            print!("{:#04x} ", *(ptr as *mut u8));
        }
    }
    // restore keyboard and timer interrupts

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
    //restart?
    crate::kernel_main(OSINFO.lock().bootinfo);


}


extern "x86-interrupt" fn stack_segment_fault_handler (
    stack_frame: InterruptStackFrame, ec:u64) {

    println!("EXCEPTION: stack_segment_fault: errcode {}", ec);
    println!("{:#?}", stack_frame);
}


extern "x86-interrupt" fn general_protection_fault_handler (
    stack_frame: InterruptStackFrame, ec:u64) {

    println!("EXCEPTION: general_protection_fault errcode {}", ec);
    println!("{:#?}", stack_frame);
}


extern "x86-interrupt" fn x87_floating_point_handler (
    stack_frame: InterruptStackFrame) {

    println!("EXCEPTION: x87_floating_point");
    println!("{:#?}", stack_frame);
}


extern "x86-interrupt" fn alignment_check_handler (
    stack_frame: InterruptStackFrame, ec:u64) {

    println!("EXCEPTION: alignment_check errcode {}", ec);
    println!("{:#?}", stack_frame);
}




extern "x86-interrupt" fn simd_floating_point_handler (
    stack_frame: InterruptStackFrame) {

    println!("EXCEPTION: simd_floating_point");
    println!("{:#?}", stack_frame);
}


extern "x86-interrupt" fn virtualization_handler (
    stack_frame: InterruptStackFrame) {

    println!("EXCEPTION: virtualization");
    println!("{:#?}", stack_frame);
}

// extern "x86-interrupt" fn vmm_communication_exception_handler (
//     stack_frame: InterruptStackFrame) {

//     println!("EXCEPTION: vmm_communication_exception");
//     println!("{:#?}", stack_frame);
// }

extern "x86-interrupt" fn security_exception_handler (
    stack_frame: InterruptStackFrame, ec:u64) {

    println!("EXCEPTION: security_exception errcode {}", ec);
    println!("{:#?}", stack_frame);
}

// #[test_case]
// fn test_breakpoint_exception() {
//     // invoke a breakpoint exception
//     x86_64::instructions::interrupts::int3();
// }