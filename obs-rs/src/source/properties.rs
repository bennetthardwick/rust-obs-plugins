use super::ObsString;
use obs_sys::{obs_data_get_double, obs_data_t, obs_properties_add_float_slider, obs_properties_t};
use std::os::raw::c_char;

pub struct ParamBuilder {}

pub(crate) struct Property {
    name: &'static str,
    property_type: PropertyType,
}

#[derive(Eq, PartialEq)]
enum PropertyType {
    Float,
}

pub struct Properties<'a> {
    pointer: *mut obs_properties_t,
    properties: &'a mut Vec<Property>,
}

impl<'a> Properties<'a> {
    pub(crate) unsafe fn from_raw(
        pointer: *mut obs_properties_t,
        properties: &'a mut Vec<Property>,
    ) -> Self {
        Self {
            pointer,
            properties,
        }
    }

    pub unsafe fn into_raw(self) -> *mut obs_properties_t {
        self.pointer
    }

    pub fn add_float_slider(
        &mut self,
        name: ObsString,
        description: ObsString,
        min: f64,
        max: f64,
        step: f64,
    ) -> &mut Self {
        unsafe {
            self.properties.push(Property {
                name: name.as_str(),
                property_type: PropertyType::Float,
            });
            obs_properties_add_float_slider(
                self.pointer,
                name.as_ptr(),
                description.as_ptr(),
                min,
                max,
                step,
            );
        }
        self
    }
}

pub struct SettingsContext<'a> {
    settings: *mut obs_data_t,
    properties: &'a Vec<Property>,
}

impl<'a> SettingsContext<'a> {
    pub(crate) unsafe fn from_raw(
        settings: *mut obs_data_t,
        properties: &'a Vec<Property>,
    ) -> Self {
        SettingsContext {
            settings,
            properties,
        }
    }

    pub(crate) unsafe fn as_raw(&self) -> *mut obs_data_t {
        self.settings
    }

    pub fn get_float(&self, param: ObsString) -> Option<f64> {
        self.properties
            .iter()
            .find(
                |Property {
                     name,
                     property_type,
                 }| {
                    property_type == &PropertyType::Float && *name == param.as_str()
                },
            )
            .map(|_| unsafe { obs_data_get_double(self.settings, param.as_ptr()) } as f64)
    }
}
