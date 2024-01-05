#![no_std]

#[crate_interface::def_interface]
pub trait LogIf {
    fn write_str(s: &str);
    fn get_time() -> core::time::Duration;
}

pub fn init() {
    extern crate alloc;

    let now = crate_interface::call_interface!(LogIf::get_time());
    let s = alloc::format!("Logging startup time: {}.{:06}",
        now.as_secs(), now.subsec_micros());
    crate_interface::call_interface!(LogIf::write_str(s.as_str()));
}
