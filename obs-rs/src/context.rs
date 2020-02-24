use obs_sys::obs_module_t;

pub struct ModuleContext {
    pub raw: *mut obs_module_t,
}
