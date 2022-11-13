use obs_sys::{audio_output_get_channels, audio_output_get_sample_rate, audio_t, obs_audio_data};

pub struct AudioDataContext {
    pointer: *mut obs_audio_data,
}

impl AudioDataContext {
    pub fn from_raw(pointer: *mut obs_audio_data) -> Self {
        Self { pointer }
    }

    pub fn frames(&self) -> usize {
        unsafe {
            self.pointer
                .as_ref()
                .expect("Audio pointer was null!")
                .frames as usize
        }
    }

    pub fn channels(&self) -> usize {
        unsafe {
            self.pointer
                .as_ref()
                .expect("Audio pointer was null!")
                .data
                .len()
        }
    }

    pub fn get_channel_as_mut_slice(&self, channel: usize) -> Option<&'_ mut [f32]> {
        unsafe {
            let data = self.pointer.as_ref()?.data;

            if channel >= data.len() {
                return None;
            }

            let frames = self.pointer.as_ref()?.frames;

            Some(core::slice::from_raw_parts_mut(
                data[channel] as *mut f32,
                frames as usize,
            ))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioInfo {
    pub sample_rate: usize,
    pub channels: usize,
}

pub struct AudioRef {
    pub pointer: *mut audio_t,
}

impl AudioRef {
    pub fn from_raw(pointer: *mut audio_t) -> Self {
        Self { pointer }
    }

    pub fn info(&self) -> AudioInfo {
        AudioInfo {
            sample_rate: self.sample_rate(),
            channels: self.channels(),
        }
    }

    pub fn sample_rate(&self) -> usize {
        unsafe { audio_output_get_sample_rate(self.pointer) as usize }
    }

    pub fn channels(&self) -> usize {
        unsafe { audio_output_get_channels(self.pointer) as usize }
    }
}
