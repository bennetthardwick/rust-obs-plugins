#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
#![allow(clippy::all)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub type size_t = usize; // Why isn't this in the bindings???
