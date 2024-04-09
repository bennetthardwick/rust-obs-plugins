use std::ffi::CStr;

use obs_sys::{
    obs_encoder_t, obs_enum_output_types, obs_enum_outputs, obs_output_active, obs_output_audio,
    obs_output_begin_data_capture, obs_output_can_begin_data_capture, obs_output_can_pause,
    obs_output_create, obs_output_end_data_capture, obs_output_force_stop,
    obs_output_get_audio_encoder, obs_output_get_delay, obs_output_get_frames_dropped,
    obs_output_get_id, obs_output_get_name, obs_output_get_ref, obs_output_get_total_bytes,
    obs_output_get_total_frames, obs_output_get_video_encoder, obs_output_initialize_encoders,
    obs_output_pause, obs_output_paused, obs_output_release, obs_output_set_audio_encoder,
    obs_output_set_delay, obs_output_set_media, obs_output_set_video_encoder, obs_output_start,
    obs_output_stop, obs_output_t, obs_output_video,
};

use crate::hotkey::HotkeyCallbacks;
use crate::media::{audio::AudioRef, video::VideoRef};
use crate::string::TryIntoObsString;
use crate::{hotkey::Hotkey, prelude::DataObj, string::ObsString, wrapper::PtrWrapper};
use crate::{Error, Result};

#[deprecated = "use `OutputRef` instead"]
pub type OutputContext = OutputRef;

/// Context wrapping an OBS output - video / audio elements which are displayed
/// to the screen.
///
/// See [OBS documentation](https://obsproject.com/docs/reference-outputs.html#c.obs_output_t)
pub struct OutputRef {
    pub(crate) inner: *mut obs_output_t,
}

impl_ptr_wrapper!(
    @ptr: inner,
    OutputRef,
    obs_output_t,
    obs_output_get_ref,
    obs_output_release
);

extern "C" fn enum_proc(params: *mut std::ffi::c_void, output: *mut obs_output_t) -> bool {
    let mut v = unsafe { Box::<Vec<*mut obs_output_t>>::from_raw(params as *mut _) };
    v.push(output);
    Box::into_raw(v);
    true
}

impl OutputRef {
    pub fn new(id: ObsString, name: ObsString, settings: Option<DataObj<'_>>) -> Result<Self> {
        let settings = match settings {
            Some(data) => unsafe { data.as_ptr_mut() },
            None => std::ptr::null_mut(),
        };
        let output = unsafe {
            obs_output_create(id.as_ptr(), name.as_ptr(), settings, std::ptr::null_mut())
        };

        unsafe { Self::from_raw_unchecked(output) }.ok_or(Error::NulPointer("obs_output_cretae"))
    }
    pub fn all_outputs() -> Vec<Self> {
        let outputs = Vec::<*mut obs_output_t>::new();
        let params = Box::into_raw(Box::new(outputs));
        unsafe {
            // `obs_enum_outputs` would return `weak_ref`, so `get_ref` needed
            obs_enum_outputs(Some(enum_proc), params as *mut _);
        }
        let outputs = unsafe { Box::from_raw(params) };
        outputs
            .into_iter()
            .filter_map(OutputRef::from_raw)
            .collect()
    }
    pub fn all_types() -> Vec<String> {
        let mut types = Vec::new();
        let mut id = std::ptr::null();
        for idx in 0.. {
            unsafe {
                if !obs_enum_output_types(idx, &mut id) {
                    break;
                }
            }
            if id.is_null() {
                types.push(String::new())
            } else {
                types.push(unsafe { CStr::from_ptr(id) }.to_str().unwrap().to_string())
            }
        }
        types
    }

    pub fn output_id(&self) -> Result<ObsString> {
        unsafe { obs_output_get_id(self.inner) }.try_into_obs_string()
    }

    pub fn name(&self) -> Result<ObsString> {
        unsafe { obs_output_get_name(self.inner) }.try_into_obs_string()
    }

