use crate::output::{traits::Outputable, OutputInfo, OutputInfoBuilder};
use crate::source::{traits::Sourceable, SourceInfo, SourceInfoBuilder};
use crate::string::{DisplayExt as _, ObsString, TryIntoObsString as _};
use crate::{Error, Result};
use obs_sys::{
    obs_get_module_author, obs_get_module_description, obs_get_module_file_name,
    obs_get_module_name, obs_module_t, obs_output_info, obs_register_output_s,
    obs_register_source_s, obs_source_info, size_t,
};
use std::marker::PhantomData;

pub struct LoadContext {
    __marker: PhantomData<()>,
    sources: Vec<*mut obs_source_info>,
    outputs: Vec<*mut obs_output_info>,
}

impl LoadContext {
    /// # Safety
    /// LoadContext can only be used at specific times. Creating it could cause
    /// UB if done at the wrong time.
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
        let pointer = source.into_raw();
        unsafe {
            obs_register_source_s(pointer, std::mem::size_of::<obs_source_info>() as size_t);
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
    fn new(ctx: ModuleRef) -> Self;
    fn get_ctx(&self) -> &ModuleRef;
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
            OBS_MODULE = ModuleRef::from_raw(raw).ok().map(<$t>::new);
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

#[deprecated = "use `ModuleRef` instead"]
pub type ModuleContext = ModuleRef;

pub struct ModuleRef {
    raw: *mut obs_module_t,
}

impl std::fmt::Debug for ModuleRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModuleRef")
            .field("name", &self.name().display())
            .field("description", &self.description().display())
            .field("author", &self.author().display())
            .field("file_name", &self.file_name().display())
            .finish()
    }
}

impl ModuleRef {
    /// # Safety
    /// Creates a ModuleRef from a pointer to the raw obs_module data which
    /// if modified could cause UB.
    pub fn from_raw(raw: *mut obs_module_t) -> Result<Self> {
        if raw.is_null() {
            Err(Error::NulPointer("obs_module_t"))
        } else {
            Ok(Self { raw })
        }
    }

    /// # Safety
    /// Returns a pointer to the raw obs_module data which if modified could
    /// cause UB.
    pub unsafe fn get_raw(&self) -> *mut obs_module_t {
        self.raw
    }
}

impl ModuleRef {
    pub fn name(&self) -> Result<ObsString> {
        unsafe { obs_get_module_name(self.raw) }.try_into_obs_string()
    }

    pub fn description(&self) -> Result<ObsString> {
        unsafe { obs_get_module_description(self.raw) }.try_into_obs_string()
    }

    pub fn author(&self) -> Result<ObsString> {
        unsafe { obs_get_module_author(self.raw) }.try_into_obs_string()
    }

    pub fn file_name(&self) -> Result<ObsString> {
        unsafe { obs_get_module_file_name(self.raw) }.try_into_obs_string()
    }
}
