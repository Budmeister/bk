#![no_std]
#![no_main]

mod panicing;

mod io;
mod shell;

// Define the entry point
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    // Initialize Raspberry Pi 4 UART
    io::uart::init(4);

    // loop {
    //     let b = io::uart::getb();
    //     io::uart::puts("Character: ");
    //     io::uart::put_u8(b);
    //     io::uart::puts("\r\n");
    // }

    loop {
        shell::run_shell();
    }
}
