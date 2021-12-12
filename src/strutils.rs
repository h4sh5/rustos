use crate::vga_buffer::{BUFFER_WIDTH};
use crate::{println};

/// compare 2 char arrays bounded by n, 
/// returns true if they are identical
pub fn strcmpl(s1: &[char; BUFFER_WIDTH], s2: &str, n: usize) -> bool {

	let s2_b = s2.as_bytes();
	// println!("strcmpl: {:?} vs {:?}", s1, s2_b);
	for i in 0..n {
		if s1[i] != s2_b[i] as char {
			// println!("mismatch: {} {}",s1[i],s2_b[i] as char );
			return false;
		}
	}
	return true;
}