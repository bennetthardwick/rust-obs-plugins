use super::audio::AudioRef;
use obs_sys::obs_get_audio;

pub struct GlobalContext;
pub struct VideoRenderContext;

impl GlobalContext {
    pub fn with_audio<T, F: FnOnce(&AudioRef) -> T>(&self, func: F) -> T {
        let audio = unsafe { AudioRef::from_raw(obs_get_audio()) };
        func(&audio)
    }
}

impl Default for VideoRenderContext {
    fn default() -> Self {
        Self
    }
}

impl Default for GlobalContext {
    fn default() -> Self {
        Self
    }
}
