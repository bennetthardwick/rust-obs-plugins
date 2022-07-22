use obs_sys::{video_data, audio_data, encoder_packet};

use crate::{string::ObsString, prelude::DataObj, properties::Properties};

pub trait Outputable {
    fn get_id() -> ObsString;
}

pub trait GetNameOutput {
    fn get_name() -> ObsString;
}

macro_rules! simple_trait {
    ($($f:ident$(($($params:tt)*))? => $t:ident $(-> $ret:ty)?)*) => ($(
        pub trait $t: Sized {
            fn $f(data: &mut Option<Self> $(, $($params)*)?) $(-> $ret)?;
        }
    )*)
}

simple_trait!{
    start => StartOutput -> bool
    stop(ts: u64) => StopOutput
}

pub trait RawVideoOutput: Sized {
    fn raw_video(data: &mut Option<Self>, frame: &mut video_data);
}

pub trait RawAudioOutput: Sized {
    fn raw_audio(data: &mut Option<Self>, frame: &mut audio_data);
}

pub trait RawAudio2Output: Sized {
    fn raw_audio2(data: &mut Option<Self>, idx: usize, frame: &mut audio_data);
}

pub trait EncodedPacketOutput: Sized {
    fn encoded_packet(data: &mut Option<Self>, packet: &mut encoder_packet);
}

pub trait UpdateOutput: Sized {
    fn update(data: &mut Option<Self>, settings: &mut DataObj);
}

pub trait GetDefaultsOutput {
    fn get_defaults(settings: &mut DataObj);
}

pub trait GetPropertiesOutput: Sized {
    fn get_properties(data: &mut Option<Self>) -> Properties;
}

simple_trait! {
    get_total_bytes => GetTotalBytesOutput -> u64
    get_dropped_frames => GetDroppedFramesOutput-> i32
    get_congestion => GetCongestionOutput -> f32
    get_connect_time_ms => GetConnectTimeMsOutput -> i32
}
