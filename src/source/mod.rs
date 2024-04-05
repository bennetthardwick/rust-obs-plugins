use paste::item;

pub mod context;
mod ffi;
pub mod traits;

use crate::media::state::MediaState;

pub use context::*;
pub use traits::*;

use obs_sys::{
    obs_filter_get_target, obs_icon_type, obs_icon_type_OBS_ICON_TYPE_AUDIO_INPUT,
    obs_icon_type_OBS_ICON_TYPE_AUDIO_OUTPUT, obs_icon_type_OBS_ICON_TYPE_BROWSER,
    obs_icon_type_OBS_ICON_TYPE_CAMERA, obs_icon_type_OBS_ICON_TYPE_COLOR,
    obs_icon_type_OBS_ICON_TYPE_CUSTOM, obs_icon_type_OBS_ICON_TYPE_DESKTOP_CAPTURE,
    obs_icon_type_OBS_ICON_TYPE_GAME_CAPTURE, obs_icon_type_OBS_ICON_TYPE_IMAGE,
    obs_icon_type_OBS_ICON_TYPE_MEDIA, obs_icon_type_OBS_ICON_TYPE_SLIDESHOW,
    obs_icon_type_OBS_ICON_TYPE_TEXT, obs_icon_type_OBS_ICON_TYPE_UNKNOWN,
    obs_icon_type_OBS_ICON_TYPE_WINDOW_CAPTURE, obs_mouse_button_type,
    obs_mouse_button_type_MOUSE_LEFT, obs_mouse_button_type_MOUSE_MIDDLE,
    obs_mouse_button_type_MOUSE_RIGHT, obs_source_active, obs_source_enabled,
    obs_source_get_base_height, obs_source_get_base_width, obs_source_get_height,
    obs_source_get_id, obs_source_get_name, obs_source_get_ref, obs_source_get_type,
    obs_source_get_width, obs_source_info, obs_source_media_ended, obs_source_media_get_duration,
    obs_source_media_get_state, obs_source_media_get_time, obs_source_media_next,
    obs_source_media_play_pause, obs_source_media_previous, obs_source_media_restart,
    obs_source_media_set_time, obs_source_media_started, obs_source_media_stop,
    obs_source_process_filter_begin, obs_source_process_filter_end,
    obs_source_process_filter_tech_end, obs_source_release, obs_source_set_enabled,
    obs_source_set_name, obs_source_showing, obs_source_skip_video_filter, obs_source_t,
    obs_source_type, obs_source_type_OBS_SOURCE_TYPE_FILTER, obs_source_type_OBS_SOURCE_TYPE_INPUT,
    obs_source_type_OBS_SOURCE_TYPE_SCENE, obs_source_type_OBS_SOURCE_TYPE_TRANSITION,
    obs_source_update, obs_text_type, OBS_SOURCE_AUDIO, OBS_SOURCE_CONTROLLABLE_MEDIA,
    OBS_SOURCE_INTERACTION, OBS_SOURCE_VIDEO,
};

use super::{
    graphics::{
        GraphicsAllowDirectRendering, GraphicsColorFormat, GraphicsEffect, GraphicsEffectContext,
    },
    string::ObsString,
};
use crate::{data::DataObj, native_enum, wrapper::PtrWrapper};

use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
};

native_enum!(MouseButton, obs_mouse_button_type {
    Left => MOUSE_LEFT,
    Middle => MOUSE_MIDDLE,
    Right => MOUSE_RIGHT
});

native_enum!(Icon, obs_icon_type {
    Unknown => OBS_ICON_TYPE_UNKNOWN,
    Image => OBS_ICON_TYPE_IMAGE,
    Color => OBS_ICON_TYPE_COLOR,
    Slideshow => OBS_ICON_TYPE_SLIDESHOW,
    AudioInput => OBS_ICON_TYPE_AUDIO_INPUT,
    AudioOutput => OBS_ICON_TYPE_AUDIO_OUTPUT,
    DesktopCapture => OBS_ICON_TYPE_DESKTOP_CAPTURE,
    WindowCapture => OBS_ICON_TYPE_WINDOW_CAPTURE,
    GameCapture => OBS_ICON_TYPE_GAME_CAPTURE,
    Camera => OBS_ICON_TYPE_CAMERA,
    Text => OBS_ICON_TYPE_TEXT,
    Media => OBS_ICON_TYPE_MEDIA,
    Browser => OBS_ICON_TYPE_BROWSER,
    Custom => OBS_ICON_TYPE_CUSTOM
});

