use core::panic::PanicInfo;
use axlog::error;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{}", info);
    super::misc::terminate()
}
