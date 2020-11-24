use std::ffi::CString;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum ObsString {
    Static(&'static str),
    Dynamic(CString),
}

impl ObsString {
    /// # Safety
    /// Does no checks for nul terminated strings. This could cause memory overruns if used
    /// incorrectly.
    pub const unsafe fn from_nul_terminted_str(string: &'static str) -> Self {
        Self::Static(string)
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Static(s) => *s,
            Self::Dynamic(s) => s.as_c_str().to_str().unwrap(),
        }
    }

    pub fn as_ptr(&self) -> *const std::os::raw::c_char {
        match self {
            Self::Static(s) => (*s).as_ptr() as *const std::os::raw::c_char,
            Self::Dynamic(s) => s.as_ptr(),
        }
    }
}

impl<T: Into<Vec<u8>>> From<T> for ObsString {
    fn from(s: T) -> Self {
        Self::Dynamic(CString::new(s).expect("Failed to convert to CString"))
    }
}

#[macro_export]
macro_rules! obs_string {
    ($e:expr) => {
        unsafe { $crate::string::ObsString::from_nul_terminted_str(concat!($e, "\0")) }
    };
}
