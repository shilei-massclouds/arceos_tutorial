/// Constructs a new handle to the standard output of the current process.

use core::fmt::{Write, Error};
use spinlock::SpinNoIrq;

struct StdoutRaw;

impl Write for StdoutRaw {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        axhal::console::write_bytes(s.as_bytes());
        Ok(())
    }
}

static STDOUT: SpinNoIrq<StdoutRaw> = SpinNoIrq::new(StdoutRaw);

pub fn __print_impl(args: core::fmt::Arguments) {
    STDOUT.lock().write_fmt(args).unwrap();
}
