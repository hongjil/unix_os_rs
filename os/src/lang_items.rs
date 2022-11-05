use crate::sbi::shutdown;
use crate::stack_trace;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "[kernel] Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("[kernel] Panicked: {}", info.message().unwrap());
    }

    #[cfg(debug_assertions)]
    unsafe {
        stack_trace::print_stack_trace();
    }

    shutdown()
}