    pub fn start(&mut self) -> bool {
        unsafe { obs_output_start(self.inner) }
    }
    pub fn stop(&mut self) {
        unsafe { obs_output_stop(self.inner) }
    }
    pub fn force_stop(&mut self) {
        unsafe { obs_output_force_stop(self.inner) }
    }
    pub fn is_active(&self) -> bool {
        unsafe { obs_output_active(self.inner) }
    }
    pub fn set_delay(&mut self, delay_secs: u32, flags: u32) {
        unsafe { obs_output_set_delay(self.inner, delay_secs, flags) }
    }
    pub fn delay(&self) -> u32 {
        unsafe { obs_output_get_delay(self.inner) }
    }
    pub fn can_pause(&self) -> bool {
        unsafe { obs_output_can_pause(self.inner) }
    }
    pub fn pause(&mut self, pause: bool) -> bool {
        unsafe { obs_output_pause(self.inner, pause) }
    }
    pub fn is_paused(&self) -> bool {
        unsafe { obs_output_paused(self.inner) }
    }
    /// # Safety
    /// make sure encoder is valid
    pub unsafe fn set_video_encoder(&mut self, encoder: *mut obs_encoder_t) {
        // TODO: later should change *mut obs_encoder_t to something like EncoderContext
        unsafe { obs_output_set_video_encoder(self.inner, encoder) }
    }
    pub fn video_encoder(&self) -> *mut obs_encoder_t {
        unsafe { obs_output_get_video_encoder(self.inner) }
    }
    /// # Safety
    /// make sure encoder is valid
    pub unsafe fn set_audio_encoder(&mut self, encoder: *mut obs_encoder_t, idx: usize) {
        // TODO: later should change *mut obs_encoder_t to something like EncoderContext
        unsafe { obs_output_set_audio_encoder(self.inner, encoder, idx as _) }
    }
    pub fn audio_encoder(&self, idx: usize) -> *mut obs_encoder_t {
        unsafe { obs_output_get_audio_encoder(self.inner, idx as _) }
    }
    pub fn init_encoders(&mut self, flags: u32) -> bool {
        unsafe { obs_output_initialize_encoders(self.inner, flags) }
    }
    pub fn can_start_capture(&self, flags: u32) -> bool {
        unsafe { obs_output_can_begin_data_capture(self.inner, flags) }
    }
    pub fn start_capture(&mut self, flags: u32) -> bool {
        unsafe { obs_output_begin_data_capture(self.inner, flags) }
    }
    pub fn stop_capture(&mut self) {
        unsafe { obs_output_end_data_capture(self.inner) }
    }

    pub fn video(&self) -> VideoRef {
        let video = unsafe { obs_output_video(self.inner) };
        VideoRef::from_raw(video)
    }
    pub fn audio(&self) -> AudioRef {
        let audio = unsafe { obs_output_audio(self.inner) };
        AudioRef::from_raw(audio)
    }
    pub fn set_media(&mut self, video: VideoRef, audio: AudioRef) {
        self.set_video_and_audio(video, audio)
    }
    pub fn set_video_and_audio(&mut self, video: VideoRef, audio: AudioRef) {
        unsafe { obs_output_set_media(self.inner, video.pointer, audio.pointer) }
    }

    pub fn total_bytes(&self) -> u64 {
        unsafe { obs_output_get_total_bytes(self.inner) }
    }
    pub fn frames_dropped(&self) -> u32 {
        unsafe { obs_output_get_frames_dropped(self.inner) as u32 }
    }
    pub fn total_frames(&self) -> u32 {
        unsafe { obs_output_get_total_frames(self.inner) as u32 }
    }
}

pub struct CreatableOutputContext<'a, D> {
    pub(crate) hotkey_callbacks: HotkeyCallbacks<D>,
    pub settings: DataObj<'a>,
}

impl<'a, D> CreatableOutputContext<'a, D> {
    pub fn from_raw(settings: DataObj<'a>) -> Self {
        Self {
            hotkey_callbacks: vec![],
            settings,
        }
    }

    pub fn register_hotkey<F: FnMut(&mut Hotkey, &mut D) + 'static>(
        &mut self,
        name: ObsString,
        description: ObsString,
        func: F,
    ) {
        self.hotkey_callbacks
            .push((name, description, Box::new(func)));
    }
}
