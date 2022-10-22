use crate::config::MICRO_PER_SEC;
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / MICRO_PER_SEC,
            usec: us % MICRO_PER_SEC,
        };
    }
    0
}
