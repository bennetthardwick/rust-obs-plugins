use super::context::ActiveContext;
use super::traits::*;
use super::{ SettingsContext, SourceContext };
use std::ffi::c_void;
use std::marker::PhantomData;
use std::os::raw::c_char;

use obs_sys::{
    obs_data_t, obs_source_info, obs_source_t, obs_source_type,
    obs_source_type_OBS_SOURCE_TYPE_FILTER, obs_source_type_OBS_SOURCE_TYPE_INPUT,
    obs_source_type_OBS_SOURCE_TYPE_SCENE, obs_source_type_OBS_SOURCE_TYPE_TRANSITION,
};

pub unsafe extern "C" fn get_name<F: GetNameSource>(type_data: *mut c_void) -> *const c_char {
    let name = F::get_name();
    name.as_bytes().as_ptr() as *const c_char
}

pub unsafe extern "C" fn get_width<D, F: GetWidthSource<D>>(data: *mut c_void) -> u32 {
    let data: &mut D = &mut *(data as *mut D);
    F::get_width(&data)
}

pub unsafe extern "C" fn get_height<D, F: GetHeightSource<D>>(data: *mut c_void) -> u32 {
    let data: &mut D = &mut *(data as *mut D);
    F::get_height(&data)
}

pub unsafe extern "C" fn create<D, F: CreatableSource<D>>(
    settings: *mut obs_data_t,
    source: *mut obs_source_t,
) -> *mut c_void {
    let settings = SettingsContext::from_raw(settings);
    let source = SourceContext { source };
    let data = Box::new(F::create(&settings, source));
    Box::into_raw(data) as *mut c_void
}

pub unsafe extern "C" fn update<D, F: UpdateSource<D>>(
    data: *mut c_void,
    settings: *mut obs_data_t,
) {
    let active = ActiveContext::default();
    let settings = SettingsContext::from_raw(settings);
    let data: &mut D = &mut *(data as *mut D);
    F::update(data, &settings, &active);
}
