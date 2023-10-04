use super::context::{CreatableSourceContext, GlobalContext, VideoRenderContext};
use super::{traits::*, SourceContext};
use super::{EnumActiveContext, EnumAllContext};
use crate::media::{audio::AudioDataContext, video::VideoDataSourceContext};
use crate::{
    data::DataObj,
    hotkey::{Hotkey, HotkeyCallbacks},
    wrapper::PtrWrapper,
};
use paste::item;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::c_void;
use std::mem::forget;
use std::os::raw::c_char;

use obs_sys::{
    gs_effect_t, obs_audio_data, obs_button_type, obs_data_t, obs_hotkey_id,
    obs_hotkey_register_source, obs_hotkey_t, obs_key_event, obs_media_state, obs_mouse_event,
    obs_properties, obs_source_audio_mix, obs_source_enum_proc_t, obs_source_frame, obs_source_t,
    size_t,
};

struct DataWrapper<D> {
    data: D,
    #[allow(clippy::type_complexity)]
    hotkey_callbacks: HashMap<obs_hotkey_id, Box<dyn FnMut(&mut Hotkey, &mut D)>>,
}

impl<D> DataWrapper<D> {
    pub(crate) unsafe fn register_callbacks(
        &mut self,
        callbacks: HotkeyCallbacks<D>,
        source: *mut obs_source_t,
        data: *mut c_void,
    ) {
        for (name, description, func) in callbacks.into_iter() {
            let id = obs_hotkey_register_source(
                source,
                name.as_ptr(),
                description.as_ptr(),
                Some(hotkey_callback::<D>),
                data,
            );

            self.hotkey_callbacks.insert(id, func);
        }
    }
}

impl<D> From<D> for DataWrapper<D> {
    fn from(data: D) -> Self {
        Self {
            data,
            hotkey_callbacks: HashMap::new(),
        }
    }
}

macro_rules! impl_simple_fn {
    ($($name:ident => $trait:ident $(-> $ret:ty)?)*) => ($(
        item! {
            pub unsafe extern "C" fn $name<D: $trait>(
                data: *mut std::os::raw::c_void,
            ) $(-> $ret)? {
                let wrapper = &mut *(data as *mut DataWrapper<D>);
                D::$name(&mut wrapper.data)
            }
        }
    )*)
}

pub unsafe extern "C" fn get_name<D: GetNameSource>(_type_data: *mut c_void) -> *const c_char {
    D::get_name().as_ptr()
}

impl_simple_fn!(
    get_width => GetWidthSource -> u32
    get_height => GetHeightSource -> u32

    activate => ActivateSource
    deactivate => DeactivateSource
);

pub unsafe extern "C" fn create<D: Sourceable>(
    settings: *mut obs_data_t,
    source: *mut obs_source_t,
) -> *mut c_void {
    let mut global = GlobalContext;
    let settings = DataObj::from_raw(settings);
    let mut context = CreatableSourceContext::from_raw(settings, &mut global);
    let source_context = SourceContext::from_raw(source);

    let data = D::create(&mut context, source_context);

    let wrapper = DataWrapper::from(data);
    forget(context.settings);
    let callbacks = context.hotkey_callbacks;

    let pointer = Box::into_raw(Box::new(wrapper));

    pointer
        .as_mut()
        .unwrap()
        .register_callbacks(callbacks, source, pointer as *mut c_void);

    pointer as *mut c_void
}

pub unsafe extern "C" fn destroy<D>(data: *mut c_void) {
    let wrapper: Box<DataWrapper<D>> = Box::from_raw(data as *mut DataWrapper<D>);
    drop(wrapper);
}

pub unsafe extern "C" fn update<D: UpdateSource>(data: *mut c_void, settings: *mut obs_data_t) {
    let mut global = GlobalContext;
    let data: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    let mut settings = DataObj::from_raw(settings);
    D::update(&mut data.data, &mut settings, &mut global);
    forget(settings);
}

pub unsafe extern "C" fn video_render<D: VideoRenderSource>(
    data: *mut std::os::raw::c_void,
    _effect: *mut gs_effect_t,
) {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    let mut global = GlobalContext;
    let mut render = VideoRenderContext;
    D::video_render(&mut wrapper.data, &mut global, &mut render);
}

pub unsafe extern "C" fn audio_render<D: AudioRenderSource>(
    data: *mut std::os::raw::c_void,
    _ts_out: *mut u64,
    _audio_output: *mut obs_source_audio_mix,
    _mixers: u32,
    _channels: size_t,
    _sample_rate: size_t,
) -> bool {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    let mut global = GlobalContext;
    D::audio_render(&mut wrapper.data, &mut global);
    // TODO: understand what this bool is
    true
}

pub unsafe extern "C" fn get_properties<D: GetPropertiesSource>(
    data: *mut std::os::raw::c_void,
) -> *mut obs_properties {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    let properties = D::get_properties(&mut wrapper.data);
    properties.into_raw()
}

pub unsafe extern "C" fn enum_active_sources<D: EnumActiveSource>(
    data: *mut std::os::raw::c_void,
    _enum_callback: obs_source_enum_proc_t,
    _param: *mut std::os::raw::c_void,
) {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    let context = EnumActiveContext {};
    D::enum_active_sources(&mut wrapper.data, &context);
}

