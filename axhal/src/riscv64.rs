mod lang_items;
mod boot;
pub mod console;
mod paging;
pub mod time;

unsafe extern "C" fn rust_entry(hartid: usize, dtb: usize) {
    extern "C" {
        fn rust_main(hartid: usize, dtb: usize);
    }
    rust_main(hartid, dtb);
}
