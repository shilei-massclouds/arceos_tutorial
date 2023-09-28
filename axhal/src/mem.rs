/// Fills the `.bss` section with zeros.
pub(crate) fn clear_bss() {
    unsafe {
        core::slice::from_raw_parts_mut(_sbss as usize as *mut u8, _ebss as usize - _sbss as usize)
            .fill(0);
    }
}

extern "C" {
    fn _sbss();
    fn _ebss();
}
