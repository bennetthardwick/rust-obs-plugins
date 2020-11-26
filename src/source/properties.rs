use super::ObsString;
use crate::{native_enum, wrapper::PtrWrapper};
use obs_sys::{
    obs_combo_format, obs_combo_format_OBS_COMBO_FORMAT_FLOAT,
    obs_combo_format_OBS_COMBO_FORMAT_INT, obs_combo_format_OBS_COMBO_FORMAT_INVALID,
    obs_combo_format_OBS_COMBO_FORMAT_STRING, obs_combo_type,
    obs_combo_type_OBS_COMBO_TYPE_EDITABLE, obs_combo_type_OBS_COMBO_TYPE_INVALID,
    obs_combo_type_OBS_COMBO_TYPE_LIST, obs_editable_list_type,
    obs_editable_list_type_OBS_EDITABLE_LIST_TYPE_FILES,
    obs_editable_list_type_OBS_EDITABLE_LIST_TYPE_FILES_AND_URLS,
    obs_editable_list_type_OBS_EDITABLE_LIST_TYPE_STRINGS, obs_path_type,
    obs_path_type_OBS_PATH_DIRECTORY, obs_path_type_OBS_PATH_FILE,
    obs_path_type_OBS_PATH_FILE_SAVE, obs_properties_add_bool, obs_properties_add_color,
    obs_properties_add_editable_list, obs_properties_add_float, obs_properties_add_float_slider,
    obs_properties_add_font, obs_properties_add_int, obs_properties_add_int_slider,
    obs_properties_add_list, obs_properties_add_path, obs_properties_add_text,
    obs_properties_create, obs_properties_destroy, obs_properties_t, obs_property_list_add_float,
    obs_property_list_add_int, obs_property_list_add_string, obs_property_list_insert_float,
    obs_property_list_insert_int, obs_property_list_insert_string, obs_property_list_item_disable,
    obs_property_list_item_remove, obs_property_t, obs_text_type, obs_text_type_OBS_TEXT_DEFAULT,
    obs_text_type_OBS_TEXT_MULTILINE, obs_text_type_OBS_TEXT_PASSWORD, size_t,
};

use std::marker::PhantomData;

native_enum!(TextType, obs_text_type {
    Default => OBS_TEXT_DEFAULT,
    Password => OBS_TEXT_PASSWORD,
    Multiline => OBS_TEXT_MULTILINE
});

native_enum!(PathType, obs_path_type {
    File => OBS_PATH_FILE,
    FileSave => OBS_PATH_FILE_SAVE,
    Directory => OBS_PATH_DIRECTORY
});

native_enum!(ComboFormat, obs_combo_format {
    Invalid => OBS_COMBO_FORMAT_INVALID,
    Int => OBS_COMBO_FORMAT_INT,
    Float => OBS_COMBO_FORMAT_FLOAT,
    String => OBS_COMBO_FORMAT_STRING
});

native_enum!(ComboType, obs_combo_type {
    Invalid => OBS_COMBO_TYPE_INVALID,
    Editable => OBS_COMBO_TYPE_EDITABLE,
    List => OBS_COMBO_TYPE_LIST
});

native_enum!(EditableListType, obs_editable_list_type {
    Strings => OBS_EDITABLE_LIST_TYPE_STRINGS,
    Files => OBS_EDITABLE_LIST_TYPE_FILES,
    FilesAndUrls => OBS_EDITABLE_LIST_TYPE_FILES_AND_URLS
});

pub struct Properties {
    pointer: *mut obs_properties_t,
}

impl PtrWrapper for Properties {
    type Pointer = obs_properties_t;

    unsafe fn from_raw(raw: *mut Self::Pointer) -> Self {
        Self { pointer: raw }
    }

    fn as_ptr(&self) -> *const Self::Pointer {
        self.pointer
    }
}

impl Properties {
    pub fn new() -> Self {
        unsafe {
            let ptr = obs_properties_create();
            Self::from_raw(ptr)
        }
    }

