#![allow(non_upper_case_globals)]
use std::{borrow::Cow, ffi::CStr, marker::PhantomData};

use obs_sys::{
    obs_data_array_count, obs_data_array_item, obs_data_array_release, obs_data_array_t,
    obs_data_create, obs_data_item_byname, obs_data_item_get_array, obs_data_item_get_bool,
    obs_data_item_get_double, obs_data_item_get_int, obs_data_item_get_obj,
    obs_data_item_get_string, obs_data_item_gettype, obs_data_item_numtype, obs_data_item_release,
    obs_data_item_t, obs_data_number_type, obs_data_number_type_OBS_DATA_NUM_DOUBLE,
    obs_data_number_type_OBS_DATA_NUM_INT, obs_data_release, obs_data_t, obs_data_type,
    obs_data_type_OBS_DATA_ARRAY, obs_data_type_OBS_DATA_BOOLEAN, obs_data_type_OBS_DATA_NUMBER,
    obs_data_type_OBS_DATA_OBJECT, obs_data_type_OBS_DATA_STRING, size_t,
};

use crate::string::ObsString;

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

impl FromDataItem for i64 {
    fn typ() -> DataType {
        DataType::Int
    }
    unsafe fn from_item_unchecked(item: *mut obs_data_item_t) -> Self {
        obs_data_item_get_int(item)
    }
}

impl FromDataItem for f64 {
    fn typ() -> DataType {
        DataType::Double
    }
    unsafe fn from_item_unchecked(item: *mut obs_data_item_t) -> Self {
        obs_data_item_get_double(item)
    }
}

impl FromDataItem for bool {
    fn typ() -> DataType {
        DataType::Int
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
        Self::new_unchecked(obs_data_item_get_obj(item))
    }
}

impl FromDataItem for DataArray<'_> {
    fn typ() -> DataType {
        DataType::Array
    }
    unsafe fn from_item_unchecked(item: *mut obs_data_item_t) -> Self {
        Self::new_unchecked(obs_data_item_get_array(item))
    }
}

/// A smart pointer to `obs_data_t`
pub struct DataObj<'parent> {
    raw: *mut obs_data_t,
    _parent: PhantomData<&'parent DataObj<'parent>>,
}

impl DataObj<'_> {
    /// Creates a empty data object
    pub fn new() -> Self {
        unsafe {
            let raw = obs_data_create();
            Self::new_unchecked(raw)
        }
    }

    // pub fn from_json(json_str: &str) -> Option<Self> {}

    pub(crate) unsafe fn new_unchecked(raw: *mut obs_data_t) -> Self {
        Self {
            raw,
            _parent: PhantomData,
        }
    }

    pub fn get<T: FromDataItem, N: Into<ObsString>>(&self, name: N) -> Option<T> {
        let name = name.into();
        let mut item_ptr = unsafe { obs_data_item_byname(self.raw, name.as_ptr()) };
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

    pub fn as_raw(&self) -> *mut obs_data_t {
        self.raw
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

impl DataArray<'_> {
    pub(crate) unsafe fn new_unchecked(raw: *mut obs_data_array_t) -> Self {
        Self {
            raw,
            _parent: PhantomData,
        }
    }
    pub fn len(&self) -> usize {
        unsafe { obs_data_array_count(self.raw) as usize }
    }
    pub fn get(&self, index: usize) -> Option<DataObj> {
        let ptr = unsafe { obs_data_array_item(self.raw, index as size_t) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { DataObj::new_unchecked(ptr) })
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
