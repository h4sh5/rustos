use crate::vga_buffer::{BUFFER_WIDTH};
use crate::strutils::{strcmpl};
use crate::{print, println};

pub const PROMPT: char = '>';

// pub fn bytes2chars(bs: &mut [char; BUFFER_WIDTH], chars: &mut [char; BUFFER_WIDTH]) {
// 	for i in 0..BUFFER_WIDTH {
// 		arr[i] = s[i];
// 	}
// }

pub fn handle_cmd(input: &[char; BUFFER_WIDTH]) {

	if strcmpl(input, "help", 4) {
		println!("you want help?");
	}
}