use std::mem;

use obs_sys::{
    obs_source_frame,
    video_output_get_width,
    video_output_get_height,
    video_output_get_frame_rate,
    video_output_get_format,
    video_t, video_data,
    video_format_VIDEO_FORMAT_NONE, video_format_VIDEO_FORMAT_I420, video_format_VIDEO_FORMAT_NV12, video_format_VIDEO_FORMAT_YVYU,
    video_format_VIDEO_FORMAT_YUY2, video_format_VIDEO_FORMAT_UYVY, video_format_VIDEO_FORMAT_RGBA, video_format_VIDEO_FORMAT_BGRA,
    video_format_VIDEO_FORMAT_BGRX, video_format_VIDEO_FORMAT_Y800, video_format_VIDEO_FORMAT_I444, video_format_VIDEO_FORMAT_BGR3,
    video_format_VIDEO_FORMAT_I422, video_format_VIDEO_FORMAT_I40A, video_format_VIDEO_FORMAT_I42A, video_format_VIDEO_FORMAT_YUVA,
    video_format_VIDEO_FORMAT_AYUV,
};

#[derive(Debug, Clone, Copy, Eq)]
pub enum VideoFormat {
    Unknown,
    None,
    /// planar 4:2:0 formats, three-plane
    I420,
    /// planar 4:2:0 formats, two-plane, luma and packed chroma
    NV12,

    /// packed 4:2:2 formats
    YVYU,
    /// packed 4:2:2 formats, YUYV
    YUY2,
    /// packed 4:2:2 formats
    UYVY,

    /// packed uncompressed formats
    RGBA,
    /// packed uncompressed formats
    BGRA,
    /// packed uncompressed formats
    BGRX,
    /// packed uncompressed formats, grayscale
    Y800,

    /// planar 4:4:4
    I444,
    /// more packed uncompressed formats
    BGR3,
    /// planar 4:2:2
    I422,
    /// planar 4:2:0 with alpha
    I40A,
    /// planar 4:2:2 with alpha
    I42A,
    /// planar 4:4:4 with alpha
    YUVA,
    /// packed 4:4:4 with alpha
    AYUV,

    /// planar 4:2:0 format, 10 bpp, three-plane
    I010,
    /// planar 4:2:0 format, 10 bpp, two-plane, luma and packed chroma
    P010,
    /// planar 4:2:2 10 bits, Little Endian
    I210,
    /// planar 4:4:4 12 bits, Little Endian
    I412,
    /// planar 4:4:4 12 bits with alpha, Little Endian
    YA2L,
}

impl PartialEq for VideoFormat {
    fn eq(&self, other: &Self) -> bool {
        if matches!(self, VideoFormat::Unknown) || matches!(other, VideoFormat::Unknown) {
            false
        } else {
            mem::discriminant(self) == mem::discriminant(other)
        }
    }
}

impl From<u32> for VideoFormat {
    #[allow(non_upper_case_globals)]
    fn from(raw: u32) -> Self {
        match raw {
            video_format_VIDEO_FORMAT_NONE => VideoFormat::None,
            video_format_VIDEO_FORMAT_I420 => VideoFormat::I420,
            video_format_VIDEO_FORMAT_NV12 => VideoFormat::NV12,
            video_format_VIDEO_FORMAT_YVYU => VideoFormat::YVYU,
            video_format_VIDEO_FORMAT_YUY2 => VideoFormat::YUY2,
            video_format_VIDEO_FORMAT_UYVY => VideoFormat::UYVY,
            video_format_VIDEO_FORMAT_RGBA => VideoFormat::RGBA,
            video_format_VIDEO_FORMAT_BGRA => VideoFormat::BGRA,
            video_format_VIDEO_FORMAT_BGRX => VideoFormat::BGRX,
            video_format_VIDEO_FORMAT_Y800 => VideoFormat::Y800,
            video_format_VIDEO_FORMAT_I444 => VideoFormat::I444,
            video_format_VIDEO_FORMAT_BGR3 => VideoFormat::BGR3,
            video_format_VIDEO_FORMAT_I422 => VideoFormat::I422,
            video_format_VIDEO_FORMAT_I40A => VideoFormat::I40A,
            video_format_VIDEO_FORMAT_I42A => VideoFormat::I42A,
            video_format_VIDEO_FORMAT_YUVA => VideoFormat::YUVA,
            video_format_VIDEO_FORMAT_AYUV => VideoFormat::AYUV,
            17 => VideoFormat::I010,
            18 => VideoFormat::P010,
            19 => VideoFormat::I210,
            20 => VideoFormat::I412,
            21 => VideoFormat::YA2L,
            _ => VideoFormat::Unknown,
        }
    }
}

pub struct VideoDataSourceContext {
    pointer: *mut obs_source_frame,
}

