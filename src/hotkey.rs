use obs_sys::{obs_hotkey_get_id, obs_hotkey_id, obs_hotkey_t};

use crate::string::ObsString;

pub type HotkeyCallbacks<T> = Vec<(ObsString, ObsString, Box<dyn FnMut(&mut Hotkey, &mut T)>)>;

pub struct Hotkey {
    key: *mut obs_hotkey_t,
    pub pressed: bool,
}

impl Hotkey {
    pub(crate) unsafe fn from_raw(key: *mut obs_hotkey_t, pressed: bool) -> Self {
        Self { key, pressed }
    }

    pub fn id(&self) -> obs_hotkey_id {
        unsafe { obs_hotkey_get_id(self.key) }
    }
}
