use obs_sys::{video_data, audio_data, encoder_packet};

use crate::{string::ObsString, prelude::DataObj, properties::Properties};

use super::ffi::CreatableOutputContext;

pub trait Outputable {
    fn get_id() -> ObsString;
}

pub trait CreatableOutput: Sized {
    fn create(context: CreatableOutputContext<'_>) -> Self;
}

pub trait GetNameOutput {
    fn get_name() -> ObsString;
}

macro_rules! simple_trait {
    ($($f:ident$(($($params:tt)*))? => $t:ident $(-> $ret:ty)?)*) => ($(
        pub trait $t: Sized {
            fn $f(&mut self $(, $($params)*)?) $(-> $ret)?;
        }
    )*)
}

simple_trait!{
    start => StartOutput -> bool
    stop(ts: u64) => StopOutput
}

pub trait RawVideoOutput: Sized {
    fn raw_video(&mut self, frame: &mut video_data);
}

pub trait RawAudioOutput: Sized {
    fn raw_audio(&mut self, frame: &mut audio_data);
}

pub trait RawAudio2Output: Sized {
    fn raw_audio2(&mut self, idx: usize, frame: &mut audio_data);
}

pub trait EncodedPacketOutput: Sized {
    fn encoded_packet(&mut self, packet: &mut encoder_packet);
}

pub trait UpdateOutput: Sized {
    fn update(&mut self, settings: &mut DataObj);
}

pub trait GetDefaultsOutput {
    fn get_defaults(settings: &mut DataObj);
}

pub trait GetPropertiesOutput: Sized {
    fn get_properties(&mut self) -> Properties;
}

simple_trait! {
    get_total_bytes => GetTotalBytesOutput -> u64
    get_dropped_frames => GetDroppedFramesOutput-> i32
    get_congestion => GetCongestionOutput -> f32
    get_connect_time_ms => GetConnectTimeMsOutput -> i32
}