impl VideoDataSourceContext {
    pub fn from_raw(pointer: *mut obs_source_frame) -> Self {
        Self { pointer }
    }

    pub fn format(&self) -> VideoFormat {
        let raw = unsafe { (*self.pointer).format };

        VideoFormat::from(raw as u32)
    }

    pub fn width(&self) -> u32 {
        unsafe { (*self.pointer).width }
    }

    pub fn height(&self) -> u32 {
        unsafe { (*self.pointer).height }
    }

    pub fn data_buffer(&self, idx: usize) -> *mut u8 {
        unsafe { (*self.pointer).data[idx] }
    }

    pub fn linesize(&self, idx: usize) -> u32 {
        unsafe { (*self.pointer).linesize[idx] }
    }

    pub fn timestamp(&self) -> u64 {
        unsafe { (*self.pointer).timestamp }
    }
}

pub struct VideoDataOutputContext {
    pointer: *mut video_data,
}

impl VideoDataOutputContext {
    pub fn from_raw(pointer: *mut video_data) -> Self {
        Self { pointer }
    }

    pub fn data_buffer(&self, idx: usize) -> *mut u8 {
        unsafe { (*self.pointer).data[idx] }
    }

    pub fn linesize(&self, idx: usize) -> u32 {
        unsafe { (*self.pointer).linesize[idx] }
    }

    pub fn timestamp(&self) -> u64 {
        unsafe { (*self.pointer).timestamp }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub struct VideoInfo {
    pub width: u32,
    pub height: u32,
    pub frame_rate: f64,
    pub format: VideoFormat,
}

pub enum FrameSize {
    Unknown,
    Planes { size: usize, count: usize },
    OnePlane(usize),
    TwoPlane(usize, usize),
    ThreePlane(usize, usize, usize),
    FourPlane(usize, usize, usize, usize),
}

impl VideoInfo {
    /// see https://github.com/obsproject/obs-studio/blob/a1e8075fba09f3b56ed43ead64cc3e340dd7a059/libobs/media-io/video-frame.c#L23
    pub fn frame_size(&self) -> FrameSize {
        use VideoFormat::*;
        let width = self.width as usize;
        let height = self.height as usize;
        let half_width = (width + 1) / 2;
        let half_height = (height + 1) / 2;
        let full_size = width * height;
        let half_size = half_width * height;
        let quarter_size = half_width * half_height;
        if self.format == VideoFormat::Unknown {
            return FrameSize::Unknown;
        };
        match self.format {
            VideoFormat::None => FrameSize::Planes { size: 0, count: 0 },
            I420 => FrameSize::ThreePlane(full_size, quarter_size, quarter_size),
            NV12 => FrameSize::TwoPlane(full_size, half_size*2),
            Y800 => FrameSize::OnePlane(full_size),
            YVYU | UYVY | YUY2 => FrameSize::OnePlane(half_size*4),
            BGRX | BGRA | RGBA | AYUV => FrameSize::OnePlane(full_size*4),
            I444 => FrameSize::Planes { count: 3, size: full_size },
            I412 => FrameSize::Planes { count: 3, size: full_size*2 },
            BGR3 => FrameSize::OnePlane(full_size*3),
            I422 => FrameSize::ThreePlane(full_size, half_size, half_size),
            I210 => FrameSize::ThreePlane(full_size*2, half_size*2, half_size*2),
            I40A => FrameSize::FourPlane(full_size, quarter_size, quarter_size, full_size),
            I42A => FrameSize::FourPlane(full_size, half_size, half_size, full_size),
            YUVA => FrameSize::Planes { count: 4, size: full_size },
            YA2L => FrameSize::Planes { count: 4, size: full_size*2 },
            I010 => FrameSize::ThreePlane(full_size*2, quarter_size*2, quarter_size*2),
            P010 => FrameSize::TwoPlane(full_size*2, quarter_size*4),
            Unknown => FrameSize::Unknown,
        }
    }
}

#[allow(unused)]
pub struct VideoRef {
    pub pointer: *mut video_t,
}

#[allow(unused)]
impl VideoRef {
    pub fn from_raw(pointer: *mut video_t) -> Self {
        Self { pointer }
    }

    pub fn info(&self) -> VideoInfo {
        VideoInfo {
            width: self.width(),
            height: self.height(),
            frame_rate: self.frame_rate(),
            format: self.format()
        }
    }

    pub fn width(&self) -> u32 {
        unsafe { video_output_get_width(self.pointer) }
    }

    pub fn height(&self) -> u32 {
        unsafe { video_output_get_height(self.pointer) }
    }

    pub fn frame_rate(&self) -> f64 {
        unsafe { video_output_get_frame_rate(self.pointer) }
    }

    pub fn format(&self) -> VideoFormat {
        let raw = unsafe { video_output_get_format(self.pointer) };

        VideoFormat::from(raw as u32)
    }
}
