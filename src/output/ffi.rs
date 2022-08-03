use super::traits::*;
use crate::{data::DataObj, wrapper::PtrWrapper};
use obs_sys::{audio_data, encoder_packet, obs_properties, video_data};
use paste::item;
use std::ffi::c_void;
use std::mem::forget;
use std::os::raw::{c_char, c_int};

use obs_sys::{
    obs_data_t, obs_output_t
};

#[derive(Default)]
struct DataWrapper<D> {
    data: D,
    // hotkey_callbacks: HashMap<obs_hotkey_id, Box<dyn FnMut(&mut Hotkey, &mut Option<D>)>>,
}

impl<D> From<D> for DataWrapper<D> {
    fn from(data: D) -> Self {
        DataWrapper { data }
    }
}

pub struct CreatableOutputContext<'a> {
    pub settings: &'a obs_data_t,
    pub output: &'a obs_output_t,
}

pub unsafe extern "C" fn create_default_data<D: Default>(
    _settings: *mut obs_data_t,
    _output: *mut obs_output_t,
) -> *mut c_void {
    let data = Box::new(DataWrapper::<D>::default());
    Box::into_raw(data) as *mut c_void
}

pub unsafe extern "C" fn create<D: CreatableOutput>(
    settings: *mut obs_data_t,
    output: *mut obs_output_t,
) -> *mut c_void {
    let data = D::create(CreatableOutputContext {
        settings: &*settings,
        output: &*output,
    });
    let data_wrapper = Box::new(DataWrapper::from(data));
    Box::into_raw(data_wrapper) as *mut c_void
}

pub unsafe extern "C" fn destroy<D>(data: *mut c_void) {
    let wrapper: Box<DataWrapper<D>> = Box::from_raw(data as *mut DataWrapper<D>);
    drop(wrapper);
}

macro_rules! impl_simple_fn {
    ($($name:ident$(($($params_name:tt:$params_ty:ty),*))? => $trait:ident $(-> $ret:ty)?)*) => ($(
        item! {
            pub unsafe extern "C" fn $name<D: $trait>(
                data: *mut ::std::os::raw::c_void,
                $($($params_name:$params_ty),*)?
            ) $(-> $ret)? {
                let wrapper = &mut *(data as *mut DataWrapper<D>);
                D::$name(&mut wrapper.data $(,$($params_name),*)?)
            }
        }
    )*)
}

pub unsafe extern "C" fn get_name<D: GetNameOutput>(
    _type_data: *mut c_void,
) -> *const c_char {
    D::get_name().as_ptr()
}

impl_simple_fn! {
    start => StartOutput -> bool
    stop(ts: u64) => StopOutput
}

pub unsafe extern "C" fn raw_video<D: RawVideoOutput>(
    data: *mut c_void,
    frame: *mut video_data,
) {
    let wrapper = &mut *(data as *mut DataWrapper<D>);
    D::raw_video(&mut wrapper.data, &mut *frame)
}

pub unsafe extern "C" fn raw_audio<D: RawAudioOutput>(
    data: *mut c_void,
    frame: *mut audio_data,
) {
    let wrapper = &mut *(data as *mut DataWrapper<D>);
    D::raw_audio(&mut wrapper.data, &mut *frame)
}

pub unsafe extern "C" fn raw_audio2<D: RawAudio2Output>(
    data: *mut c_void,
    idx: u64,
    frame: *mut audio_data,
) {
    let wrapper = &mut *(data as *mut DataWrapper<D>);
    D::raw_audio2(&mut wrapper.data, idx as usize, &mut *frame)
}

pub unsafe extern "C" fn encoded_packet<D: EncodedPacketOutput>(
    data: *mut c_void,
    packet: *mut encoder_packet,
) {
    let wrapper = &mut *(data as *mut DataWrapper<D>);
    D::encoded_packet(&mut wrapper.data, &mut *packet)
}

pub unsafe extern "C" fn update<D: UpdateOutput>(
    data: *mut c_void,
    settings: *mut obs_data_t,
) {
    let data: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    let mut settings = DataObj::from_raw(settings);
    D::update(&mut data.data, &mut settings);
    forget(settings);
}

pub unsafe extern "C" fn get_defaults<D: GetDefaultsOutput>(settings: *mut obs_data_t) {
    let mut settings = DataObj::from_raw(settings);
    D::get_defaults(&mut settings);
    forget(settings);
}

// pub unsafe extern "C" fn get_defaults2<D: GetDefaults2Output>(
//     data: *mut c_void,
//     settings: *mut obs_data_t,
// ) {
//     let mut settings = DataObj::from_raw(settings);
//     D::get_defaults2(??, &mut settings);
//     forget(settings);
// }

pub unsafe extern "C" fn get_properties<D: GetPropertiesOutput>(
    data: *mut ::std::os::raw::c_void,
) -> *mut obs_properties {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);
    let properties = D::get_properties(&mut wrapper.data);
    properties.into_raw()
}

impl_simple_fn! {
    get_total_bytes => GetTotalBytesOutput -> u64
    get_dropped_frames => GetDroppedFramesOutput-> c_int
    get_congestion => GetCongestionOutput -> f32
    get_connect_time_ms => GetConnectTimeMsOutput -> c_int
}
