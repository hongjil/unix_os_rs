use crate::mm::translated_byte_buffer;
use crate::task::{current_idx, current_user_memory_set};

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buffers = translated_byte_buffer(
                current_user_memory_set().exclusive_access().token(),
                buf,
                len,
            );
            #[cfg(debug_assertions)]
            print!("[app {}] ", current_idx());
            for buffer in buffers {
                user_print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            #[cfg(debug_assertions)]
            println!("");
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}
