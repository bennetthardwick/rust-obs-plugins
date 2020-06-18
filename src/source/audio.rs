use obs_sys::obs_audio_data;

pub struct AudioDataContext {
    pointer: *mut obs_audio_data,
}

impl AudioDataContext {
    pub(crate) unsafe fn from_raw(pointer: *mut obs_audio_data) -> Self {
        Self { pointer }
    }
}
