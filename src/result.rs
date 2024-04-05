pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error from OBS API
    #[error("OBS Error: {0}")]
    ObsError(i32),
    /// Null Pointer
    #[error("Null Pointer Error: {0}")]
    NulPointer(&'static str),
    /// Null Error
    #[error("Null String Error: {0}")]
    NulError(#[from] std::ffi::NulError),
    /// Error converting string
    #[error("Utf8 Error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    /// Error converting enum
    #[error("Enum Out of Range: {0} {1}")]
    EnumOutOfRange(&'static str, i64),
}

pub trait OptionExt {
    fn null_pointer(self, msg: &'static str) -> Result<()>;
}
impl OptionExt for Option<()> {
    fn null_pointer(self, msg: &'static str) -> Result<()> {
        self.ok_or(Error::NulPointer(msg))
    }
}
