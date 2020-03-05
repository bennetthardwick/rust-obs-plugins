use super::ObsString;
use obs_sys::{
    obs_data_get_double, obs_data_get_int, obs_data_get_json, obs_data_t, obs_properties_add_float,
    obs_properties_add_float_slider, obs_properties_add_int, obs_properties_t,
};
use std::ffi::CStr;

use serde_json::Value;

pub struct ParamBuilder {}

pub(crate) struct Property {
    name: &'static str,
    property_type: PropertyType,
}

#[derive(Eq, PartialEq)]
enum PropertyType {
    Float,
    Int,
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

    /// # Safety
    /// Modifying this pointer could cause UB
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

    pub fn add_float(
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
            obs_properties_add_float(
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

    pub fn add_int(
        &mut self,
        name: ObsString,
        description: ObsString,
        min: i32,
        max: i32,
        step: i32,
    ) -> &mut Self {
        unsafe {
            self.properties.push(Property {
                name: name.as_str(),
                property_type: PropertyType::Int,
            });
            obs_properties_add_int(
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
    properties: &'a [Property],
    init_data: Option<Value>,
}

impl<'a> SettingsContext<'a> {
    pub(crate) unsafe fn from_raw(settings: *mut obs_data_t, properties: &'a [Property]) -> Self {
        SettingsContext {
            settings,
            properties,
            init_data: None,
        }
    }

    pub(crate) unsafe fn as_raw(&self) -> *mut obs_data_t {
        self.settings
    }

    fn get_data(&mut self) -> &Option<Value> {
        let mut json_data: Option<Value> = None;

        if self.init_data.is_none() {
            let data = unsafe { CStr::from_ptr(obs_data_get_json(self.settings)) };
            if let Some(value) = data
                .to_str()
                .ok()
                .and_then(|x| serde_json::from_str(x).ok())
            {
                json_data = Some(value);
            }
        }

        if let Some(data) = json_data {
            self.init_data.replace(data);
        }

        &self.init_data
    }

    pub fn get_float(&mut self, param: ObsString) -> Option<f64> {
        if self
            .properties
            .iter()
            .any(|p| p.property_type == PropertyType::Float && p.name == param.as_str())
        {
            Some(unsafe { obs_data_get_double(self.settings, param.as_ptr()) })
        } else {
            if let Some(data) = self.get_data() {
                let param = param.as_str();
                if let Some(val) = data.get(&param[..param.len() - 1]) {
                    return val.as_f64();
                }
            }

            None
        }
    }

    pub fn get_int(&mut self, param: ObsString) -> Option<i32> {
        if self
            .properties
            .iter()
            .any(|p| p.property_type == PropertyType::Int && p.name == param.as_str())
        {
            Some(unsafe { obs_data_get_int(self.settings, param.as_ptr()) } as i32)
        } else {
            if let Some(data) = self.get_data() {
                let param = param.as_str();
                if let Some(val) = data.get(&param[..param.len() - 1]) {
                    if let Some(val) = val.as_i64() {
                        return Some(val as i32);
                    }
                }
            }

            None
        }
    }
}
