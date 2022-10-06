use crate::loader;
use crate::loader::get_app_base;
use crate::sync::UPSafeCell;
use crate::trap;
use lazy_static::*;

struct AppManager {
    // The number of application.
    num_app: usize,
    // The index of running app.
    current_app: usize,
}
// The macro lazy_static would postpone the initialization until the first time
// variables are used.
lazy_static! {
    static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {
        UPSafeCell::new({
            extern "C" {
                fn _num_app();
            }
            let num_app_ptr = _num_app as usize as *const usize;
            let num_app = num_app_ptr.read_volatile();
            AppManager {
                num_app,
                current_app: 0,
            }
        })
    };
}

impl AppManager {
    pub fn get_current_app(&self) -> usize {
        self.current_app
    }
    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }
}

pub fn run_next_app() -> ! {
    let mut app_manager = APP_MANAGER.exclusive_access();
    let app_id = app_manager.get_current_app();
    if app_id == app_manager.num_app {
        panic!("All applications done!");
    }
    app_manager.move_to_next_app();
    drop(app_manager);
    extern "C" {
        fn __restore(ctx: usize);
    }
    unsafe {
        __restore(
            loader::KERNEL_STACK[0].push_context(trap::TrapContext::app_init_context(
                get_app_base(app_id),
                loader::USER_STACK[0].get_sp(),
            )) as *const trap::TrapContext as usize,
        );
    }

    panic!("Unreachable in batch::run_current_app!");
}
