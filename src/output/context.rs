use obs_sys::{obs_output_create, obs_output_get_ref, obs_output_release, obs_output_t};

use crate::{hotkey::Hotkey, prelude::DataObj, string::ObsString, wrapper::PtrWrapper};

/// Context wrapping an OBS output - video / audio elements which are displayed
/// to the screen.
///
/// See [OBS documentation](https://obsproject.com/docs/reference-outputs.html#c.obs_output_t)
pub struct OutputContext {
    pub(crate) inner: *mut obs_output_t,
}

impl OutputContext {
    pub fn from_raw(output: *mut obs_output_t) -> Self {
        Self {
            inner: unsafe { obs_output_get_ref(output) },
        }
    }
}

impl Clone for OutputContext {
    fn clone(&self) -> Self {
        Self::from_raw(self.inner)
    }
}

impl OutputContext {
    pub fn new(id: ObsString, name: ObsString, settings: Option<DataObj<'_>>) -> Self {
        let settings = match settings {
            Some(mut data) => data.as_ptr_mut(),
            None => std::ptr::null_mut(),
        };
        let output = unsafe {
            obs_output_create(id.as_ptr(), name.as_ptr(), settings, std::ptr::null_mut())
        };
        Self::from_raw(output)
    }
}

impl Drop for OutputContext {
    fn drop(&mut self) {
        unsafe { obs_output_release(self.inner) }
    }
}

pub struct CreatableOutputContext<'a, D> {
    pub(crate) hotkey_callbacks: Vec<(ObsString, ObsString, Box<dyn FnMut(&mut Hotkey, &mut D)>)>,
    pub settings: DataObj<'a>,
}

impl<'a, D> CreatableOutputContext<'a, D> {
    pub fn from_raw(settings: DataObj<'a>) -> Self {
        Self {
            hotkey_callbacks: vec![],
            settings,
        }
    }

    pub fn register_hotkey<F: FnMut(&mut Hotkey, &mut D) + 'static>(
        &mut self,
        name: ObsString,
        description: ObsString,
        func: F,
    ) {
        self.hotkey_callbacks
            .push((name, description, Box::new(func)));
    }
}
