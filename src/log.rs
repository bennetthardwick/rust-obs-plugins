use std::os::raw::c_char;

use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use obs_sys::{blog, LOG_DEBUG, LOG_ERROR, LOG_INFO, LOG_WARNING};

/// A logger that plugs into OBS's logging system.
///
/// Since OBS only has 4 logging levels and the lowest level is
/// only enabled in debug builds of OBS, this logger provides a option
/// to promote lower-level logs as `info`.
///
/// You can also use any other logger implementation, but we recommend this since
/// OBS also writes everything in its logging system to a file, which can be viewed
/// if there is a problem and OBS is not started from a console.
pub struct Logger {
    max_level: LevelFilter,
    promote_debug: bool,
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            max_level: LevelFilter::Info,
            promote_debug: false,
        }
    }
}

impl Logger {
    /// Creates a new logger with default levle set to [`Level::Trace`] and does
    /// not promote debug logs.
    #[must_use = "You must call init() to begin logging"]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init(self) -> Result<(), SetLoggerError> {
        log::set_max_level(self.max_level);
        log::set_boxed_logger(Box::new(self))?;
        Ok(())
    }

    /// Sets whether to promote [`Level::Debug`] and [`Level::Trace`] logs.
    #[must_use = "You must call init() to begin logging"]
    pub fn with_promote_debug(mut self, promote_debug: bool) -> Self {
        self.promote_debug = promote_debug;
        self
    }

    /// Sets the maximum logging level.
    #[must_use = "You must call init() to begin logging"]
    pub fn with_max_level(mut self, max_level: LevelFilter) -> Self {
        self.max_level = max_level;
        self
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() >= self.max_level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let level = record.level();
        let native_level = to_native_level(level, self.promote_debug);
        let target = if !record.target().is_empty() {
            record.target()
        } else {
            record.module_path().unwrap_or_default()
        };

        let line = if self.promote_debug && level <= Level::Debug {
            format!("({}) [{}] {}\0", level, target, record.args())
        } else {
            format!("[{}] {}\0", target, record.args())
        };

        unsafe {
            blog(
                native_level as i32,
                "%s\0".as_ptr() as *const c_char,
                line.as_ptr() as *const c_char,
            );
        }
    }

    fn flush(&self) {
        // No need to flush
    }
}

fn to_native_level(level: Level, promote_debug: bool) -> u32 {
    match level {
        Level::Error => LOG_ERROR,
        Level::Warn => LOG_WARNING,
        Level::Info => LOG_INFO,
        _ => {
            if promote_debug {
                // Debug logs are only enabled in debug builds of OBS, make them accessible as info if needed
                LOG_INFO
            } else {
                LOG_DEBUG
            }
        }
    }
}