    pub fn add_float(
        &mut self,
        name: ObsString,
        description: ObsString,
        min: f64,
        max: f64,
        step: f64,
        slider: bool,
    ) -> &mut Self {
        unsafe {
            if slider {
                obs_properties_add_float_slider(
                    self.pointer,
                    name.as_ptr(),
                    description.as_ptr(),
                    min,
                    max,
                    step,
                );
            } else {
                obs_properties_add_float(
                    self.pointer,
                    name.as_ptr(),
                    description.as_ptr(),
                    min,
                    max,
                    step,
                );
            }
        }
        self
    }

    pub fn add_int(
        &mut self,
        name: ObsString,
        description: ObsString,
        min: i32,
        max: i32,
        step: i32,
        slider: bool,
    ) -> &mut Self {
        unsafe {
            if slider {
                obs_properties_add_int_slider(
                    self.pointer,
                    name.as_ptr(),
                    description.as_ptr(),
                    min,
                    max,
                    step,
                );
            } else {
                obs_properties_add_int(
                    self.pointer,
                    name.as_ptr(),
                    description.as_ptr(),
                    min,
                    max,
                    step,
                );
            }
        }
        self
    }

    pub fn add_bool(&mut self, name: ObsString, description: ObsString) -> &mut Self {
        unsafe {
            obs_properties_add_bool(self.pointer, name.as_ptr(), description.as_ptr());
        }
        self
    }

    pub fn add_text(
        &mut self,
        name: ObsString,
        description: ObsString,
        typ: TextType,
    ) -> &mut Self {
        unsafe {
            obs_properties_add_text(
                self.pointer,
                name.as_ptr(),
                description.as_ptr(),
                typ.into(),
            );
        }
        self
    }

    /// Adds a 'path' property.  Can be a directory or a file.
    ///
    /// If target is a file path, the filters should be this format, separated by
    /// double semi-colens, and extensions separated by space:
    ///
    /// "Example types 1 and 2 (*.ex1 *.ex2);;Example type 3 (*.ex3)"
    ///
    /// Arguments
    /// *  `props`: Properties object
    /// *  `name`: Settings name
    /// *  `description`: Description (display name) of the property
    /// *  `type`: Type of path (directory or file)
    /// *  `filter`: If type is a file path, then describes the file filter
    ///              that the user can browse.  Items are separated via
    ///              double semi-colens.  If multiple file types in a
    ///              filter, separate with space.
    pub fn add_path(
        &mut self,
        name: ObsString,
        description: ObsString,
        typ: PathType,
        filter: impl Into<Option<ObsString>>,
        default_path: impl Into<Option<ObsString>>,
    ) -> &mut Self {
        let filter = filter.into();
        let default_path = default_path.into();

        unsafe {
            obs_properties_add_path(
                self.pointer,
                name.as_ptr(),
                description.as_ptr(),
                typ.into(),
                ObsString::ptr_or_null(&filter),
                ObsString::ptr_or_null(&default_path),
            );
        }
        self
    }

    pub fn add_list<T: ListType>(
        &mut self,
        name: ObsString,
        description: ObsString,
        editable: bool,
    ) -> &mut ListProp<T> {
        unsafe {
            let raw = obs_properties_add_list(
                self.pointer,
                name.as_ptr(),
                description.as_ptr(),
                if editable {
                    ComboType::Editable
                } else {
                    ComboType::List
                }
                .into(),
                T::format().into(),
            );
            ListProp::from_ptr_mut(raw)
        }
    }

    pub fn add_color(&mut self, name: ObsString, description: ObsString) -> &mut Self {
        unsafe {
            obs_properties_add_color(self.pointer, name.as_ptr(), description.as_ptr());
        }
        self
    }

