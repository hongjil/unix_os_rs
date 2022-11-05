use core::mem::size_of;

use crate::config::MICRO_PER_SEC;
use crate::mm::translated_mut_byte_buffer;
use crate::task::current_user_token;
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

impl From<TimeVal> for usize {
    fn from(v: TimeVal) -> Self {
        (v.sec & 0xffff) * 1000 + v.usec / 1000
    }
}

// The argument `ts` is a user-space pointer, we need translated it into
// kernel-space before populating data; However, the pointer might map to a list
// of non-contiguous memory segments hence we need to create one locally and
// populate it into the segments finally.
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    let len = size_of::<TimeVal>();
    let now = TimeVal {
        sec: us / MICRO_PER_SEC,
        usec: us % MICRO_PER_SEC,
    };
    // Translates the user's pointer into a list of buffers to populate.
    let mut user_bufs =
        translated_mut_byte_buffer(current_user_token(), ts as *const u8, len);
    unsafe {
        let ts_slice = core::slice::from_raw_parts(
            (&now as *const TimeVal) as *const u8,
            len,
        );
        // Copy the slice into the buffers to populate.
        let mut offset = 0;
        for buf in user_bufs.iter_mut() {
            buf.copy_from_slice(&ts_slice[offset..(offset + buf.len())]);
            offset = offset + buf.len();
        }
    }
    0
}
