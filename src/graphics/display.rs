use obs_sys::{obs_display_destroy, obs_display_enabled, obs_display_resize, obs_display_t};

pub struct DisplayRef {
    inner: *mut obs_display_t
}

impl_ptr_wrapper!(@ptr: inner, DisplayRef, obs_display_t, @identity, obs_display_destroy);

impl DisplayRef {
    pub fn enabled(&self) -> bool {
        unsafe {
            obs_display_enabled(self.inner)
        }
    }

    pub fn set_size(&self, cx: u32, cy: u32) {
        unsafe {
            obs_display_resize(self.inner, cx, cy)
        }
    }
}
