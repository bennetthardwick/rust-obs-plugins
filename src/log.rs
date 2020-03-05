#[macro_export]
macro_rules! obs_log {
        ($level:expr, $($arg:tt)*) => (unsafe {
            $crate::obs_sys::blog($level, format!("{}", format_args!($($arg)*)).as_ptr() as *const std::os::raw::c_char)
        });
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
