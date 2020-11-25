#![allow(non_upper_case_globals)]
use std::{
    borrow::Cow,
    ffi::{CStr, CString},
    marker::PhantomData,
};

use obs_sys::{
    obs_data_array_count, obs_data_array_item, obs_data_array_release, obs_data_array_t,
    obs_data_clear, obs_data_create, obs_data_create_from_json, obs_data_create_from_json_file,
    obs_data_create_from_json_file_safe, obs_data_erase, obs_data_get_json, obs_data_item_byname,
    obs_data_item_get_array, obs_data_item_get_bool, obs_data_item_get_double,
    obs_data_item_get_int, obs_data_item_get_obj, obs_data_item_get_string, obs_data_item_gettype,
    obs_data_item_numtype, obs_data_item_release, obs_data_item_t, obs_data_number_type,
    obs_data_number_type_OBS_DATA_NUM_DOUBLE, obs_data_number_type_OBS_DATA_NUM_INT,
    obs_data_release, obs_data_set_default_bool, obs_data_set_default_double,
    obs_data_set_default_int, obs_data_set_default_obj, obs_data_set_default_string, obs_data_t,
    obs_data_type, obs_data_type_OBS_DATA_ARRAY, obs_data_type_OBS_DATA_BOOLEAN,
    obs_data_type_OBS_DATA_NUMBER, obs_data_type_OBS_DATA_OBJECT, obs_data_type_OBS_DATA_STRING,
    size_t,
};

use crate::string::ObsString;
use paste::item;

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
    unsafe fn set_default_unchecked(obj: *mut obs_data_t, name: ObsString, val: Self);
}

impl FromDataItem for Cow<'_, str> {
    fn typ() -> DataType {
        DataType::String
    }
    unsafe fn from_item_unchecked(item: *mut obs_data_item_t) -> Self {
        let ptr = obs_data_item_get_string(item);
        CStr::from_ptr(ptr).to_string_lossy()
    }
    unsafe fn set_default_unchecked(obj: *mut obs_data_t, name: ObsString, val: Self) {
        let s = CString::new(val.as_ref()).unwrap();
        obs_data_set_default_string(obj, name.as_ptr(), s.as_ptr());
    }
}

macro_rules! impl_primitive {
    ($($data_ty:ty, $func_suffix:ident => $rust_ty:ty)*) => {
        $(
            item! {
                impl FromDataItem for $rust_ty {
                    fn typ() -> DataType {
                        $data_ty
                    }
                    unsafe fn from_item_unchecked(item: *mut obs_data_item_t) -> Self {
                        [<obs_data_item_get_ $func_suffix>](item)
                    }
                    unsafe fn set_default_unchecked(obj: *mut obs_data_t, name: ObsString, val: Self) {
                        [<obs_data_set_default_ $func_suffix>](obj, name.as_ptr(),val)
                    }
                }
            }
        )*
    };
}

impl_primitive!(
    DataType::Int, int => i64
    DataType::Double, double => f64
    DataType::Boolean, bool => bool
);

impl FromDataItem for DataObj<'_> {
    fn typ() -> DataType {
        DataType::Object
    }
    unsafe fn from_item_unchecked(item: *mut obs_data_item_t) -> Self {
        Self::new_unchecked(obs_data_item_get_obj(item))
    }
    unsafe fn set_default_unchecked(obj: *mut obs_data_t, name: ObsString, val: Self) {
        obs_data_set_default_obj(obj, name.as_ptr(), val.as_raw());
    }
}

impl FromDataItem for DataArray<'_> {
    fn typ() -> DataType {
        DataType::Array
    }
    unsafe fn from_item_unchecked(item: *mut obs_data_item_t) -> Self {
        Self::new_unchecked(obs_data_item_get_array(item))
    }
    unsafe fn set_default_unchecked(_obj: *mut obs_data_t, _name: ObsString, _val: Self) {
        // obs_data_set_default_array(obj, name.as_ptr(), val.as_raw());
        // TODO: The function above does not actually exist, document that this is no-op or find an alternative
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

    pub fn from_json(json_str: impl Into<ObsString>) -> Option<Self> {
        let json_str = json_str.into();
        unsafe {
            let raw = obs_data_create_from_json(json_str.as_ptr());
            if raw.is_null() {
                None
            } else {
                Some(Self::new_unchecked(raw))
            }
        }
    }

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
                Some(Self::new_unchecked(raw))
            }
        }
    }

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

    /// Sets a default value for the key.
    ///
    /// Notes
    /// -----
    /// Setting a default value for a [`DataArray`] is current a no-op because of a API
    /// problem of OBS.
    pub fn set_default<N: Into<ObsString>, T: FromDataItem, V: Into<T>>(
        &mut self,
        name: N,
        value: V,
    ) {
        unsafe { T::set_default_unchecked(self.as_raw(), name.into(), value.into()) }
    }

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

    pub fn as_raw(&self) -> *mut obs_data_t {
        self.raw
    }

    pub fn clear(&mut self) {
        unsafe {
            obs_data_clear(self.raw);
        }
    }

    pub fn remove<N: Into<ObsString>>(&mut self, name: N) {
        let name = name.into();
        unsafe {
            obs_data_erase(self.raw, name.as_ptr());
        }
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

    pub fn as_raw(&self) -> *mut obs_data_array_t {
        self.raw
    }
}

impl Drop for DataArray<'_> {
    fn drop(&mut self) {
        unsafe {
            obs_data_array_release(self.raw);
        }
    }
}
