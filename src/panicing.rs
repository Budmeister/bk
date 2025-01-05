
use core::panic::PanicInfo;
use crate::io;

// Define a panic handler
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    io::uart::puts("Thread panic!");
    loop {}
}
