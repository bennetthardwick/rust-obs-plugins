pub use obs_sys;

pub mod context;
pub mod module;
pub mod source;

pub use context::ModuleContext;
pub use module::*;

pub mod string {
    pub struct ObsString(&'static str);

    impl ObsString {
        pub unsafe fn from_str(string: &'static str) -> Self {
            Self(string)
        }

        pub fn as_ptr(&self) -> *const std::os::raw::c_char {
            self.0.as_ptr() as *const std::os::raw::c_char
        }
    }

    #[macro_export]
    macro_rules! obs_string {
        ($e:expr) => {
            unsafe { $crate::ObsString::from_str(concat!($e, "\0")) }
        };
    }
}

pub mod log {
    #[macro_export]
    macro_rules! obs_log {
        ($level:expr, $($arg:tt)*) => (
            $crate::obs_sys::blog($level, format!("{}", format_args!($($arg)*)).as_ptr() as *const std::os::raw::c_char)
        );
    }

    #[macro_export]
    macro_rules! debug {
        ($($arg:tt)*) => ($crate::obs_log!(400, $($arg)*));
    }

    #[macro_export]
    macro_rules! info {
        ($($arg:tt)*) => ($crate::obs_log!(300, $($arg)*));
    }

    #[macro_export]
    macro_rules! warning {
        ($($arg:tt)*) => ($crate::obs_log!(200, $($arg)*));
    }

    #[macro_export]
    macro_rules! error {
        ($($arg:tt)*) => ($crate::obs_log!(100, $($arg)*));
    }
}

pub use log::*;
pub use string::*;
