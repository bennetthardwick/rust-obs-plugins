#![allow(non_upper_case_globals)]
use std::{borrow::Cow, ffi::CStr, marker::PhantomData};

use obs_sys::{
    obs_data_array_count, obs_data_array_item, obs_data_array_release, obs_data_array_t,
    obs_data_clear, obs_data_create, obs_data_create_from_json, obs_data_create_from_json_file,
    obs_data_create_from_json_file_safe, obs_data_erase, obs_data_get_json, obs_data_item_byname,
    obs_data_item_get_array, obs_data_item_get_bool, obs_data_item_get_double,
    obs_data_item_get_int, obs_data_item_get_obj, obs_data_item_get_string, obs_data_item_gettype,
    obs_data_item_numtype, obs_data_item_release, obs_data_item_t, obs_data_number_type,
    obs_data_number_type_OBS_DATA_NUM_DOUBLE, obs_data_number_type_OBS_DATA_NUM_INT,
    obs_data_release, obs_data_t, obs_data_type, obs_data_type_OBS_DATA_ARRAY,
    obs_data_type_OBS_DATA_BOOLEAN, obs_data_type_OBS_DATA_NUMBER, obs_data_type_OBS_DATA_OBJECT,
    obs_data_type_OBS_DATA_STRING, size_t,
    obs_data_set_default_string,
    obs_data_set_default_double,
};

use crate::{string::ObsString, wrapper::PtrWrapper};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum DataType {
    String,
    Int,
    Double,
    Boolean,
    /// Map container
    Object,
    /// Array container
    Array,
}

impl DataType {
    pub fn new(typ: obs_data_type, numtyp: obs_data_number_type) -> Self {
        match typ {
            obs_data_type_OBS_DATA_STRING => Self::String,
            obs_data_type_OBS_DATA_NUMBER => match numtyp {
                obs_data_number_type_OBS_DATA_NUM_INT => Self::Int,
                obs_data_number_type_OBS_DATA_NUM_DOUBLE => Self::Double,
                _ => unimplemented!(),
            },
            obs_data_type_OBS_DATA_BOOLEAN => Self::Boolean,
            obs_data_type_OBS_DATA_OBJECT => Self::Object,
            obs_data_type_OBS_DATA_ARRAY => Self::Array,
            _ => unimplemented!(),
        }
    }

    unsafe fn from_item(item_ptr: *mut obs_data_item_t) -> Self {
        let typ = obs_data_item_gettype(item_ptr);
        let numtyp = obs_data_item_numtype(item_ptr);
        Self::new(typ, numtyp)
    }
}

pub trait FromDataItem {
    fn typ() -> DataType;
    unsafe fn from_item_unchecked(item: *mut obs_data_item_t) -> Self;
}

impl FromDataItem for Cow<'_, str> {
    fn typ() -> DataType {
        DataType::String
    }
    unsafe fn from_item_unchecked(item: *mut obs_data_item_t) -> Self {
        let ptr = obs_data_item_get_string(item);
        CStr::from_ptr(ptr).to_string_lossy()
    }
}

macro_rules! impl_get_int {
    ($($t:ty)*) => {
        $(
            impl FromDataItem for $t {
                fn typ() -> DataType {
                    DataType::Int
                }
                unsafe fn from_item_unchecked(item: *mut obs_data_item_t) -> Self {
                    obs_data_item_get_int(item) as $t
                }
            }
        )*
    };
}

impl_get_int!(i64 u64 i32 u32 i16 u16 i8 u8 isize usize);

impl FromDataItem for f64 {
    fn typ() -> DataType {
        DataType::Double
    }
    unsafe fn from_item_unchecked(item: *mut obs_data_item_t) -> Self {
        obs_data_item_get_double(item)
    }
}

impl FromDataItem for f32 {
    fn typ() -> DataType {
        DataType::Double
    }
    unsafe fn from_item_unchecked(item: *mut obs_data_item_t) -> Self {
        obs_data_item_get_double(item) as f32
    }
}

impl FromDataItem for bool {
    fn typ() -> DataType {
        DataType::Boolean
    }
    unsafe fn from_item_unchecked(item: *mut obs_data_item_t) -> Self {
        obs_data_item_get_bool(item)
    }
}

impl FromDataItem for DataObj<'_> {
    fn typ() -> DataType {
        DataType::Object
    }
    unsafe fn from_item_unchecked(item: *mut obs_data_item_t) -> Self {
        Self::from_raw(obs_data_item_get_obj(item))
    }
}

impl FromDataItem for DataArray<'_> {
    fn typ() -> DataType {
        DataType::Array
    }
    unsafe fn from_item_unchecked(item: *mut obs_data_item_t) -> Self {
        Self::from_raw(obs_data_item_get_array(item))
    }
}

pub trait DefaultValue {
    fn set_default<N: Into<ObsString>>(&self, settings: &DataObj, name: N);
}

