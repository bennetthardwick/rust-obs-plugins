#![allow(non_upper_case_globals)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use crate::{native_enum, string::ObsString, wrapper::PtrWrapper};
use num_traits::{one, Bounded, Float, Num, NumCast, PrimInt, ToPrimitive};
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

use std::{marker::PhantomData, ops::RangeBounds, os::raw::c_int};

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

/// Wrapper around [`obs_properties_t`], which is used by
/// OBS to generate a user-friendly configuration UI.
pub struct Properties {
    pointer: *mut obs_properties_t,
}

impl PtrWrapper for Properties {
    type Pointer = obs_properties_t;

    unsafe fn from_raw_unchecked(raw: *mut Self::Pointer) -> Option<Self> {
        if raw.is_null() {
            None
        } else {
            Some(Self { pointer: raw })
        }
    }

    unsafe fn as_ptr(&self) -> *const Self::Pointer {
        self.pointer
    }

    unsafe fn get_ref(ptr: *mut Self::Pointer) -> *mut Self::Pointer {
        ptr
    }

    unsafe fn release(_ptr: *mut Self::Pointer) {}
}

impl Default for Properties {
    fn default() -> Self {
        Properties::new()
    }
}

impl Properties {
    pub fn new() -> Self {
        unsafe {
            let ptr = obs_properties_create();
            Self::from_raw_unchecked(ptr).expect("obs_properties_create")
        }
    }

    pub fn add<T: ObsProp>(
        &mut self,
        name: ObsString,
        description: ObsString,
        prop: T,
    ) -> &mut Self {
        unsafe {
            prop.add_to_props(self.pointer, name, description);
        }
        self
    }

    pub fn add_list<T: ListType>(
        &mut self,
        name: ObsString,
        description: ObsString,
        editable: bool,
    ) -> ListProp<T> {
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
            ListProp::from_raw(raw).expect("obs_properties_add_list")
        }
    }
}

impl Drop for Properties {
    fn drop(&mut self) {
        unsafe { obs_properties_destroy(self.pointer) }
    }
}

/// Wrapper around [`obs_property_t`], which is a list of possible values for a
/// property.
pub struct ListProp<'props, T> {
    raw: *mut obs_property_t,
    _props: PhantomData<&'props mut Properties>,
    _type: PhantomData<T>,
}

impl<T> PtrWrapper for ListProp<'_, T> {
    type Pointer = obs_property_t;

    unsafe fn from_raw_unchecked(raw: *mut Self::Pointer) -> Option<Self> {
        if raw.is_null() {
            None
        } else {
            Some(Self {
                raw,
                _props: PhantomData,
                _type: PhantomData,
            })
        }
    }

    unsafe fn as_ptr(&self) -> *const Self::Pointer {
        self.raw
    }

    unsafe fn get_ref(ptr: *mut Self::Pointer) -> *mut Self::Pointer {
        ptr
    }

    unsafe fn release(_ptr: *mut Self::Pointer) {}
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

enum NumberType {
    Integer,
    Float,
}
/// ## Panics
/// This type of property may cause panic when being added to the properties
/// if the provided `min`, `max` or `step` exceeds [`c_int`].
pub struct NumberProp<T> {
    min: T,
    max: T,
    step: T,
    slider: bool,
    typ: NumberType,
}

impl<T: PrimInt> NumberProp<T> {
    /// Creates a new integer property with step set to 1.
    pub fn new_int() -> Self {
        Self {
            min: T::min_value(),
            max: T::max_value(),
            step: one(),
            slider: false,
            typ: NumberType::Integer,
        }
    }
}

impl<T: Float> NumberProp<T> {
    /// Creates a new float property with a certain step.
    pub fn new_float(step: T) -> Self {
        Self {
            min: T::min_value(),
            max: T::max_value(),
            step,
            slider: false,
            typ: NumberType::Float,
        }
    }
}

impl<T: Num + Bounded + Copy> NumberProp<T> {
    /// Sets the step of the property.
    pub fn with_step(mut self, step: T) -> Self {
        self.step = step;
        self
    }
    /// Sets the range of the property, inclusion and exclustion are calucated
    /// based on the **current step**.
    pub fn with_range<R: RangeBounds<T>>(mut self, range: R) -> Self {
        use std::ops::Bound::*;
        self.min = match range.start_bound() {
            Included(min) => *min,
            Excluded(min) => *min + self.step,
            std::ops::Bound::Unbounded => T::min_value(),
        };

        self.max = match range.end_bound() {
            Included(max) => *max,
            Excluded(max) => *max - self.step,
            std::ops::Bound::Unbounded => T::max_value(),
        };

        self
    }
    /// Sets this property as a slider.
    pub fn with_slider(mut self) -> Self {
        self.slider = true;
        self
    }
}

pub trait ObsProp {
    /// Callback to add this property to a [`obs_properties_t`].
    ///
    /// # Safety
    ///
    /// Must call with a valid pointer.
    unsafe fn add_to_props(self, p: *mut obs_properties_t, name: ObsString, description: ObsString);
}

