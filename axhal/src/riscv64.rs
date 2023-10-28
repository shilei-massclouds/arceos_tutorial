mod boot;

pub mod paging;
pub mod console;
pub mod time;
pub mod misc;
pub mod mem;

unsafe extern "C" fn rust_entry(hartid: usize, dtb: usize) {
    extern "C" {
        fn rust_main(hartid: usize, dtb: usize);
    }

    rust_main(hartid, dtb);
}
