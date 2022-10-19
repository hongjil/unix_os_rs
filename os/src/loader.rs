use crate::config;
use crate::trap::TrapContext;
use core::arch::{asm, global_asm};

#[repr(align(4096))]
#[derive(Copy, Clone)]
pub struct KernelStack {
    data: [u8; config::KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
#[derive(Copy, Clone)]
pub struct UserStack {
    data: [u8; config::USER_STACK_SIZE],
}

impl KernelStack {
    pub fn get_sp(&self) -> usize {
        return self.data.as_ptr() as usize + config::KERNEL_STACK_SIZE;
    }
    pub fn push_context(&self, ctx: TrapContext) -> &'static mut TrapContext {
        let ctx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *ctx_ptr = ctx;
        }
        unsafe { ctx_ptr.as_mut().unwrap() }
    }
}
impl UserStack {
    pub fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + config::USER_STACK_SIZE
    }
}

pub static KERNEL_STACK: [KernelStack; config::MAX_APP_NUM] = [KernelStack {
    data: [0; config::KERNEL_STACK_SIZE],
}; config::MAX_APP_NUM];

pub static USER_STACK: [UserStack; config::MAX_APP_NUM] = [UserStack {
    data: [0; config::USER_STACK_SIZE],
}; config::MAX_APP_NUM];

pub fn get_app_base(app_id: usize) -> usize {
    config::APP_BASE_ADDRESS + app_id * config::APP_SIZE_LIMIT
}

global_asm!(include_str!("link_app.S"));
pub fn load_apps() {
    extern "C" {
        fn _num_app();
    }
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = unsafe { num_app_ptr.read_volatile() };
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
    // clear i-cache first
    unsafe {
        asm!("fence.i");
    }
    // load apps
    for i in 0..num_app {
        let base_i = get_app_base(i);
        println!("[kernel] App {} base address 0x{:x}", i, base_i);
        // clear region
        (base_i..base_i + config::APP_SIZE_LIMIT)
            .for_each(|addr| unsafe { (addr as *mut u8).write_volatile(0) });
        // load app from data section to memory
        let src = unsafe {
            core::slice::from_raw_parts(app_start[i] as *const u8, app_start[i + 1] - app_start[i])
        };
        let dst = unsafe { core::slice::from_raw_parts_mut(base_i as *mut u8, src.len()) };
        dst.copy_from_slice(src);
    }
}

pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    let num_app_ptr = _num_app as usize as *const usize;
    unsafe { num_app_ptr.read_volatile() }
}
