use core::fmt::{Write, Error};

pub fn putchar(c: u8) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c as usize);
}

pub fn write_bytes(bytes: &[u8]) {
    for c in bytes {
        putchar(*c);
    }
}

#[macro_export]
macro_rules! ax_print {
    ($($arg:tt)*) => {
        $crate::console::__print_impl(format_args!($($arg)*));
    }
}

#[macro_export]
macro_rules! ax_println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => {
        $crate::console::__print_impl(format_args!("{}\n", format_args!($($arg)*)));
    }
}

struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        write_bytes(s.as_bytes());
        Ok(())
    }
}

pub fn __print_impl(args: core::fmt::Arguments) {
    Console.write_fmt(args).unwrap();
}