/// OBS source type
///
/// See [OBS documentation](https://obsproject.com/docs/reference-sources.html#c.obs_source_get_type)
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceType {
    INPUT,
    SCENE,
    FILTER,
    TRANSITION,
}

impl SourceType {
    #[allow(non_upper_case_globals)]
    pub(crate) fn from_native(source_type: obs_source_type) -> Option<SourceType> {
        match source_type {
            obs_source_type_OBS_SOURCE_TYPE_INPUT => Some(SourceType::INPUT),
            obs_source_type_OBS_SOURCE_TYPE_SCENE => Some(SourceType::SCENE),
            obs_source_type_OBS_SOURCE_TYPE_FILTER => Some(SourceType::FILTER),
            obs_source_type_OBS_SOURCE_TYPE_TRANSITION => Some(SourceType::TRANSITION),
            _ => None,
        }
    }

    pub(crate) fn to_native(self) -> obs_source_type {
        match self {
            SourceType::INPUT => obs_source_type_OBS_SOURCE_TYPE_INPUT,
            SourceType::SCENE => obs_source_type_OBS_SOURCE_TYPE_SCENE,
            SourceType::FILTER => obs_source_type_OBS_SOURCE_TYPE_FILTER,
            SourceType::TRANSITION => obs_source_type_OBS_SOURCE_TYPE_TRANSITION,
        }
    }
}

#[deprecated = "use `SourceRef` instead"]
pub type SourceContext = SourceRef;

/// Context wrapping an OBS source - video / audio elements which are displayed
/// to the screen.
///
/// See [OBS documentation](https://obsproject.com/docs/reference-sources.html#c.obs_source_t)
pub struct SourceRef {
    inner: *mut obs_source_t,
}

impl SourceRef {
    /// # Safety
    ///
    /// Must call with a valid pointer.
    pub fn from_raw(source: *mut obs_source_t) -> Option<Self> {
        unsafe {
            Self::from_raw_unchecked(obs_source_get_ref(source))
        }
    }

    pub unsafe fn from_raw_unchecked(source: *mut obs_source_t) -> Option<Self> {
        if source.is_null() {
            None
        } else {
            Some(Self { inner: source })
        }
    }
}

impl Clone for SourceRef {
    fn clone(&self) -> Self {
        Self::from_raw(self.inner).expect("clone")
    }
}

impl Drop for SourceRef {
    fn drop(&mut self) {
        unsafe { obs_source_release(self.inner) }
    }
}

impl SourceRef {
    /// Run a function on the next source in the filter chain.
    ///
    /// Note: only works with sources that are filters.
    pub fn do_with_target<F: FnOnce(&mut SourceRef)>(&mut self, func: F) {
        unsafe {
            if let Some(SourceType::FILTER) =
                SourceType::from_native(obs_source_get_type(self.inner))
            {
                let target = obs_filter_get_target(self.inner);
                if let Some(mut context) = SourceRef::from_raw(target) {
                    func(&mut context);
                }
            }
        }
    }

    /// Return a unique id for the filter
    pub fn id(&self) -> usize {
        self.inner as usize
    }

    pub fn get_base_width(&self) -> u32 {
        unsafe { obs_source_get_base_width(self.inner) }
    }

    pub fn get_base_height(&self) -> u32 {
        unsafe { obs_source_get_base_height(self.inner) }
    }

    pub fn showing(&self) -> bool {
        unsafe { obs_source_showing(self.inner) }
    }

    pub fn active(&self) -> bool {
        unsafe { obs_source_active(self.inner) }
    }

    pub fn enabled(&self) -> bool {
        unsafe { obs_source_enabled(self.inner) }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        unsafe { obs_source_set_enabled(self.inner, enabled) }
    }

