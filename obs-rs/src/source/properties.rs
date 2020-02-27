use super::ObsString;
use obs_sys::{obs_properties_add_float_slider, obs_properties_t};
use std::os::raw::c_char;

pub struct ParamBuilder {}

pub struct Properties {
    pointer: *mut obs_properties_t,
}

impl Properties {
    pub unsafe fn from_raw(pointer: *mut obs_properties_t) -> Self {
        Self { pointer }
    }

    pub unsafe fn into_raw(self) -> *mut obs_properties_t {
        self.pointer
    }

    pub fn add_float_slider(
        &mut self,
        name: ObsString,
        description: ObsString,
        min: f64,
        max: f64,
        step: f64,
    ) -> &mut Self {
        unsafe {
            obs_properties_add_float_slider(
                self.pointer,
                name.as_ptr(),
                description.as_ptr(),
                min,
                max,
                step,
            );
        }
        self
    }
}