pub unsafe extern "C" fn enum_all_sources<D: EnumAllSource>(
    data: *mut std::os::raw::c_void,
    _enum_callback: obs_source_enum_proc_t,
    _param: *mut std::os::raw::c_void,
) {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    let context = EnumAllContext {};
    D::enum_all_sources(&mut wrapper.data, &context);
}

impl_simple_fn!(
    transition_start => TransitionStartSource
    transition_stop => TransitionStopSource
);

pub unsafe extern "C" fn video_tick<D: VideoTickSource>(
    data: *mut std::os::raw::c_void,
    seconds: f32,
) {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    D::video_tick(&mut wrapper.data, seconds);
}

pub unsafe extern "C" fn filter_audio<D: FilterAudioSource>(
    data: *mut std::os::raw::c_void,
    audio: *mut obs_audio_data,
) -> *mut obs_audio_data {
    let mut context = AudioDataContext::from_raw(audio);
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    D::filter_audio(&mut wrapper.data, &mut context);
    audio
}

pub unsafe extern "C" fn filter_video<D: FilterVideoSource>(
    data: *mut std::os::raw::c_void,
    video: *mut obs_source_frame,
) -> *mut obs_source_frame {
    let mut context = VideoDataSourceContext::from_raw(video);
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    D::filter_video(&mut wrapper.data, &mut context);
    video
}

pub unsafe extern "C" fn media_play_pause<D: MediaPlayPauseSource>(
    data: *mut std::os::raw::c_void,
    pause: bool,
) {
    let wrapper = &mut *(data as *mut DataWrapper<D>);
    D::play_pause(&mut wrapper.data, pause);
}

pub unsafe extern "C" fn media_get_state<D: MediaGetStateSource>(
    data: *mut std::os::raw::c_void,
) -> obs_media_state {
    let wrapper = &mut *(data as *mut DataWrapper<D>);
    D::get_state(&mut wrapper.data).to_native()
}

pub unsafe extern "C" fn media_set_time<D: MediaSetTimeSource>(
    data: *mut std::os::raw::c_void,
    milliseconds: i64,
) {
    let wrapper = &mut *(data as *mut DataWrapper<D>);
    D::set_time(&mut wrapper.data, milliseconds);
}

macro_rules! impl_media {
    ($($name:ident => $trait:ident $(-> $ret:ty)?)*) => ($(
        item! {
            pub unsafe extern "C" fn [<media_$name>]<D: $trait>(
                data: *mut std::os::raw::c_void,
            ) $(-> $ret)? {
                let wrapper = &mut *(data as *mut DataWrapper<D>);
                D::$name(&mut wrapper.data)
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

pub unsafe extern "C" fn get_defaults<D: GetDefaultsSource>(settings: *mut obs_data_t) {
    let mut settings = DataObj::from_raw(settings);
    D::get_defaults(&mut settings);
    forget(settings);
}

pub unsafe extern "C" fn hotkey_callback<D>(
    data: *mut c_void,
    id: obs_hotkey_id,
    hotkey: *mut obs_hotkey_t,
    pressed: bool,
) {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);

    let data = &mut wrapper.data;
    let hotkey_callbacks = &mut wrapper.hotkey_callbacks;
    let mut key = Hotkey::from_raw(hotkey, pressed);

    if let Some(callback) = hotkey_callbacks.get_mut(&id) {
        callback(&mut key, data);
    }
}

pub unsafe extern "C" fn mouse_click<D: MouseClickSource>(
    data: *mut std::os::raw::c_void,
    event: *const obs_mouse_event,
    type_: i32,
    mouse_up: bool,
    click_count: u32,
) {
    let wrapper = &mut *(data as *mut DataWrapper<D>);
    D::mouse_click(
        &mut wrapper.data,
        *event,
        super::MouseButton::try_from(type_ as obs_button_type).unwrap(),
        !mouse_up,
        click_count as u8,
    )
}

pub unsafe extern "C" fn mouse_move<D: MouseMoveSource>(
    data: *mut std::os::raw::c_void,
    event: *const obs_mouse_event,
    mouse_leave: bool,
) {
    let wrapper = &mut *(data as *mut DataWrapper<D>);
    D::mouse_move(&mut wrapper.data, *event, mouse_leave);
}

pub unsafe extern "C" fn mouse_wheel<D: MouseWheelSource>(
    data: *mut std::os::raw::c_void,
    event: *const obs_mouse_event,
    xdelta: i32,
    ydelta: i32,
) {
    let wrapper = &mut *(data as *mut DataWrapper<D>);
    D::mouse_wheel(&mut wrapper.data, *event, xdelta, ydelta);
}

pub unsafe extern "C" fn key_click<D: KeyClickSource>(
    data: *mut std::os::raw::c_void,
    event: *const obs_key_event,
    key_up: bool,
) {
    let wrapper = &mut *(data as *mut DataWrapper<D>);
    D::key_click(&mut wrapper.data, *event, !key_up);
}

pub unsafe extern "C" fn focus<D: FocusSource>(data: *mut std::os::raw::c_void, focus: bool) {
    let wrapper = &mut *(data as *mut DataWrapper<D>);
    D::focus(&mut wrapper.data, focus);
}
