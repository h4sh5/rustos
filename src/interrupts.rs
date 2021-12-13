
// #![no_std]

use crate::{gdt, print, println};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::vga_buffer::{backspace, cursor_left, cursor_right, 
    WRITER, BUFFER_WIDTH};

use crate::cmd::{PROMPT, handle_cmd};
use crate::hlt_loop;

use core::result::Result::Ok;
use core::option::Option::Some;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

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
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
// {
    // println!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    
    // crate::_start();

    panic!("EXCEPTION: DOUBLE FAULT (code {})\n{:#?}",error_code, stack_frame);
    // crate::_start();
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
    hlt_loop();
}

// timer handler, maybe shouldn't do anything?
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

// #[test_case]
// fn test_breakpoint_exception() {
//     // invoke a breakpoint exception
//     x86_64::instructions::interrupts::int3();
// }