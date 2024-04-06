#[derive(Debug)]
pub struct NativeParsingError {
    struct_name: &'static str,
    value: i64,
}

impl NativeParsingError {
    pub(crate) fn new(struct_name: &'static str, value: i64) -> Self {
        Self { struct_name, value }
    }
}

impl std::fmt::Display for NativeParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to convert native value {} into {}",
            self.value, self.struct_name
        )
    }
}

impl std::error::Error for NativeParsingError {}

#[macro_export]
macro_rules! native_enum {
    ($(#[$($attrs_enum:tt)*])* $name:ident,$native_name:ident { $($(#[$($attrss:tt)*])* $rust:ident => $native:ident,)* }) => {
        paste::item! {
            $(#[$($attrs_enum)*])*
            #[derive(Debug, Clone, Copy, Eq, PartialEq)]
            pub enum $name {
                $(
                    $(#[$($attrss)*])*
                    $rust,
                )*
            }

            impl $name {
                pub fn as_raw(&self) -> $native_name {
                    match self {
                        $(Self::$rust => [<$native_name _ $native>],)*
                    }
                }

                #[allow(non_upper_case_globals)]
                pub fn from_raw(value: $native_name) -> Result<Self, $crate::native_enum::NativeParsingError> {
                    match value {
                        $([<$native_name _ $native>] => Ok(Self::$rust)),*,
                        _ => Err($crate::native_enum::NativeParsingError::new(stringify!($name), value as i64))
                    }
                }
            }

            #[allow(clippy::from_over_into)]
            impl Into<$native_name> for $name {
                fn into(self) -> $native_name {
                    self.as_raw()
                }
            }

            impl std::convert::TryFrom<$native_name> for $name {
                type Error = $crate::native_enum::NativeParsingError;
                fn try_from(value: $native_name) -> Result<Self, $crate::native_enum::NativeParsingError> {
                    Self::from_raw(value)
                }
            }
        }
    };
}
