use crate::source::{
    traits::Sourceable, SettingsContext, SourceContext, SourceInfo, SourceInfoBuilder, SourceType,
};
use crate::ModuleContext;
use obs_sys::{obs_register_source_s, obs_source_info};
use std::ffi::c_void;
use std::marker::PhantomData;

pub use lazy_static::lazy_static;

pub struct LoadContext {
    __marker: PhantomData<()>,
    sources: Vec<*mut obs_source_info>,
}

impl LoadContext {
    pub unsafe fn new() -> LoadContext {
        LoadContext {
            __marker: PhantomData,
            sources: vec![],
        }
    }

    pub fn create_source_builder<T: Sourceable, D: Default>(
        &self,
        id: &'static str,
        source_type: SourceType,
    ) -> SourceInfoBuilder<T, D> {
        SourceInfoBuilder::new(id, source_type)
    }

    pub fn register_source(&mut self, source: SourceInfo) {
        let pointer = unsafe {
            let pointer = source.into_raw();
            obs_register_source_s(pointer, std::mem::size_of::<obs_source_info>() as u64);
            pointer
        };
        self.sources.push(pointer);
    }
}

impl Drop for LoadContext {
    fn drop(&mut self) {
        unsafe {
            for pointer in self.sources.drain(..) {
                drop(Box::from_raw(pointer))
            }
        }
    }
}

pub trait Module {
    fn new(ctx: ModuleContext) -> Self;
    fn get_ctx(&self) -> &ModuleContext;
    fn load(&mut self, load_context: &mut LoadContext) -> bool {
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
        static mut OBS_MODULE: Option<$t> = None;
        static mut LOAD_CONTEXT: Option<$crate::LoadContext> = None;

        #[no_mangle]
        pub unsafe extern "C" fn obs_module_set_pointer(raw: *mut $crate::obs_sys::obs_module_t) {
            OBS_MODULE = Some(<$t>::new(ModuleContext { raw }));
        }

        #[no_mangle]
        pub unsafe extern "C" fn obs_current_module() -> *mut $crate::obs_sys::obs_module_t {
            if let Some(module) = &OBS_MODULE {
                module.get_ctx().raw
            } else {
                panic!("Could not get current module!");
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn obs_module_ver() -> u32 {
            $crate::obs_sys::LIBOBS_API_MAJOR_VER
        }

        #[no_mangle]
        pub unsafe extern "C" fn obs_module_load() -> bool {
            let mut module = OBS_MODULE.as_mut().expect("Could not get current module!");
            let mut context = unsafe { $crate::LoadContext::new() };
            let ret = module.load(&mut context);
            LOAD_CONTEXT = Some(context);
            ret
        }

        #[no_mangle]
        pub unsafe extern "C" fn obs_module_unload() {
            LOAD_CONTEXT = None;
            let mut module = OBS_MODULE.as_mut().expect("Could not get current module!");
            module.unload();
        }

        #[no_mangle]
        pub unsafe extern "C" fn obs_module_post_load() {
            let mut module = OBS_MODULE.as_mut().expect("Could not get current module!");
            module.post_load();
        }

        #[no_mangle]
        pub unsafe extern "C" fn obs_module_name() -> *const std::os::raw::c_char {
            <$t>::name().as_bytes().as_ptr() as *const std::os::raw::c_char
        }

        #[no_mangle]
        pub unsafe extern "C" fn obs_module_description() -> *const std::os::raw::c_char {
            <$t>::description().as_bytes().as_ptr() as *const std::os::raw::c_char
        }

        #[no_mangle]
        pub unsafe extern "C" fn obs_module_author() -> *const std::os::raw::c_char {
            <$t>::description().as_bytes().as_ptr() as *const std::os::raw::c_char
        }
    };
}
