mod boot;

mod context;
pub use context::{TaskContext};

pub mod paging;
pub mod console;
pub mod time;
pub mod misc;
pub mod mem;
pub mod cpu;
pub mod trap;
pub mod irq;

unsafe extern "C" fn rust_entry(hartid: usize, dtb: usize) {
    extern "C" {
        fn trap_vector_base();
        fn rust_main(hartid: usize, dtb: usize);
    }

    trap::set_trap_vector_base(trap_vector_base as usize);
    rust_main(hartid, dtb);
}