impl<T: ToPrimitive> ObsProp for NumberProp<T> {
    unsafe fn add_to_props(
        self,
        p: *mut obs_properties_t,
        name: ObsString,
        description: ObsString,
    ) {
        match self.typ {
            NumberType::Integer => {
                let min: c_int = NumCast::from(self.min).unwrap();
                let max: c_int = NumCast::from(self.max).unwrap();
                let step: c_int = NumCast::from(self.step).unwrap();

                if self.slider {
                    obs_properties_add_int_slider(
                        p,
                        name.as_ptr(),
                        description.as_ptr(),
                        min,
                        max,
                        step,
                    );
                } else {
                    obs_properties_add_int(p, name.as_ptr(), description.as_ptr(), min, max, step);
                }
            }
            NumberType::Float => {
                let min: f64 = NumCast::from(self.min).unwrap();
                let max: f64 = NumCast::from(self.max).unwrap();
                let step: f64 = NumCast::from(self.step).unwrap();

                if self.slider {
                    obs_properties_add_float_slider(
                        p,
                        name.as_ptr(),
                        description.as_ptr(),
                        min,
                        max,
                        step,
                    );
                } else {
                    obs_properties_add_float(
                        p,
                        name.as_ptr(),
                        description.as_ptr(),
                        min,
                        max,
                        step,
                    );
                }
            }
        }
    }
}

pub struct BoolProp;

impl ObsProp for BoolProp {
    unsafe fn add_to_props(
        self,
        p: *mut obs_properties_t,
        name: ObsString,
        description: ObsString,
    ) {
        obs_properties_add_bool(p, name.as_ptr(), description.as_ptr());
    }
}
pub struct TextProp {
    typ: TextType,
}

impl TextProp {
    pub fn new(typ: TextType) -> Self {
        Self { typ }
    }
}

impl ObsProp for TextProp {
    unsafe fn add_to_props(
        self,
        p: *mut obs_properties_t,
        name: ObsString,
        description: ObsString,
    ) {
        obs_properties_add_text(p, name.as_ptr(), description.as_ptr(), self.typ.into());
    }
}

pub struct ColorProp;

impl ObsProp for ColorProp {
    unsafe fn add_to_props(
        self,
        p: *mut obs_properties_t,
        name: ObsString,
        description: ObsString,
    ) {
        obs_properties_add_color(p, name.as_ptr(), description.as_ptr());
    }
}

/// Adds a font selection property.
///
/// A font is an obs_data sub-object which contains the following items:
/// * face:   face name string
/// * style:  style name string
/// * size:   size integer
/// * flags:  font flags integer (OBS_FONT_* defined above)
pub struct FontProp;

impl ObsProp for FontProp {
    unsafe fn add_to_props(
        self,
        p: *mut obs_properties_t,
        name: ObsString,
        description: ObsString,
    ) {
        obs_properties_add_font(p, name.as_ptr(), description.as_ptr());
    }
}

/// Adds a 'path' property.  Can be a directory or a file.
///
/// If target is a file path, the filters should be this format, separated by
/// double semi-colens, and extensions separated by space:
///
/// "Example types 1 and 2 (*.ex1 *.ex2);;Example type 3 (*.ex3)"
///
/// Arguments
/// * `props`: Properties object
/// * `name`: Settings name
/// * `description`: Description (display name) of the property
/// * `type`: Type of path (directory or file)
/// * `filter`: If type is a file path, then describes the file filter that the
///   user can browse.  Items are separated via double semi-colens.  If multiple
///   file types in a filter, separate with space.
pub struct PathProp {
    typ: PathType,
    filter: Option<ObsString>,
    default_path: Option<ObsString>,
}

impl PathProp {
    pub fn new(typ: PathType) -> Self {
        Self {
            typ,
            filter: None,
            default_path: None,
        }
    }

    pub fn with_filter(mut self, f: ObsString) -> Self {
        self.filter = Some(f);
        self
    }

    pub fn with_default_path(mut self, d: ObsString) -> Self {
        self.default_path = Some(d);
        self
    }
}

impl ObsProp for PathProp {
    unsafe fn add_to_props(
        self,
        p: *mut obs_properties_t,
        name: ObsString,
        description: ObsString,
    ) {
        obs_properties_add_path(
            p,
            name.as_ptr(),
            description.as_ptr(),
            self.typ.into(),
            ObsString::ptr_or_null(&self.filter),
            ObsString::ptr_or_null(&self.default_path),
        );
    }
}

pub struct EditableListProp {
    typ: EditableListType,
    filter: Option<ObsString>,
    default_path: Option<ObsString>,
}

impl EditableListProp {
    pub fn new(typ: EditableListType) -> Self {
        Self {
            typ,
            filter: None,
            default_path: None,
        }
    }

    pub fn with_filter(mut self, f: ObsString) -> Self {
        self.filter = Some(f);
        self
    }

    pub fn with_default_path(mut self, d: ObsString) -> Self {
        self.default_path = Some(d);
        self
    }
}

impl ObsProp for EditableListProp {
    unsafe fn add_to_props(
        self,
        p: *mut obs_properties_t,
        name: ObsString,
        description: ObsString,
    ) {
        obs_properties_add_editable_list(
            p,
            name.as_ptr(),
            description.as_ptr(),
            self.typ.into(),
            ObsString::ptr_or_null(&self.filter),
            ObsString::ptr_or_null(&self.default_path),
        );
    }
}
