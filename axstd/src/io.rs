/// Constructs a new handle to the standard output of the current process.

use core::fmt::{Write, Error};
use spinlock::SpinRaw;

struct StdoutRaw;

impl Write for StdoutRaw {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        axhal::console::write_bytes(s.as_bytes());
        Ok(())
    }
}

static STDOUT: SpinRaw<StdoutRaw> = SpinRaw::new(StdoutRaw);

pub fn __print_impl(args: core::fmt::Arguments) {
    STDOUT.lock().write_fmt(args).unwrap();
}
