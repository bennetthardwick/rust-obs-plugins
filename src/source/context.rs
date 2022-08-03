use super::audio::AudioRef;
use super::hotkey::Hotkey;
use crate::prelude::DataObj;
use crate::string::ObsString;
use obs_sys::{obs_get_audio, obs_source_t};

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

pub struct CreatableSourceContext<'a, D> {
    _source: *mut obs_source_t,
    pub(crate) hotkey_callbacks: Vec<(
        ObsString,
        ObsString,
        Box<dyn FnMut(&mut Hotkey, &mut D)>,
    )>,
    pub settings: DataObj<'a>,
    pub global: &'a mut GlobalContext,
}

impl<'a, D> CreatableSourceContext<'a, D> {
    pub(crate) unsafe fn from_raw(
        source: *mut obs_source_t,
        settings: DataObj<'a>,
        global: &'a mut GlobalContext,
    ) -> Self {
        Self {
            _source: source,
            hotkey_callbacks: Default::default(),
            settings,
            global,
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

    // Inherited from child contexts

    pub fn with_audio<T, F: FnOnce(&AudioRef) -> T>(&self, func: F) -> T {
        self.global.with_audio(func)
    }
}
