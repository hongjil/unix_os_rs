use core::arch::global_asm;

global_asm!(include_str!("link_app.S"));
pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    // The value in _num_app address is the number of applications.
    let num_app_ptr = _num_app as usize as *const usize;
    unsafe { num_app_ptr.read_volatile() }
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    extern "C" {
        fn _num_app();
    }
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    // Following the _num_app, there are _num_app + 1 addresses. For application
    // i-th, its start address is i-th value and the end address is (i+1)-th
    // value.
    let app_start =
        unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
    assert!(app_id < num_app);
    unsafe {
        core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id],
        )
    }
}
