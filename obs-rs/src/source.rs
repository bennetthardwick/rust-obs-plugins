use obs_sys::{
    obs_source_info, obs_source_type, obs_source_type_OBS_SOURCE_TYPE_FILTER,
    obs_source_type_OBS_SOURCE_TYPE_INPUT, obs_source_type_OBS_SOURCE_TYPE_SCENE,
    obs_source_type_OBS_SOURCE_TYPE_TRANSITION,
};

use std::marker::PhantomData;
use std::os::raw::c_char;

#[derive(Clone, Copy)]
pub enum SourceType {
    INPUT,
    SCENE,
    FILTER,
    TRANSITION,
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

pub struct SourceContext {}

pub struct SettingsContext {}

pub struct AudioRenderContext {}

pub struct VideoRenderContext {}

pub struct PropertiesContext {}

pub struct EnumActiveContext {}
pub struct EnumAllContext {}

pub struct SourceInfo {
    info: obs_source_info,
}

pub mod traits {
    use super::SourceType;

    pub trait Sourceable {
        fn get_id() -> &'static str;
        fn get_type() -> SourceType;
    }

    pub trait GetNameSource {
        fn get_name() -> &'static str;
    }

    pub trait GetWidthSource<D> {
        fn get_width(data: &D) -> u32;
    }

    pub trait GetHeightSource<D> {
        fn get_height(data: &D) -> u32;
    }
}

use traits::*;

pub struct SourceInfoBuilder<T: Sourceable, D> {
    __source: PhantomData<T>,
    __data: PhantomData<D>,
    info: obs_source_info, // id: &'static str,
                           // source_type: SourceType,
                           // output_flags: u32,
                           // get_name: Option<Box<dyn Fn() -> &'static str>>,
                           // create: Box<dyn Fn(&SettingsContext, SourceContext) -> S>,
                           // get_width: Option<Box<dyn Fn(&S) -> u32>>,
                           // get_height: Option<Box<dyn Fn(&S) -> u32>>,
                           // update: Option<Box<dyn Fn(&mut S, &SettingsContext)>>,
                           // video_render: Option<Box<dyn Fn(&mut S, &mut VideoRenderContext)>>,
                           // audio_render: Option<Box<dyn Fn(&mut S, &mut AudioRenderContext)>>,
                           // get_properties: Option<Box<dyn Fn(&mut S, &mut PropertiesContext)>>,
                           // enum_active_sources: Option<Box<dyn Fn(&mut S, &mut EnumActiveContext)>>,
                           // enum_all_sources: Option<Box<dyn Fn(&mut S, &mut EnumAllContext)>>,
                           // transition_start: Option<Box<dyn Fn(&mut S)>>,
                           // transition_stop: Option<Box<dyn Fn(&mut S)>>
}

pub unsafe extern "C" fn get_name<F: GetNameSource>(
    type_data: *mut ::std::os::raw::c_void,
) -> *const c_char {
    let name = F::get_name();
    name.as_bytes().as_ptr() as *const c_char
}

pub unsafe extern "C" fn get_width<D, F: GetWidthSource<D>>(data: *mut std::ffi::c_void) -> u32 {
    let d: &mut D = &mut *(data as *mut D);
    F::get_width(&d)
}

pub unsafe extern "C" fn get_height<D, F: GetHeightSource<D>>(data: *mut std::ffi::c_void) -> u32 {
    let d: &mut D = &mut *(data as *mut D);
    F::get_height(&d)
}

impl<T: Sourceable, D: Default> SourceInfoBuilder<T, D> {
    pub(crate) fn new(id: &'static str, source_type: SourceType) -> Self {
        Self {
            __source: PhantomData,
            __data: PhantomData,
            info: obs_source_info {
                id: T::get_id().as_bytes().as_ptr() as *const c_char,
                type_: source_type.to_native(),
                output_flags: 0,
                get_name: None,
                create: None,
                destroy: None,
                get_width: None,
                get_height: None,
                get_defaults: None,
                get_properties: None,
                update: None,
                activate: None,
                deactivate: None,
                show: None,
                hide: None,
                video_tick: None,
                video_render: None,
                filter_video: None,
                filter_audio: None,
                enum_active_sources: None,
                save: None,
                load: None,
                mouse_click: None,
                mouse_move: None,
                mouse_wheel: None,
                focus: None,
                key_click: None,
                filter_remove: None,
                type_data: std::ptr::null_mut(),
                free_type_data: None,
                audio_render: None,
                enum_all_sources: None,
                transition_start: None,
                transition_stop: None,
                get_defaults2: None,
                get_properties2: None,
                audio_mix: None,
                icon_type: 0,
                media_play_pause: None,
                media_restart: None,
                media_stop: None,
                media_next: None,
                media_previous: None,
                media_get_duration: None,
                media_get_time: None,
                media_set_time: None,
                media_get_state: None,
            },
        }
    }

    pub fn build(self) -> SourceInfo {
        SourceInfo { info: self.info }
    }
}

impl<T: Sourceable + GetNameSource, D: Default> SourceInfoBuilder<T, D> {
    pub fn with_get_name(mut self) -> Self {
        self.info.get_name = Some(get_name::<T>);
        self
    }
}

impl<D: Default, T: Sourceable + GetWidthSource<D>> SourceInfoBuilder<T, D> {
    pub fn with_get_width(mut self) -> Self {
        self.info.get_width = Some(get_width::<D, T>);
        self
    }
}

impl<D: Default, T: Sourceable + GetHeightSource<D>> SourceInfoBuilder<T, D> {
    pub fn with_get_height(mut self) -> Self {
        self.info.get_width = Some(get_height::<D, T>);
        self
    }
}