    pub fn source_id(&self) -> Option<&str> {
        unsafe {
            let ptr = obs_source_get_id(self.inner);
            if ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ptr).to_str().unwrap())
            }
        }
    }

    pub fn name(&self) -> Option<&str> {
        unsafe {
            let ptr = obs_source_get_name(self.inner);
            if ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ptr).to_str().unwrap())
            }
        }
    }

    pub fn set_name(&mut self, name: &str) {
        let cstr = CString::new(name).unwrap();
        unsafe {
            obs_source_set_name(self.inner, cstr.as_ptr());
        }
    }

    pub fn width(&self) -> u32 {
        unsafe { obs_source_get_width(self.inner) }
    }

    pub fn height(&self) -> u32 {
        unsafe { obs_source_get_height(self.inner) }
    }

    pub fn media_play_pause(&mut self, pause: bool) {
        unsafe {
            obs_source_media_play_pause(self.inner, pause);
        }
    }

    pub fn media_restart(&mut self) {
        unsafe {
            obs_source_media_restart(self.inner);
        }
    }

    pub fn media_stop(&mut self) {
        unsafe {
            obs_source_media_stop(self.inner);
        }
    }

    pub fn media_next(&mut self) {
        unsafe {
            obs_source_media_next(self.inner);
        }
    }

    pub fn media_previous(&mut self) {
        unsafe {
            obs_source_media_previous(self.inner);
        }
    }

    pub fn media_duration(&self) -> i64 {
        unsafe { obs_source_media_get_duration(self.inner) }
    }

    pub fn media_time(&self) -> i64 {
        unsafe { obs_source_media_get_time(self.inner) }
    }

    pub fn media_set_time(&mut self, ms: i64) {
        unsafe { obs_source_media_set_time(self.inner, ms) }
    }

    pub fn media_state(&self) -> MediaState {
        let ret = unsafe { obs_source_media_get_state(self.inner) };
        MediaState::from_native(ret).expect("Invalid media state value")
    }

    pub fn media_started(&mut self) {
        unsafe {
            obs_source_media_started(self.inner);
        }
    }

    pub fn media_ended(&mut self) {
        unsafe {
            obs_source_media_ended(self.inner);
        }
    }

    /// Skips the video filter if it's invalid
    pub fn skip_video_filter(&mut self) {
        unsafe {
            obs_source_skip_video_filter(self.inner);
        }
    }

    /// Run a function to do drawing - if the source is a filter.
    /// This function is wrapped by calls that automatically handle effect-based
    /// filter processing.
    ///
    /// See [OBS documentation](https://obsproject.com/docs/reference-sources.html#c.obs_source_process_filter_begin)
    ///
    /// Note: only works with sources that are filters.
    pub fn process_filter<F: FnOnce(&mut GraphicsEffectContext, &mut GraphicsEffect)>(
        &mut self,
        _render: &mut VideoRenderContext,
        effect: &mut GraphicsEffect,
        (cx, cy): (u32, u32),
        format: GraphicsColorFormat,
        direct: GraphicsAllowDirectRendering,
        func: F,
    ) {
        unsafe {
            if let Some(SourceType::FILTER) =
                SourceType::from_native(obs_source_get_type(self.inner))
            {
                if obs_source_process_filter_begin(self.inner, format.as_raw(), direct.as_raw()) {
                    let mut context = GraphicsEffectContext::new();
                    func(&mut context, effect);
                    obs_source_process_filter_end(self.inner, effect.as_ptr(), cx, cy);
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn process_filter_tech<F: FnOnce(&mut GraphicsEffectContext, &mut GraphicsEffect)>(
        &mut self,
        _render: &mut VideoRenderContext,
        effect: &mut GraphicsEffect,
        (cx, cy): (u32, u32),
        format: GraphicsColorFormat,
        direct: GraphicsAllowDirectRendering,
        technique: ObsString,
        func: F,
    ) {
        unsafe {
            if let Some(SourceType::FILTER) =
                SourceType::from_native(obs_source_get_type(self.inner))
            {
                if obs_source_process_filter_begin(self.inner, format.as_raw(), direct.as_raw()) {
                    let mut context = GraphicsEffectContext::new();
                    func(&mut context, effect);
                    obs_source_process_filter_tech_end(
                        self.inner,
                        effect.as_ptr(),
                        cx,
                        cy,
                        technique.as_ptr(),
                    );
                }
            }
        }
    }

    /// Update the source settings based on a settings context.
    pub fn update_source_settings(&mut self, settings: &mut DataObj) {
        unsafe {
            obs_source_update(self.inner, settings.as_ptr_mut());
        }
    }
}

pub struct EnumActiveContext {}

pub struct EnumAllContext {}

pub struct SourceInfo {
    info: Box<obs_source_info>,
}

impl SourceInfo {
    pub fn into_raw(self) -> *mut obs_source_info {
        Box::into_raw(self.info)
    }

    pub fn set_icon(&mut self, icon: Icon) {
        self.info.icon_type = icon.into();
    }
}

impl AsRef<obs_source_info> for SourceInfo {
    fn as_ref(&self) -> &obs_source_info {
        self.info.as_ref()
    }
}

impl AsMut<obs_source_info> for SourceInfo {
    fn as_mut(&mut self) -> &mut obs_source_info {
        self.info.as_mut()
    }
}

/// The SourceInfoBuilder that handles creating the [SourceInfo](https://obsproject.com/docs/reference-sources.html#c.obs_source_info) object.
///
/// For each trait that is implemented for the Source, it needs to be enabled
/// using this builder. If an struct called `FocusFilter` implements
/// `CreateSource` and `GetNameSource` it would need to enable those features.
///
/// ```rs
/// let source = load_context
///  .create_source_builder::<FocusFilter, ()>()
///  .enable_get_name()
///  .enable_create()
///  .build();
/// ```
pub struct SourceInfoBuilder<D: Sourceable> {
    __data: PhantomData<D>,
    info: obs_source_info,
}

impl<D: Sourceable> SourceInfoBuilder<D> {
    pub(crate) fn new() -> Self {
        Self {
            __data: PhantomData,
            info: obs_source_info {
                id: D::get_id().as_ptr(),
                type_: D::get_type().to_native(),
                create: Some(ffi::create::<D>),
                destroy: Some(ffi::destroy::<D>),
                type_data: std::ptr::null_mut(),
                ..Default::default()
            },
        }
    }

    pub fn build(mut self) -> SourceInfo {
        if self.info.video_render.is_some() {
            self.info.output_flags |= OBS_SOURCE_VIDEO;
        }

        if self.info.audio_render.is_some() || self.info.filter_audio.is_some() {
            self.info.output_flags |= OBS_SOURCE_AUDIO;
        }

        if self.info.media_get_state.is_some() || self.info.media_play_pause.is_some() {
            self.info.output_flags |= OBS_SOURCE_CONTROLLABLE_MEDIA;
        }

        if self.info.mouse_click.is_some()
            || self.info.mouse_move.is_some()
            || self.info.mouse_wheel.is_some()
            || self.info.focus.is_some()
            || self.info.key_click.is_some()
        {
            self.info.output_flags |= OBS_SOURCE_INTERACTION;
        }

        SourceInfo {
            info: Box::new(self.info),
        }
    }

    pub fn with_icon(mut self, icon: Icon) -> Self {
        self.info.icon_type = icon.into();
        self
    }
}

macro_rules! impl_source_builder {
    ($($f:ident => $t:ident)*) => ($(
        item! {
            impl<D: Sourceable + [<$t>]> SourceInfoBuilder<D> {
                pub fn [<enable_$f>](mut self) -> Self {
                    self.info.[<$f>] = Some(ffi::[<$f>]::<D>);
                    self
                }
            }
        }
    )*)
}

impl_source_builder! {
    get_name => GetNameSource
    get_width => GetWidthSource
    get_height => GetHeightSource
    activate => ActivateSource
    deactivate => DeactivateSource
    update => UpdateSource
    video_render => VideoRenderSource
    audio_render => AudioRenderSource
    get_properties => GetPropertiesSource
    enum_active_sources => EnumActiveSource
    enum_all_sources => EnumAllSource
    transition_start => TransitionStartSource
    transition_stop => TransitionStopSource
    video_tick => VideoTickSource
    filter_audio => FilterAudioSource
    filter_video => FilterVideoSource
    get_defaults => GetDefaultsSource
    media_play_pause => MediaPlayPauseSource
    media_restart => MediaRestartSource
    media_stop => MediaStopSource
    media_next => MediaNextSource
    media_previous => MediaPreviousSource
    media_get_duration => MediaGetDurationSource
    media_get_time => MediaGetTimeSource
    media_set_time => MediaSetTimeSource
    media_get_state => MediaGetStateSource
    mouse_wheel => MouseWheelSource
    mouse_click => MouseClickSource
    mouse_move => MouseMoveSource
    key_click => KeyClickSource
    focus => FocusSource
}
