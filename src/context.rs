use obs_sys::obs_module_t;

pub struct ModuleContext {
    raw: *mut obs_module_t,
}

impl ModuleContext {
    pub unsafe fn new(raw: *mut obs_module_t) -> Self {
        Self { raw }
    }

    pub unsafe fn get_raw(&self) -> *mut obs_module_t {
        self.raw
    }
}
