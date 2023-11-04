mod boot;

mod context;
pub use context::{TaskContext};

pub mod paging;
pub mod console;
pub mod time;
pub mod misc;
pub mod mem;
pub mod cpu;

unsafe extern "C" fn rust_entry(hartid: usize, dtb: usize) {
    extern "C" {
        fn rust_main(hartid: usize, dtb: usize);
    }

    rust_main(hartid, dtb);
}
