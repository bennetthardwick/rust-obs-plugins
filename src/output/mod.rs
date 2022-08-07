use paste::item;

use std::marker::PhantomData;

use obs_sys::{obs_output_info, OBS_OUTPUT_VIDEO, OBS_OUTPUT_AUDIO, OBS_OUTPUT_MULTI_TRACK, OBS_OUTPUT_ENCODED};

pub mod context;
mod ffi;
pub mod traits;

pub use context::*;
pub use traits::*;

pub struct OutputInfo {
    info: Box<obs_output_info>,
}

impl OutputInfo {
    /// # Safety
    /// Creates a raw pointer from a box and could cause UB is misused.
    pub unsafe fn into_raw(self) -> *mut obs_output_info {
        Box::into_raw(self.info)
    }
}

impl AsRef<obs_output_info> for OutputInfo {
    fn as_ref(&self) -> &obs_output_info {
        self.info.as_ref()
    }
}

/// The OutputInfoBuilder that handles creating the [OutputInfo](https://obsproject.com/docs/reference-outputs.html#c.obs_output_info) object.
///
/// For each trait that is implemented for the Output, it needs to be enabled
/// using this builder. If an struct called `FocusFilter` implements
/// `CreateOutput` and `GetNameOutput` it would need to enable those features.
///
/// ```rs
/// let output = load_context
///  .create_output_builder::<FocusFilter>()
///  .enable_get_name()
///  .enable_create()
///  .build();
/// ```
pub struct OutputInfoBuilder<D: Outputable> {
    __data: PhantomData<D>,
    info: obs_output_info,
}

impl<D: Outputable> OutputInfoBuilder<D> {
    pub(crate) fn new() -> Self {
        Self {
            __data: PhantomData,
            info: obs_output_info {
                id: D::get_id().as_ptr(),
                create: Some(ffi::create::<D>),
                destroy: Some(ffi::destroy::<D>),
                start: Some(ffi::start::<D>),
                stop: Some(ffi::stop::<D>),
                type_data: std::ptr::null_mut(),
                ..Default::default()
            },
        }
    }

    pub fn build(mut self) -> OutputInfo {
        // see libobs/obs-module.c:obs_register_output_s
        if self.info.encoded_packet.is_some() {
            self.info.flags |= OBS_OUTPUT_ENCODED;
        }

        if self.info.raw_video.is_some() {
            self.info.flags |= OBS_OUTPUT_VIDEO;
        }
        if self.info.raw_audio.is_some() || self.info.raw_audio2.is_some() {
            self.info.flags |= OBS_OUTPUT_AUDIO;
        }
        if self.info.raw_audio2.is_some() {
            self.info.flags |= OBS_OUTPUT_MULTI_TRACK;
        }

        OutputInfo {
            info: Box::new(self.info),
        }
    }
}

macro_rules! impl_output_builder {
    ($($f:ident => $t:ident)*) => ($(
        item! {
            impl<D: Outputable + [<$t>]> OutputInfoBuilder<D> {
                pub fn [<enable_$f>](mut self) -> Self {
                    self.info.[<$f>] = Some(ffi::[<$f>]::<D>);
                    self
                }
            }
        }
    )*)
}

impl_output_builder! {
    get_name => GetNameOutput
    // this two is required
    // start => StartOutput
    // stop => StopOutput
    raw_video => RawVideoOutput
    raw_audio => RawAudioOutput
    raw_audio2 => RawAudio2Output
    encoded_packet => EncodedPacketOutput
    update => UpdateOutput
    get_defaults => GetDefaultsOutput
    // TODO: version?
    // get_defaults2 => GetDefaults2Output
    get_properties => GetPropertiesOutput
    // get_properties2
    // unused1
    get_total_bytes => GetTotalBytesOutput
    get_dropped_frames => GetDroppedFramesOutput
    // type_data
    // free_type_data
    get_congestion => GetCongestionOutput
    get_connect_time_ms => GetConnectTimeMsOutput
}
