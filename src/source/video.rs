use obs_sys::{
    obs_source_frame, video_output_get_format, video_output_get_frame_rate,
    video_output_get_height, video_output_get_width, video_t,
};

#[derive(Debug, PartialEq)]
pub enum VideoFormat {
    Unknown,
    None,
    I420,
    NV12,
    YVYU,
    YUY2,
    UYVY,
    RGBA,
    BGRA,
    BGRX,
    Y800,
    I444,
    BGR3,
    I422,
    I40A,
    I42A,
    YUVA,
    AYUV,
}

impl From<u32> for VideoFormat {
    fn from(raw: u32) -> Self {
        match raw {
            0 => VideoFormat::None,
            1 => VideoFormat::I420,
            2 => VideoFormat::NV12,
            3 => VideoFormat::YVYU,
            4 => VideoFormat::YUY2,
            5 => VideoFormat::UYVY,
            6 => VideoFormat::RGBA,
            7 => VideoFormat::BGRA,
            8 => VideoFormat::BGRX,
            9 => VideoFormat::Y800,
            10 => VideoFormat::I444,
            11 => VideoFormat::BGR3,
            12 => VideoFormat::I422,
            13 => VideoFormat::I40A,
            14 => VideoFormat::I42A,
            15 => VideoFormat::YUVA,
            16 => VideoFormat::AYUV,
            _ => VideoFormat::Unknown,
        }
    }
}

pub struct VideoDataContext {
    pointer: *mut obs_source_frame,
}

impl VideoDataContext {
    pub(crate) unsafe fn from_raw(pointer: *mut obs_source_frame) -> Self {
        Self { pointer }
    }

    pub fn get_format(&self) -> VideoFormat {
        let raw = unsafe { (*self.pointer).format };

        VideoFormat::from(raw as u32)
    }

    pub fn get_width(&self) -> u32 {
        unsafe { (*self.pointer).width }
    }

    pub fn get_height(&self) -> u32 {
        unsafe { (*self.pointer).height }
    }

    pub fn get_data_buffer(&self, idx: usize) -> *mut u8 {
        unsafe { (*self.pointer).data[idx] }
    }

    pub fn get_linesize(&self, idx: usize) -> u32 {
        unsafe { (*self.pointer).linesize[idx] }
    }
}

pub struct VideoRef {
    pointer: *mut video_t,
}

impl VideoRef {
    pub(crate) unsafe fn from_raw(pointer: *mut video_t) -> Self {
        Self { pointer }
    }

    pub(crate) fn get_width(&self) -> u32 {
        unsafe { video_output_get_width(self.pointer) }
    }

    pub(crate) fn get_height(&self) -> u32 {
        unsafe { video_output_get_height(self.pointer) }
    }

    pub(crate) fn get_frame_rate(&self) -> f64 {
        unsafe { video_output_get_frame_rate(self.pointer) }
    }

    pub(crate) fn get_format(&self) -> VideoFormat {
        let raw = unsafe { video_output_get_format(self.pointer) };

        VideoFormat::from(raw as u32)
    }
}
