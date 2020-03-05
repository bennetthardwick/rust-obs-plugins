use obs_sys::obs_module_t;

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
