use obs_sys::{
    obs_source_type,
    obs_source_type_OBS_SOURCE_TYPE_INPUT,
    obs_source_type_OBS_SOURCE_TYPE_SCENE,
    obs_source_type_OBS_SOURCE_TYPE_FILTER,
    obs_source_type_OBS_SOURCE_TYPE_TRANSITION
};

pub enum SourceType {
    INPUT,
    SCENE,
    FILTER,
    TRANSITION
}

impl SourceType {
    pub(crate) fn to_native(&self) -> obs_source_type {
        match self {
            SourceType::INPUT => obs_source_type_OBS_SOURCE_TYPE_INPUT,
            SourceType::SCENE => obs_source_type_OBS_SOURCE_TYPE_SCENE,
            SourceType::FILTER => obs_source_type_OBS_SOURCE_TYPE_FILTER,
            SourceType::TRANSITION => obs_source_type_OBS_SOURCE_TYPE_TRANSITION,
        }
    }
}

pub trait Source {
    fn get_id(&self) -> &'static str;

}

// id: *const c_char
// type_: obs_source_type
// output_flags: u32
// get_name: Option<unsafe extern "C" fn(type_data: *mut c_void) -> *const c_char>
// create: Option<unsafe extern "C" fn(settings: *mut obs_data_t, source: *mut obs_source_t) -> *mut c_void>
// destroy: Option<unsafe extern "C" fn(data: *mut c_void)>
// get_width: Option<unsafe extern "C" fn(data: *mut c_void) -> u32>
// get_height: Option<unsafe extern "C" fn(data: *mut c_void) -> u32>
// get_defaults: Option<unsafe extern "C" fn(settings: *mut obs_data_t)>
// get_properties: Option<unsafe extern "C" fn(data: *mut c_void) -> *mut obs_properties_t>
// update: Option<unsafe extern "C" fn(data: *mut c_void, settings: *mut obs_data_t)>
// activate: Option<unsafe extern "C" fn(data: *mut c_void)>
// deactivate: Option<unsafe extern "C" fn(data: *mut c_void)>
// show: Option<unsafe extern "C" fn(data: *mut c_void)>
// hide: Option<unsafe extern "C" fn(data: *mut c_void)>
// video_tick: Option<unsafe extern "C" fn(data: *mut c_void, seconds: f32)>
// video_render: Option<unsafe extern "C" fn(data: *mut c_void, effect: *mut gs_effect_t)>
// filter_video: Option<unsafe extern "C" fn(data: *mut c_void, frame: *mut obs_source_frame) -> *mut obs_source_frame>
// filter_audio: Option<unsafe extern "C" fn(data: *mut c_void, audio: *mut obs_audio_data) -> *mut obs_audio_data>
// enum_active_sources: Option<unsafe extern "C" fn(data: *mut c_void, enum_callback: obs_source_enum_proc_t, param: *mut c_void)>
// save: Option<unsafe extern "C" fn(data: *mut c_void, settings: *mut obs_data_t)>
// load: Option<unsafe extern "C" fn(data: *mut c_void, settings: *mut obs_data_t)>
// mouse_click: Option<unsafe extern "C" fn(data: *mut c_void, event: *const obs_mouse_event, type_: i32, mouse_up: bool, click_count: u32)>
// mouse_move: Option<unsafe extern "C" fn(data: *mut c_void, event: *const obs_mouse_event, mouse_leave: bool)>
// mouse_wheel: Option<unsafe extern "C" fn(data: *mut c_void, event: *const obs_mouse_event, x_delta: c_int, y_delta: c_int)>
// focus: Option<unsafe extern "C" fn(data: *mut c_void, focus: bool)>
// key_click: Option<unsafe extern "C" fn(data: *mut c_void, event: *const obs_key_event, key_up: bool)>
// filter_remove: Option<unsafe extern "C" fn(data: *mut c_void, source: *mut obs_source_t)>
// type_data: *mut c_void
// free_type_data: Option<unsafe extern "C" fn(type_data: *mut c_void)>
// audio_render: Option<unsafe extern "C" fn(data: *mut c_void, ts_out: *mut u64, audio_output: *mut obs_source_audio_mix, mixers: u32, channels: size_t, sample_rate: size_t) -> bool>
// enum_all_sources: Option<unsafe extern "C" fn(data: *mut c_void, enum_callback: obs_source_enum_proc_t, param: *mut c_void)>
// transition_start: Option<unsafe extern "C" fn(data: *mut c_void)>
// transition_stop: Option<unsafe extern "C" fn(data: *mut c_void)>
// get_defaults2: Option<unsafe extern "C" fn(type_data: *mut c_void, settings: *mut obs_data_t)>
// get_properties2: Option<unsafe extern "C" fn(data: *mut c_void, type_data: *mut c_void) -> *mut obs_properties_t>
// audio_mix: Option<unsafe extern "C" fn(data: *mut c_void, ts_out: *mut u64, audio_output: *mut audio_output_data, channels: size_t, sample_rate: size_t) -> bool>
// icon_type: obs_icon_type
// media_play_pause: Option<unsafe extern "C" fn(data: *mut c_void, pause: bool)>
// media_restart: Option<unsafe extern "C" fn(data: *mut c_void)>
// media_stop: Option<unsafe extern "C" fn(data: *mut c_void)>
// media_next: Option<unsafe extern "C" fn(data: *mut c_void)>
// media_previous: Option<unsafe extern "C" fn(data: *mut c_void)>
// media_get_duration: Option<unsafe extern "C" fn(data: *mut c_void) -> i64>
// media_get_time: Option<unsafe extern "C" fn(data: *mut c_void) -> i64>
// media_set_time: Option<unsafe extern "C" fn(data: *mut c_void, miliseconds: i64)>
// media_get_state: Option<unsafe extern "C" fn(data: *mut c_void) -> obs_media_state>
