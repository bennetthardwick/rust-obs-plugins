use obs_sys::obs_hotkey_t;

pub struct Hotkey {
    key: *mut obs_hotkey_t,
    pub pressed: bool
}

impl Hotkey {
    pub(crate) unsafe fn from_raw(key: *mut obs_hotkey_t, pressed: bool) -> Self {
        Self { key, pressed }
    }
}
