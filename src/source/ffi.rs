use super::audio::AudioDataContext;
use super::context::{GlobalContext, VideoRenderContext};
use super::properties::{Properties, Property, SettingsContext};
use super::traits::*;
use super::{EnumActiveContext, EnumAllContext, SourceContext};
use std::ffi::c_void;
use std::os::raw::c_char;

use obs_sys::{
    gs_effect_t, obs_audio_data, obs_data_t, obs_properties, obs_properties_create,
    obs_source_audio_mix, obs_source_enum_proc_t, obs_source_t, size_t,
};

struct DataWrapper<D> {
    data: Option<D>,
    properties: Vec<Property>,
}

impl<D> Default for DataWrapper<D> {
    fn default() -> Self {
        Self {
            data: None,
            properties: vec![],
        }
    }
}

impl<D> From<D> for DataWrapper<D> {
    fn from(data: D) -> Self {
        Self {
            data: Some(data),
            properties: vec![],
        }
    }
}

pub unsafe extern "C" fn get_name<D, F: GetNameSource<D>>(
    _type_data: *mut c_void,
) -> *const c_char {
    F::get_name().as_ptr()
}

pub unsafe extern "C" fn get_width<D, F: GetWidthSource<D>>(data: *mut c_void) -> u32 {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    F::get_width(&mut wrapper.data)
}

pub unsafe extern "C" fn get_height<D, F: GetHeightSource<D>>(data: *mut c_void) -> u32 {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    F::get_height(&wrapper.data)
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
    let mut wrapper = DataWrapper::default();
    let mut settings = SettingsContext::from_raw(settings, &wrapper.properties);

    let source = SourceContext { source };
    let mut global = GlobalContext::default();

    let data = F::create(&mut settings, source, &mut global);

    wrapper.data = Some(data);

    Box::into_raw(Box::new(wrapper)) as *mut c_void
}

pub unsafe extern "C" fn destroy<D>(data: *mut c_void) {
    let wrapper: Box<DataWrapper<D>> = Box::from_raw(data as *mut DataWrapper<D>);
    drop(wrapper);
}

pub unsafe extern "C" fn update<D, F: UpdateSource<D>>(
    data: *mut c_void,
    settings: *mut obs_data_t,
) {
    let mut global = GlobalContext::default();
    let data: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    let mut settings = SettingsContext::from_raw(settings, &data.properties);
    F::update(&mut data.data, &mut settings, &mut global);
}

pub unsafe extern "C" fn video_render<D, F: VideoRenderSource<D>>(
    data: *mut ::std::os::raw::c_void,
    _effect: *mut gs_effect_t,
) {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    let mut global = GlobalContext::default();
    let mut render = VideoRenderContext::default();
    F::video_render(&mut wrapper.data, &mut global, &mut render);
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
    let mut global = GlobalContext::default();
    F::audio_render(&mut wrapper.data, &mut global);
    // TODO: understand what this bool is
    true
}

pub unsafe extern "C" fn get_properties<D, F: GetPropertiesSource<D>>(
    data: *mut ::std::os::raw::c_void,
) -> *mut obs_properties {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);

    let mut properties = Properties::from_raw(obs_properties_create(), &mut wrapper.properties);

    F::get_properties(&mut wrapper.data, &mut properties);

    properties.into_raw()
}

pub unsafe extern "C" fn enum_active_sources<D, F: EnumActiveSource<D>>(
    data: *mut ::std::os::raw::c_void,
    _enum_callback: obs_source_enum_proc_t,
    _param: *mut ::std::os::raw::c_void,
) {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    let context = EnumActiveContext {};
    F::enum_active_sources(&mut wrapper.data, &context);
}

pub unsafe extern "C" fn enum_all_sources<D, F: EnumAllSource<D>>(
    data: *mut ::std::os::raw::c_void,
    _enum_callback: obs_source_enum_proc_t,
    _param: *mut ::std::os::raw::c_void,
) {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    let context = EnumAllContext {};
    F::enum_all_sources(&mut wrapper.data, &context);
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

pub unsafe extern "C" fn video_tick<D, F: VideoTickSource<D>>(
    data: *mut ::std::os::raw::c_void,
    seconds: f32,
) {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    F::video_tick(&mut wrapper.data, seconds);
}

pub unsafe extern "C" fn filter_audio<D, F: FilterAudioSource<D>>(
    data: *mut ::std::os::raw::c_void,
    audio: *mut obs_audio_data,
) -> *mut obs_audio_data {
    let mut context = AudioDataContext::from_raw(audio);
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    F::filter_audio(&mut wrapper.data, &mut context);
    audio
}