impl DefaultValue for ObsString {
    fn set_default<N: Into<ObsString>>(&self, settings: &DataObj, name: N) {
        let name = name.into();
        unsafe {
            obs_data_set_default_string(settings.as_ptr() as *mut _, name.as_ptr(), self.as_ptr());
        }
    }
}

impl DefaultValue for f32 {
    fn set_default<N: Into<ObsString>>(&self, settings: &DataObj, name: N) {
        let name = name.into();
        unsafe {
            obs_data_set_default_double(settings.as_ptr() as *mut _, name.as_ptr(), (*self).into());
        }
    }
}

/// A smart pointer to `obs_data_t`
pub struct DataObj<'parent> {
    raw: *mut obs_data_t,
    _parent: PhantomData<&'parent DataObj<'parent>>,
}

impl PtrWrapper for DataObj<'_> {
    type Pointer = obs_data_t;

    unsafe fn from_raw(raw: *mut Self::Pointer) -> Self {
        Self {
            raw,
            _parent: PhantomData,
        }
    }

    fn as_ptr(&self) -> *const Self::Pointer {
        self.raw
    }
}

impl DataObj<'_> {
    /// Creates a empty data object
    pub fn new() -> Self {
        unsafe {
            let raw = obs_data_create();
            Self::from_raw(raw)
        }
    }
    /// Loads data into a object from a JSON string.
    pub fn from_json(json_str: impl Into<ObsString>) -> Option<Self> {
        let json_str = json_str.into();
        unsafe {
            let raw = obs_data_create_from_json(json_str.as_ptr());
            if raw.is_null() {
                None
            } else {
                Some(Self::from_raw(raw))
            }
        }
    }
    /// Loads data into a object from a JSON file.
    /// * `backup_ext`: optional backup file path in case the original file is bad.
    pub fn from_json_file(
        json_file: impl Into<ObsString>,
        backup_ext: impl Into<Option<ObsString>>,
    ) -> Option<Self> {
        let json_file = json_file.into();

        unsafe {
            let raw = if let Some(backup_ext) = backup_ext.into() {
                obs_data_create_from_json_file_safe(json_file.as_ptr(), backup_ext.as_ptr())
            } else {
                obs_data_create_from_json_file(json_file.as_ptr())
            };
            if raw.is_null() {
                None
            } else {
                Some(Self::from_raw(raw))
            }
        }
    }
    /// Fetches a property from this object. Numbers are implicitly casted.
    pub fn get<T: FromDataItem, N: Into<ObsString>>(&self, name: N) -> Option<T> {
        let name = name.into();
        let mut item_ptr = unsafe { obs_data_item_byname(self.as_ptr() as *mut _, name.as_ptr()) };
        if item_ptr.is_null() {
            return None;
        }
        // Release it immediately since it is also referenced by this object.
        unsafe {
            obs_data_item_release(&mut item_ptr);
        }
        assert!(!item_ptr.is_null()); // We should not be the last holder

        let typ = unsafe { DataType::from_item(item_ptr) };

        if typ == T::typ() {
            Some(unsafe { T::from_item_unchecked(item_ptr) })
        } else {
            None
        }
    }
    /// Creates a JSON representation of this object.
    pub fn get_json(&self) -> Option<String> {
        unsafe {
            let ptr = obs_data_get_json(self.raw);
            if ptr.is_null() {
                None
            } else {
                let slice = CStr::from_ptr(ptr);
                Some(slice.to_string_lossy().into_owned())
            }
        }
    }
    /// Clears all values.
    pub fn clear(&mut self) {
        unsafe {
            obs_data_clear(self.raw);
        }
    }

    pub fn remove(&mut self, name: impl Into<ObsString>) {
        let name = name.into();
        unsafe {
            obs_data_erase(self.raw, name.as_ptr());
        }
    }

    pub fn set_default<T: DefaultValue, N: Into<ObsString>>(&mut self, name: N, value: T) {
        value.set_default(&self, name);
    }
}

impl Drop for DataObj<'_> {
    fn drop(&mut self) {
        unsafe {
            obs_data_release(self.raw);
        }
    }
}

pub struct DataArray<'parent> {
    raw: *mut obs_data_array_t,
    _parent: PhantomData<&'parent DataArray<'parent>>,
}

impl PtrWrapper for DataArray<'_> {
    type Pointer = obs_data_array_t;

    unsafe fn from_raw(raw: *mut Self::Pointer) -> Self {
        Self {
            raw,
            _parent: PhantomData,
        }
    }

    fn as_ptr(&self) -> *const Self::Pointer {
        self.raw
    }
}

impl DataArray<'_> {
    pub fn len(&self) -> usize {
        unsafe { obs_data_array_count(self.raw) as usize }
    }

    pub fn get(&self, index: usize) -> Option<DataObj> {
        let ptr = unsafe { obs_data_array_item(self.raw, index as size_t) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { DataObj::from_raw(ptr) })
        }
    }
}

impl Drop for DataArray<'_> {
    fn drop(&mut self) {
        unsafe {
            obs_data_array_release(self.raw);
        }
    }
}