    /// Adds a font selection property.
    ///
    /// A font is an obs_data sub-object which contains the following items:
    /// * face:   face name string
    /// * style:  style name string
    /// * size:   size integer
    /// * flags:  font flags integer (OBS_FONT_* defined above)
    pub fn add_font(&mut self, name: ObsString, description: ObsString) -> &mut Self {
        unsafe {
            obs_properties_add_font(self.pointer, name.as_ptr(), description.as_ptr());
        }
        self
    }

    pub fn add_editable_list(
        &mut self,
        name: ObsString,
        description: ObsString,
        typ: EditableListType,
        filter: impl Into<Option<ObsString>>,
        default_path: impl Into<Option<ObsString>>,
    ) -> &mut Self {
        let filter = filter.into();
        let default_path = default_path.into();
        unsafe {
            obs_properties_add_editable_list(
                self.pointer,
                name.as_ptr(),
                description.as_ptr(),
                typ.into(),
                ObsString::ptr_or_null(&filter),
                ObsString::ptr_or_null(&default_path),
            );
        }
        self
    }
}

impl Drop for Properties {
    fn drop(&mut self) {
        unsafe { obs_properties_destroy(self.pointer) }
    }
}

pub struct ListProp<'props, T> {
    raw: *mut obs_property_t,
    _props: PhantomData<&'props mut Properties>,
    _type: PhantomData<T>,
}

impl<T> PtrWrapper for ListProp<'_, T> {
    type Pointer = obs_property_t;

    unsafe fn from_raw(raw: *mut Self::Pointer) -> Self {
        Self {
            raw,
            _props: PhantomData,
            _type: PhantomData,
        }
    }

    fn as_ptr(&self) -> *const Self::Pointer {
        self.raw
    }
}

impl<T: ListType> ListProp<'_, T> {
    pub fn push(&mut self, name: impl Into<ObsString>, value: T) {
        value.push_into(self.raw, name.into());
    }

    pub fn insert(&mut self, index: usize, name: impl Into<ObsString>, value: T) {
        value.insert_into(self.raw, name.into(), index);
    }

    pub fn remove(&mut self, index: usize) {
        unsafe {
            obs_property_list_item_remove(self.raw, index as size_t);
        }
    }

    pub fn disable(&mut self, index: usize, disabled: bool) {
        unsafe {
            obs_property_list_item_disable(self.raw, index as size_t, disabled);
        }
    }
}

pub trait ListType {
    fn format() -> ComboFormat;
    fn push_into(self, ptr: *mut obs_property_t, name: ObsString);
    fn insert_into(self, ptr: *mut obs_property_t, name: ObsString, index: usize);
}

impl ListType for ObsString {
    fn format() -> ComboFormat {
        ComboFormat::String
    }

    fn push_into(self, ptr: *mut obs_property_t, name: ObsString) {
        unsafe {
            obs_property_list_add_string(ptr, name.as_ptr(), self.as_ptr());
        }
    }

    fn insert_into(self, ptr: *mut obs_property_t, name: ObsString, index: usize) {
        unsafe {
            obs_property_list_insert_string(ptr, index as size_t, name.as_ptr(), self.as_ptr());
        }
    }
}

impl ListType for i64 {
    fn format() -> ComboFormat {
        ComboFormat::Int
    }

    fn push_into(self, ptr: *mut obs_property_t, name: ObsString) {
        unsafe {
            obs_property_list_add_int(ptr, name.as_ptr(), self);
        }
    }

    fn insert_into(self, ptr: *mut obs_property_t, name: ObsString, index: usize) {
        unsafe {
            obs_property_list_insert_int(ptr, index as size_t, name.as_ptr(), self);
        }
    }
}

impl ListType for f64 {
    fn format() -> ComboFormat {
        ComboFormat::Float
    }

    fn push_into(self, ptr: *mut obs_property_t, name: ObsString) {
        unsafe {
            obs_property_list_add_float(ptr, name.as_ptr(), self);
        }
    }

    fn insert_into(self, ptr: *mut obs_property_t, name: ObsString, index: usize) {
        unsafe {
            obs_property_list_insert_float(ptr, index as size_t, name.as_ptr(), self);
        }
    }
}
