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

        pub fn as_ptr(&self) -> *const u8 {
            self.0.as_ptr()
        }
    }

    #[macro_export]
    macro_rules! obs_string {
        ($e:expr) => {
            unsafe { $crate::ObsString::from_str(concat!($e, "\0")) }
        };
    }
}

pub use string::*;
