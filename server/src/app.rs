//! Out first app module. Just print string.

pub fn main(_hartid: usize, _dtb: usize) {
    crate::console::write_bytes(b"\nHello, ArceOS!\n");
}
