use crate::ModuleContext;

pub trait Module {
    fn new(ctx: ModuleContext) -> Self;
    fn get_ctx(&self) -> &ModuleContext;
    fn load(&mut self) -> bool {
        true
    }
    fn unload(&mut self) {}
    fn post_load(&mut self) {}
    fn description() -> &'static str;
    fn name() -> &'static str;
    fn author() -> &'static str;
}

#[macro_export]
macro_rules! obs_register_module {
    ($t:ty) => {
        use std::cell::RefCell;
        use std::os::raw::c_char;
        use $crate::obs_sys::{obs_module_t, LIBOBS_API_MAJOR_VER};

        static mut OBS_MODULE: Option<$t> = None;

        #[no_mangle]
        pub unsafe extern "C" fn obs_module_set_pointer(raw: *mut obs_module_t) {
            OBS_MODULE = Some(<$t>::new(ModuleContext { raw }));
        }

        #[no_mangle]
        pub unsafe extern "C" fn obs_current_module() -> *mut obs_module_t {
            if let Some(module) = &OBS_MODULE {
                module.get_ctx().raw
            } else {
                panic!("Could not get current module!");
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn obs_module_ver() -> u32 {
            LIBOBS_API_MAJOR_VER
        }

        #[no_mangle]
        pub unsafe extern "C" fn obs_module_load() -> bool {
            let mut module = OBS_MODULE.as_mut().expect("Could not get current module!");
            module.load()
        }

        #[no_mangle]
        pub unsafe extern "C" fn obs_module_unload() {
            let mut module = OBS_MODULE.as_mut().expect("Could not get current module!");
            module.unload();
        }

        #[no_mangle]
        pub unsafe extern "C" fn obs_module_post_load() {
            let mut module = OBS_MODULE.as_mut().expect("Could not get current module!");
            module.post_load();
        }

        #[no_mangle]
        pub unsafe extern "C" fn obs_module_name() -> *const c_char {
            <$t>::name().as_bytes().as_ptr() as *const c_char
        }

        #[no_mangle]
        pub unsafe extern "C" fn obs_module_description() -> *const c_char {
            <$t>::description().as_bytes().as_ptr() as *const c_char
        }

        #[no_mangle]
        pub unsafe extern "C" fn obs_module_author() -> *const c_char {
            <$t>::description().as_bytes().as_ptr() as *const c_char
        }
    };
}
