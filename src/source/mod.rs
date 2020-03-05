#![allow(non_upper_case_globals)]

use paste::item;

mod ffi;

pub mod context;
pub mod properties;
pub mod traits;

pub use context::*;
pub use properties::*;
pub use traits::*;

use obs_sys::{
    obs_filter_get_target, obs_source_get_base_height, obs_source_get_base_width,
    obs_source_get_type, obs_source_info, obs_source_process_filter_begin,
    obs_source_process_filter_end, obs_source_skip_video_filter, obs_source_t, obs_source_type,
    obs_source_type_OBS_SOURCE_TYPE_FILTER, obs_source_type_OBS_SOURCE_TYPE_INPUT,
    obs_source_type_OBS_SOURCE_TYPE_SCENE, obs_source_type_OBS_SOURCE_TYPE_TRANSITION,
    obs_source_update,
};

use super::{
    graphics::{
        GraphicsAllowDirectRendering, GraphicsColorFormat, GraphicsEffect, GraphicsEffectContext,
    },
    string::ObsString,
};

use std::marker::PhantomData;

/// OBS source type
///
/// See [OBS documentation](https://obsproject.com/docs/reference-sources.html#c.obs_source_get_type)
#[derive(Clone, Copy)]
pub enum SourceType {
    INPUT,
    SCENE,
    FILTER,
    TRANSITION,
}

impl SourceType {
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

/// Context wrapping an OBS source - video / audio elements which are displayed to the screen.
///
/// See [OBS documentation](https://obsproject.com/docs/reference-sources.html#c.obs_source_t)
pub struct SourceContext {
    source: *mut obs_source_t,
}

impl SourceContext {
    /// Run a function on the next source in the filter chain.
    ///
    /// Note: only works with sources that are filters.
    pub fn do_with_target<F: FnOnce(&mut SourceContext)>(&mut self, func: F) {
        unsafe {
            if let Some(SourceType::FILTER) =
                SourceType::from_native(obs_source_get_type(self.source))
            {
                let target = obs_filter_get_target(self.source);
                let mut context = SourceContext { source: target };
                func(&mut context);
            }
        }
    }

    /// Return a unique id for the filter
    pub fn id(&self) -> usize {
        self.source as usize
    }

    pub fn get_base_width(&self) -> u32 {
        unsafe { obs_source_get_base_width(self.source) }
    }

    pub fn get_base_height(&self) -> u32 {
        unsafe { obs_source_get_base_height(self.source) }
    }

    /// Skips the video filter if it's invalid
    pub fn skip_video_filter(&mut self) {
        unsafe {
            obs_source_skip_video_filter(self.source);
        }
    }

    /// Run a function to do drawing - if the source is a filter.
    /// This function is wrapped by calls that automatically handle effect-based filter processing.
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
                SourceType::from_native(obs_source_get_type(self.source))
            {
                if obs_source_process_filter_begin(self.source, format.as_raw(), direct.as_raw()) {
                    let mut context = GraphicsEffectContext::new();
                    func(&mut context, effect);
                    obs_source_process_filter_end(self.source, effect.as_ptr(), cx, cy);
                }
            }
        }
    }

    /// Update the source settings based on a settings context.
    pub fn update_source_settings(&mut self, settings: &SettingsContext) {
        unsafe {
            obs_source_update(self.source, settings.as_raw());
        }
    }
}

pub struct EnumActiveContext {}

pub struct EnumAllContext {}

pub struct SourceInfo {
    info: Box<obs_source_info>,
}

impl SourceInfo {
    /// # Safety
    /// Creates a raw pointer from a box and could cause UB is misused.
    pub unsafe fn into_raw(self) -> *mut obs_source_info {
        Box::into_raw(self.info)
    }
}

/// The SourceInfoBuilder that handles creating the [SourceInfo](https://obsproject.com/docs/reference-sources.html#c.obs_source_info) object.
///
/// For each trait that is implemented for the Source, it needs to be enabled using this builder.
/// If an struct called `FocusFilter` implements `CreateSource` and `GetNameSource` it would need
/// to enable those features.
///
/// ```rs
/// let source = load_context
///  .create_source_builder::<FocusFilter, ()>()
///  .enable_get_name()
///  .enable_create()
///  .build();
/// ```
///
pub struct SourceInfoBuilder<T: Sourceable, D> {
    __source: PhantomData<T>,
    __data: PhantomData<D>,
    info: obs_source_info,
}

impl<T: Sourceable, D> SourceInfoBuilder<T, D> {
    pub(crate) fn new() -> Self {
        Self {
            __source: PhantomData,
            __data: PhantomData,
            info: obs_source_info {
                id: T::get_id().as_ptr(),
                type_: T::get_type().to_native(),
                output_flags: 0,
                get_name: None,
                create: Some(ffi::create_default_data::<D>),
                destroy: Some(ffi::destroy::<D>),
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
            },
        }
    }

    pub fn with_output_flags(mut self, flags: u32) -> Self {
        self.info.output_flags = flags;
        self
    }

    pub fn build(self) -> SourceInfo {
        SourceInfo {
            info: Box::new(self.info),
        }
    }
}

macro_rules! impl_source_builder {
    ($($f:ident => $t:ident)*) => ($(
        item! {
            impl<D, T: Sourceable + [<$t>]<D>> SourceInfoBuilder<T, D> {
                pub fn [<enable_$f>](mut self) -> Self {
                    self.info.[<$f>] = Some(ffi::[<$f>]::<D, T>);
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
    create => CreatableSource
    update => UpdateSource
    video_render => VideoRenderSource
    audio_render => AudioRenderSource
    get_properties => GetPropertiesSource
    enum_active_sources => EnumActiveSource
    enum_all_sources => EnumAllSource
    transition_start => TransitionStartSource
    transition_stop => TransitionStopSource
    video_tick => VideoTickSource
}

