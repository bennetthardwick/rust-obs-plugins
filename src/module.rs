use crate::source::{traits::Sourceable, SourceInfo, SourceInfoBuilder};
use crate::output::{traits::Outputable, OutputInfo, OutputInfoBuilder};
use crate::string::ObsString;
use obs_sys::{
    obs_module_t,
    obs_register_source_s, obs_register_output_s,
    obs_source_info, obs_output_info,
    size_t,
};
use std::marker::PhantomData;

pub struct LoadContext {
    __marker: PhantomData<()>,
    sources: Vec<*mut obs_source_info>,
    outputs: Vec<*mut obs_output_info>,
}

impl LoadContext {
    /// # Safety
    /// LoadContext can only be used at specific times. Creating it could cause UB if done at the
    /// wrong time.
    pub unsafe fn new() -> LoadContext {
        LoadContext {
            __marker: PhantomData,
            sources: vec![],
            outputs: vec![],
        }
    }

    pub fn create_source_builder<D: Sourceable>(&self) -> SourceInfoBuilder<D> {
        SourceInfoBuilder::new()
    }

    pub fn create_output_builder<D: Outputable>(&self) -> OutputInfoBuilder<D> {
        OutputInfoBuilder::new()
    }

    pub fn register_source(&mut self, source: SourceInfo) {
        let pointer = unsafe {
            let pointer = source.into_raw();
            obs_register_source_s(pointer, std::mem::size_of::<obs_source_info>() as size_t);
            pointer
        };
        self.sources.push(pointer);
    }

    pub fn register_output(&mut self, output: OutputInfo) {
        let pointer = unsafe {
            let pointer = output.into_raw();
            obs_register_output_s(pointer, std::mem::size_of::<obs_output_info>() as size_t);
            pointer
        };
        self.outputs.push(pointer);
    }
}

impl Drop for LoadContext {
    fn drop(&mut self) {
        unsafe {
            for pointer in self.sources.drain(..) {
                drop(Box::from_raw(pointer))
            }
            for pointer in self.outputs.drain(..) {
                drop(Box::from_raw(pointer))
            }
        }
    }
}

pub trait Module {
    fn new(ctx: ModuleContext) -> Self;
    fn get_ctx(&self) -> &ModuleContext;
    fn load(&mut self, _load_context: &mut LoadContext) -> bool {
        true
    }
    fn unload(&mut self) {}
    fn post_load(&mut self) {}
    fn description() -> ObsString;
    fn name() -> ObsString;
    fn author() -> ObsString;
}

#[macro_export]
macro_rules! obs_register_module {
    ($t:ty) => {
        static mut OBS_MODULE: Option<$t> = None;
        static mut LOAD_CONTEXT: Option<$crate::module::LoadContext> = None;

        #[allow(missing_safety_doc)]
        #[no_mangle]
        pub unsafe extern "C" fn obs_module_set_pointer(raw: *mut $crate::obs_sys::obs_module_t) {
            OBS_MODULE = Some(<$t>::new(ModuleContext::new(raw)));
        }

        #[allow(missing_safety_doc)]
        #[no_mangle]
        pub unsafe extern "C" fn obs_current_module() -> *mut $crate::obs_sys::obs_module_t {
            if let Some(module) = &OBS_MODULE {
                module.get_ctx().get_raw()
            } else {
                panic!("Could not get current module!");
            }
        }

        #[allow(missing_safety_doc)]
        #[no_mangle]
        pub unsafe extern "C" fn obs_module_ver() -> u32 {
            $crate::obs_sys::LIBOBS_API_MAJOR_VER
        }

        #[allow(missing_safety_doc)]
        #[no_mangle]
        pub unsafe extern "C" fn obs_module_load() -> bool {
            let mut module = OBS_MODULE.as_mut().expect("Could not get current module!");
            let mut context = unsafe { $crate::module::LoadContext::new() };
            let ret = module.load(&mut context);
            LOAD_CONTEXT = Some(context);

            ret
        }

        #[allow(missing_safety_doc)]
        #[no_mangle]
        pub unsafe extern "C" fn obs_module_unload() {
            let mut module = OBS_MODULE.as_mut().expect("Could not get current module!");
            module.unload();
        }

        #[allow(missing_safety_doc)]
        #[no_mangle]
        pub unsafe extern "C" fn obs_module_post_load() {
            let mut module = OBS_MODULE.as_mut().expect("Could not get current module!");
            module.post_load();
        }

        #[allow(missing_safety_doc)]
        #[no_mangle]
        pub unsafe extern "C" fn obs_module_name() -> *const std::os::raw::c_char {
            <$t>::name().as_ptr()
        }

        #[allow(missing_safety_doc)]
        #[no_mangle]
        pub unsafe extern "C" fn obs_module_description() -> *const std::os::raw::c_char {
            <$t>::description().as_ptr()
        }

        #[allow(missing_safety_doc)]
        #[no_mangle]
        pub unsafe extern "C" fn obs_module_author() -> *const std::os::raw::c_char {
            <$t>::author().as_ptr()
        }
    };
}

pub struct ModuleContext {
    raw: *mut obs_module_t,
}

impl ModuleContext {
    /// # Safety
    /// Creates a ModuleContext from a pointer to the raw obs_module data which if modified could
    /// cause UB.
    pub unsafe fn new(raw: *mut obs_module_t) -> Self {
        Self { raw }
    }

    /// # Safety
    /// Returns a pointer to the raw obs_module data which if modified could
    /// cause UB.
    pub unsafe fn get_raw(&self) -> *mut obs_module_t {
        self.raw
    }
}
