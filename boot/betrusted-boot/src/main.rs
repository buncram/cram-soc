#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use utralib::generated::*;

mod debug;

mod panic_handler {
    use core::panic::PanicInfo;
    #[panic_handler]
    fn handle_panic(_arg: &PanicInfo) -> ! {
        crate::println!("{}", _arg);
        loop {}
    }
}

#[export_name = "rust_entry"]
pub unsafe extern "C" fn rust_entry(_unused1: *const usize, _unused2: u32) -> ! {
    // do stuff here
    println!("hello world, this is the bootloader!");

    // now jump to the loader if everything checks out.
    start_loader(
        0x2098_0000,  // loader arguments
        0x0,          // this is unsused
        0x2050_0000,  // address of the loader itself
    );
}

extern "C" {
    fn start_loader(
        arg_buffer: usize,
        signature: usize,
        loader_addr: usize,
    ) -> !;
}
