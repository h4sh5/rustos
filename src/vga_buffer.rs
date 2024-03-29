use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

lazy_static! {
    /// A global `Writer` instance that can be used for printing to the VGA text buffer.
    ///
    /// Used by the `print!` and `println!` macros.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::LightGreen, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });

}

/// The standard color palette in VGA text mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// A combination of a foreground and a background color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    /// Create a new `ColorCode` with the given foreground and background colors.
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// A screen character in the VGA text buffer, consisting of an ASCII character and a `ColorCode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// The height of the text buffer (normally 25 lines).
pub const BUFFER_HEIGHT: usize = 25;
/// The width of the text buffer (normally 80 columns).
pub const BUFFER_WIDTH: usize = 80;

/// A structure representing the VGA text buffer.
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// A writer type that allows writing ASCII bytes and strings to an underlying `Buffer`.
///
/// Wraps lines at `BUFFER_WIDTH`. Supports newline characters and implements the
/// `core::fmt::Write` trait.
pub struct Writer {
    // TODO: track row position too?
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    /// Writes an ASCII byte to the buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character.
    /// #: doesn't need to write every single char (refresh display) again 
    /// because those characters will already be on the screen/device mem
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    /// Writes the given ASCII string to the buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character. Does **not**
    /// support strings with non-ASCII characters, since they can't be printed in the VGA text
    /// mode.
    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Shifts all lines one line up and clears the last row.
    /// interpret line input if a command matches, by storing the last line
    fn new_line(&mut self) {
        
        // this nested loop shifts all lines up line up
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    pub fn get_prev_line(&self,  line: &mut [char; BUFFER_WIDTH]) {
        // let line: [char; BUFFER_WIDTH];

        for col in 1..BUFFER_WIDTH {
            line[col-1] = self.buffer.chars[BUFFER_HEIGHT-1][col].read().ascii_character as char;
            // self.buffer.chars[row - 1][col].write(character);
        }
    }

    /// Clears a row by overwriting it with blank characters.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    /// deleting charracters before (backspace) or after (DEL)
    /// might need to support arrow keys and cursors eventually too so that
    /// DEL works properly
    /// if backspace, count should be negative (go backwards)
    pub fn _backspace(&mut self, backwards: bool) {
        // count can be positive or negative, basically clears bytes
        // in that "direction" in the buffer
        if backwards {
            if self.column_position > 0 { // don't let it run over
                self.column_position -= 1; 
                self.write_byte(' ' as u8);
                self.column_position -= 1;  // restore cursor position
            }
            
        } else {
            if self.column_position < BUFFER_WIDTH { // don't let it run over
                self.column_position += 1;
                self.write_byte(' ' as u8);
                self.column_position += 1; 
            }
        }
        
    }

    pub fn _cursor_left(&mut self) {

        if self.column_position > 0 { // don't let it run over
            self.column_position -= 1; 
        }
        
        
    }

    pub fn _cursor_right(&mut self) {
        if self.column_position < BUFFER_WIDTH { // don't let it run over
            self.column_position += 1;
        }
    }

    // add cursor up and down?
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Like the `print!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

/// Like the `println!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Prints the given formatted string to the VGA text buffer through the global `WRITER` instance.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {     // disable interrupts as long as the Mutex is locked:
        WRITER.lock().write_fmt(args).unwrap();
    });
}


pub fn backspace(backwards: bool) {

    // interrupts::without_interrupts(|| {     // disable interrupts as long as the Mutex is locked:
    WRITER.lock()._backspace(backwards);
    // });
}

pub fn cursor_left() {
    WRITER.lock()._cursor_left();
}


pub fn cursor_right() {
    WRITER.lock()._cursor_right();
}

