/// Writes a byte to the console.
pub fn putchar(c: u8) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c as usize);
}

/// Write a slice of bytes to the console.
pub fn write_bytes(bytes: &[u8]) {
    for c in bytes {
        putchar(*c);
    }
}
