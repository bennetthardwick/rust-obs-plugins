use obs_sys::{audio_data, encoder_packet, video_data};

use crate::{prelude::DataObj, properties::Properties, string::ObsString};

use super::{CreatableOutputContext, OutputContext};

pub trait Outputable: Sized {
    fn get_id() -> ObsString;
    fn create(context: &mut CreatableOutputContext<'_, Self>, output: OutputContext) -> Self;

    fn start(&mut self) -> bool {
        true
    }
    fn stop(&mut self, _ts: u64) {}
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
