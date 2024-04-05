use std::{
    ffi::{CStr, CString},
    ptr::null,
};

use crate::Result;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum ObsString {
    Static(&'static str),
    Dynamic(CString),
}

impl ObsString {
    /// # Safety
    /// Does no checks for nul terminated strings. This could cause memory
    /// overruns if used incorrectly.
    pub const unsafe fn from_nul_terminted_str(string: &'static str) -> Self {
        Self::Static(string)
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Static(s) => s,
            Self::Dynamic(s) => s.as_c_str().to_str().unwrap(),
        }
    }

    pub fn as_ptr(&self) -> *const std::os::raw::c_char {
        match self {
            Self::Static(s) => (*s).as_ptr() as *const std::os::raw::c_char,
            Self::Dynamic(s) => s.as_ptr(),
        }
    }

    pub fn ptr_or_null(opt: &Option<Self>) -> *const std::os::raw::c_char {
        opt.as_ref().map(|s| s.as_ptr()).unwrap_or(null())
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

pub trait TryIntoObsString {
    fn try_into_obs_string(self) -> Result<ObsString>;
}

impl TryIntoObsString for &str {
    fn try_into_obs_string(self) -> Result<ObsString> {
        Ok(ObsString::Dynamic(CString::new(self)?))
    }
}
impl TryIntoObsString for String {
    fn try_into_obs_string(self) -> Result<ObsString> {
        Ok(ObsString::Dynamic(CString::new(self)?))
    }
}
impl TryIntoObsString for *const std::os::raw::c_char {
    fn try_into_obs_string(self) -> Result<ObsString> {
        if self.is_null() {
            return Err(crate::Error::NulPointer("ObsString"));
        }
        Ok(ObsString::Dynamic(
            unsafe { CStr::from_ptr(self) }.to_owned(),
        ))
    }
}

pub struct DisplayStr<'a, T>(&'a T);
pub trait DisplayExt: Sized {
    fn display(&self) -> DisplayStr<'_, Self> {
        DisplayStr(self)
    }
}
impl DisplayExt for ObsString {}
impl DisplayExt for Option<ObsString> {}
impl<'a> DisplayExt for Option<&'a ObsString> {}
impl DisplayExt for Result<ObsString> {}
impl std::fmt::Display for DisplayStr<'_, ObsString> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisplayStr(ObsString::Static(s)) => return write!(f, "{}", &s[..s.len() - 1]),
            DisplayStr(ObsString::Dynamic(s)) => return write!(f, "{}", s.to_string_lossy()),
        }
    }
}
impl std::fmt::Debug for DisplayStr<'_, ObsString> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.to_string())
    }
}

impl<'a, T: DisplayExt> std::fmt::Display for DisplayStr<'a, Option<T>>
where
    DisplayStr<'a, T>: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(s) => write!(f, "{}", s.display()),
            None => write!(f, "<null>"),
        }
    }
}
impl<'a, T: DisplayExt> std::fmt::Debug for DisplayStr<'a, Option<T>>
where
    DisplayStr<'a, T>: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(s) => write!(f, "{:?}", s.display()),
            None => write!(f, "None"),
        }
    }
}
impl<'a, T: DisplayExt> std::fmt::Debug for DisplayStr<'a, Result<T>>
where
    DisplayStr<'a, T>: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Ok(s) => write!(f, "{:?}", s.display()),
            _ => write!(f, "Error"),
        }
    }
}
