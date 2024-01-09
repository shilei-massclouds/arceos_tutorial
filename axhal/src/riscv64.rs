mod lang_items;
mod boot;
pub mod console;
mod paging;
pub use paging::write_page_table_root;
pub mod time;
mod misc;
pub use misc::terminate;
pub mod mem;

unsafe extern "C" fn rust_entry(hartid: usize, dtb: usize) {
    extern "C" {
        fn rust_main(hartid: usize, dtb: usize);
    }
    rust_main(hartid, dtb);
}

struct LogIfImpl;

#[crate_interface::impl_interface]
impl axlog::LogIf for LogIfImpl {
    fn write_str(s: &str) {
        console::write_bytes(s.as_bytes());
    }

    fn get_time() -> core::time::Duration {
        time::current_time()
    }
}
