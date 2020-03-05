pub struct ObsString(&'static str);

impl ObsString {
    pub unsafe fn from_str(string: &'static str) -> Self {
        Self(string)
    }

    pub fn as_str(&self) -> &'static str {
        self.0
    }

    pub fn as_ptr(&self) -> *const std::os::raw::c_char {
        self.0.as_ptr() as *const std::os::raw::c_char
    }
}

#[macro_export]
macro_rules! obs_string {
    ($e:expr) => {
        unsafe { $crate::string::ObsString::from_str(concat!($e, "\0")) }
    };
}
