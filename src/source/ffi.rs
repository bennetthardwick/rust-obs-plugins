use super::audio::AudioDataContext;
use super::context::{GlobalContext, VideoRenderContext};
use super::properties::Properties;
use super::traits::*;
use super::{EnumActiveContext, EnumAllContext, SourceContext};
use paste::item;
use crate::data::DataObj;
use std::ffi::c_void;
use std::mem::forget;
use std::os::raw::c_char;

use obs_sys::{
    gs_effect_t, obs_audio_data, obs_data_t, obs_media_state, obs_properties,
    obs_properties_create, obs_source_audio_mix, obs_source_enum_proc_t, obs_source_t, size_t,
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

macro_rules! impl_simple_fn {
    ($($name:ident => $trait:ident $(-> $ret:ty)?)*) => ($(
        item! {
            pub unsafe extern "C" fn $name<D, F: $trait<D>>(
                data: *mut ::std::os::raw::c_void,
            ) $(-> $ret)? {
                let wrapper = &mut *(data as *mut DataWrapper<D>);
                F::$name(&mut wrapper.data)
            }
        }
    )*)
}

pub unsafe extern "C" fn get_name<D, F: GetNameSource<D>>(
    _type_data: *mut c_void,
) -> *const c_char {
    F::get_name().as_ptr()
}

impl_simple_fn!(
    get_width => GetWidthSource -> u32
    get_height => GetHeightSource -> u32

    activate => ActivateSource
    deactivate => DeactivateSource
);

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
    let mut settings = DataObj::new_unchecked(settings);

    let source = SourceContext { source };
    let mut global = GlobalContext::default();

    let data = F::create(&mut settings, source, &mut global);
    wrapper.data = Some(data);
    forget(settings);
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
    let mut settings = DataObj::new_unchecked(settings);
    F::update(&mut data.data, &mut settings, &mut global);
    forget(settings);
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

impl_simple_fn!(
    transition_start => TransitionStartSource
    transition_stop => TransitionStopSource
);

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

pub unsafe extern "C" fn media_play_pause<D, F: MediaPlayPauseSource<D>>(
    data: *mut ::std::os::raw::c_void,
    pause: bool,
) {
    let wrapper = &mut *(data as *mut DataWrapper<D>);
    F::play_pause(&mut wrapper.data, pause);
}

pub unsafe extern "C" fn media_get_state<D, F: MediaGetStateSource<D>>(
    data: *mut ::std::os::raw::c_void,
) -> obs_media_state {
    let wrapper = &mut *(data as *mut DataWrapper<D>);
    F::get_state(&mut wrapper.data).to_native()
}

macro_rules! impl_media {
    ($($name:ident => $trait:ident $(-> $ret:ty)?)*) => ($(
        item! {
            pub unsafe extern "C" fn [<media_$name>]<D, F: $trait<D>>(
                data: *mut ::std::os::raw::c_void,
            ) $(-> $ret)? {
                let wrapper = &mut *(data as *mut DataWrapper<D>);
                F::$name(&mut wrapper.data)
            }
        }
    )*)
}

impl_media!(
    stop => MediaStopSource
    restart => MediaRestartSource
    next => MediaNextSource
    previous => MediaPreviousSource
    get_duration => MediaGetDurationSource -> i64
    get_time => MediaGetTimeSource -> i64
);

pub unsafe extern "C" fn get_defaults<D, F: GetDefaultsSource<D>>(settings: *mut obs_data_t) {
    let mut settings = DataObj::new_unchecked(settings);
    F::get_defaults(&mut settings);
    forget(settings);
}

