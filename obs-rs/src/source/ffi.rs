use super::context::ActiveContext;
use super::properties::Properties;
use super::traits::*;
use super::{EnumActiveContext, EnumAllContext, SettingsContext, SourceContext};
use std::ffi::c_void;
use std::marker::PhantomData;
use std::os::raw::c_char;

use obs_sys::{
    gs_effect_t, obs_data_t, obs_properties, obs_properties_create, obs_source_audio_mix,
    obs_source_enum_proc_t, obs_source_info, obs_source_t, obs_source_type,
    obs_source_type_OBS_SOURCE_TYPE_FILTER, obs_source_type_OBS_SOURCE_TYPE_INPUT,
    obs_source_type_OBS_SOURCE_TYPE_SCENE, obs_source_type_OBS_SOURCE_TYPE_TRANSITION, size_t,
};

struct DataWrapper<D> {
    data: Option<D>,
}

impl<D> Default for DataWrapper<D> {
    fn default() -> Self {
        Self { data: None }
    }
}

impl<D> From<D> for DataWrapper<D> {
    fn from(data: D) -> Self {
        Self { data: Some(data) }
    }
}

pub unsafe extern "C" fn get_name<F: GetNameSource>(type_data: *mut c_void) -> *const c_char {
    F::get_name().as_ptr()
}

pub unsafe extern "C" fn get_width<D, F: GetWidthSource<D>>(data: *mut c_void) -> u32 {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    F::get_width(&mut wrapper.data)
}

pub unsafe extern "C" fn get_height<D, F: GetHeightSource<D>>(data: *mut c_void) -> u32 {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    F::get_height(&mut wrapper.data)
}

pub unsafe extern "C" fn create_default_data<D>(
    _settings: *mut obs_data_t,
    _source: *mut obs_source_t,
) -> *mut c_void {
    let data = Box::new(DataWrapper::<D>::default());
    Box::into_raw(data) as *mut c_void
}

pub unsafe extern "C" fn create<D, F: CreatableSource<D>>(
    settings: *mut obs_data_t,
    source: *mut obs_source_t,
) -> *mut c_void {
    let settings = SettingsContext::from_raw(settings);
    let source = SourceContext { source };
    let data = Box::new(DataWrapper::from(F::create(&settings, source)));
    Box::into_raw(data) as *mut c_void
}

pub unsafe extern "C" fn destroy<D>(data: *mut c_void) {
    let wrapper: Box<DataWrapper<D>> = Box::from_raw(data as *mut DataWrapper<D>);
    drop(wrapper);
}

pub unsafe extern "C" fn update<D, F: UpdateSource<D>>(
    data: *mut c_void,
    settings: *mut obs_data_t,
) {
    let mut active = ActiveContext::default();
    let settings = SettingsContext::from_raw(settings);
    let data: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    F::update(&mut data.data, &settings, &mut active);
}

pub unsafe extern "C" fn video_render<D, F: VideoRenderSource<D>>(
    data: *mut ::std::os::raw::c_void,
    _effect: *mut gs_effect_t,
) {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    let mut active = ActiveContext::default();
    F::video_render(&mut wrapper.data, &mut active);
}

pub unsafe extern "C" fn audio_render<D, F: AudioRenderSource<D>>(
    data: *mut ::std::os::raw::c_void,
    _ts_out: *mut u64,
    _audio_output: *mut obs_source_audio_mix,
    _mixers: u32,
    _channels: size_t,
    _sample_rate: size_t,
) -> bool {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    let mut active = ActiveContext::default();
    F::audio_render(&mut wrapper.data, &mut active);
    // TODO: understand what this bool is
    true
}

pub unsafe extern "C" fn get_properties<D, F: GetPropertiesSource<D>>(
    data: *mut ::std::os::raw::c_void,
) -> *mut obs_properties {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);

    let mut properties = Properties::from_raw(obs_properties_create());

    F::get_properties(&mut wrapper.data, &mut properties);

    properties.into_raw()
}

pub unsafe extern "C" fn enum_active_sources<D, F: EnumActiveSource<D>>(
    data: *mut ::std::os::raw::c_void,
    _enum_callback: obs_source_enum_proc_t,
    _param: *mut ::std::os::raw::c_void,
) {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    let mut context = EnumActiveContext {};
    F::enum_active_sources(&mut wrapper.data, &mut context);
}

pub unsafe extern "C" fn enum_all_sources<D, F: EnumAllSource<D>>(
    data: *mut ::std::os::raw::c_void,
    _enum_callback: obs_source_enum_proc_t,
    _param: *mut ::std::os::raw::c_void,
) {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    let mut context = EnumAllContext {};
    F::enum_all_sources(&mut wrapper.data, &mut context);
}

pub unsafe extern "C" fn transition_start<D, F: TransitionStartSource<D>>(
    data: *mut ::std::os::raw::c_void,
) {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    F::transition_start(&mut wrapper.data);
}

pub unsafe extern "C" fn transition_stop<D, F: TransitionStopSource<D>>(
    data: *mut ::std::os::raw::c_void,
) {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    F::transition_stop(&mut wrapper.data);
}
