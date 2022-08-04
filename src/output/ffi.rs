use super::{traits::*, OutputContext, CreatableOutputContext};
use crate::hotkey::Hotkey;
use crate::string::ObsString;
use crate::{data::DataObj, wrapper::PtrWrapper};
use obs_sys::{audio_data, encoder_packet, obs_properties, video_data, obs_hotkey_id, obs_hotkey_register_output, obs_hotkey_t};
use paste::item;
use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::forget;
use std::os::raw::{c_char, c_int};

use obs_sys::{
    obs_data_t, obs_output_t
};

struct DataWrapper<D> {
    data: D,
    hotkey_callbacks: HashMap<obs_hotkey_id, Box<dyn FnMut(&mut Hotkey, &mut D)>>,
}

impl<D> DataWrapper<D> {
    pub(crate) unsafe fn register_callbacks(
        &mut self,
        callbacks: Vec<(
            ObsString,
            ObsString,
            Box<dyn FnMut(&mut Hotkey, &mut D)>,
        )>,
        output: *mut obs_output_t,
        data: *mut c_void,
    ) {
        for (name, description, func) in callbacks.into_iter() {
            let id = obs_hotkey_register_output(
                output,
                name.as_ptr(),
                description.as_ptr(),
                Some(hotkey_callback::<D>),
                data,
            );

            self.hotkey_callbacks.insert(id, func);
        }
    }
}

impl<D> From<D> for DataWrapper<D> {
    fn from(data: D) -> Self {
        DataWrapper {
            data,
            hotkey_callbacks: HashMap::new(),
        }
    }
}

pub unsafe extern "C" fn create<D: Outputable>(
    settings: *mut obs_data_t,
    output: *mut obs_output_t,
) -> *mut c_void {
    let settings = DataObj::from_raw(settings);
    let mut context = CreatableOutputContext::from_raw(settings);
    let output_context = OutputContext::from_raw(output);

    let data = D::create(&mut context, output_context);
    let wrapper = Box::new(DataWrapper::from(data));
    forget(context.settings);
    let callbacks = context.hotkey_callbacks;

    let pointer = Box::into_raw(wrapper);

    pointer
        .as_mut()
        .unwrap()
        .register_callbacks(callbacks, output, pointer as *mut c_void);

    return pointer as *mut c_void;
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
    start => Outputable -> bool
    stop(ts: u64) => Outputable
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

pub unsafe extern "C" fn hotkey_callback<D>(
    data: *mut c_void,
    id: obs_hotkey_id,
    hotkey: *mut obs_hotkey_t,
    pressed: bool,
) {
    let wrapper: &mut DataWrapper<D> = &mut *(data as *mut DataWrapper<D>);

    let data = &mut wrapper.data;
    let hotkey_callbacks = &mut wrapper.hotkey_callbacks;
    let mut key = Hotkey::from_raw(hotkey, pressed);

    if let Some(callback) = hotkey_callbacks.get_mut(&id) {
        callback(&mut key, data);
    }
}
