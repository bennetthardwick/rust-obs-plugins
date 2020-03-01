use obs_sys::{
    gs_address_mode, gs_address_mode_GS_ADDRESS_BORDER, gs_address_mode_GS_ADDRESS_CLAMP,
    gs_address_mode_GS_ADDRESS_MIRROR, gs_address_mode_GS_ADDRESS_MIRRORONCE,
    gs_address_mode_GS_ADDRESS_WRAP, gs_color_format, gs_color_format_GS_A8,
    gs_color_format_GS_BGRA, gs_color_format_GS_BGRX, gs_color_format_GS_DXT1,
    gs_color_format_GS_DXT3, gs_color_format_GS_DXT5, gs_color_format_GS_R10G10B10A2,
    gs_color_format_GS_R16, gs_color_format_GS_R16F, gs_color_format_GS_R32F,
    gs_color_format_GS_R8, gs_color_format_GS_R8G8, gs_color_format_GS_RG16F,
    gs_color_format_GS_RG32F, gs_color_format_GS_RGBA, gs_color_format_GS_RGBA16,
    gs_color_format_GS_RGBA16F, gs_color_format_GS_RGBA32F, gs_color_format_GS_UNKNOWN,
    gs_effect_create, gs_effect_destroy, gs_effect_get_param_by_name, gs_effect_set_vec2,
    gs_effect_t, gs_eparam_t, gs_sample_filter, gs_sample_filter_GS_FILTER_ANISOTROPIC,
    gs_sample_filter_GS_FILTER_LINEAR, gs_sample_filter_GS_FILTER_MIN_LINEAR_MAG_MIP_POINT,
    gs_sample_filter_GS_FILTER_MIN_LINEAR_MAG_POINT_MIP_LINEAR,
    gs_sample_filter_GS_FILTER_MIN_MAG_LINEAR_MIP_POINT,
    gs_sample_filter_GS_FILTER_MIN_MAG_POINT_MIP_LINEAR,
    gs_sample_filter_GS_FILTER_MIN_POINT_MAG_LINEAR_MIP_POINT,
    gs_sample_filter_GS_FILTER_MIN_POINT_MAG_MIP_LINEAR, gs_sample_filter_GS_FILTER_POINT,
    gs_sampler_info, gs_samplerstate_create, gs_samplerstate_destroy, gs_samplerstate_t,
    obs_allow_direct_render, obs_allow_direct_render_OBS_ALLOW_DIRECT_RENDERING,
    obs_allow_direct_render_OBS_NO_DIRECT_RENDERING, obs_enter_graphics, obs_leave_graphics, vec2,
};

use super::ObsString;

pub struct GraphicsEffect {
    raw: *mut gs_effect_t,
}

impl GraphicsEffect {
    pub fn from_effect_string(value: ObsString, name: ObsString) -> Option<Self> {
        unsafe {
            obs_enter_graphics();
            let raw = gs_effect_create(value.as_ptr(), name.as_ptr(), std::ptr::null_mut());
            obs_leave_graphics();

            if raw.is_null() {
                None
            } else {
                Some(Self { raw })
            }
        }
    }

    pub fn get_effect_param_by_name(&mut self, name: ObsString) -> Option<GraphicsEffectParam> {
        unsafe {
            let pointer = gs_effect_get_param_by_name(self.raw, name.as_ptr());
            if !pointer.is_null() {
                Some(GraphicsEffectParam::from_raw(pointer))
            } else {
                None
            }
        }
    }

    pub unsafe fn as_ptr(&self) -> *mut gs_effect_t {
        self.raw
    }
}

impl Drop for GraphicsEffect {
    fn drop(&mut self) {
        unsafe {
            obs_enter_graphics();
            gs_effect_destroy(self.raw);
            obs_leave_graphics();
        }
    }
}

pub struct GraphicsEffectParam {
    raw: *mut gs_eparam_t,
}

impl GraphicsEffectParam {
    pub unsafe fn from_raw(raw: *mut gs_eparam_t) -> Self {
        Self { raw }
    }

    pub fn set_vec2(&mut self, _context: &GraphicsEffectContext, value: &Vec2) {
        unsafe {
            gs_effect_set_vec2(self.raw, &value.raw);
        }
    }
}

pub enum GraphicsAddressMode {
    Clamp,
    Wrap,
    Mirror,
    Border,
    MirrorOnce,
}

impl GraphicsAddressMode {
    pub fn as_raw(&self) -> gs_address_mode {
        match self {
            GraphicsAddressMode::Clamp => gs_address_mode_GS_ADDRESS_CLAMP,
            GraphicsAddressMode::Wrap => gs_address_mode_GS_ADDRESS_WRAP,
            GraphicsAddressMode::Mirror => gs_address_mode_GS_ADDRESS_MIRROR,
            GraphicsAddressMode::Border => gs_address_mode_GS_ADDRESS_BORDER,
            GraphicsAddressMode::MirrorOnce => gs_address_mode_GS_ADDRESS_MIRRORONCE,
        }
    }
}

pub enum GraphicsSampleFilter {
    Point,
    Linear,
    Anisotropic,
    MinMagPointMipLinear,
    MinPointMagLinearMipPoint,
    MinPointMagMipLinear,
    MinLinearMapMipPoint,
    MinLinearMagPointMipLinear,
    MinMagLinearMipPoint,
}

impl GraphicsSampleFilter {
    fn as_raw(&self) -> gs_sample_filter {
        match self {
            GraphicsSampleFilter::Point => gs_sample_filter_GS_FILTER_POINT,
            GraphicsSampleFilter::Linear => gs_sample_filter_GS_FILTER_LINEAR,
            GraphicsSampleFilter::Anisotropic => gs_sample_filter_GS_FILTER_ANISOTROPIC,
            GraphicsSampleFilter::MinMagPointMipLinear => {
                gs_sample_filter_GS_FILTER_MIN_MAG_POINT_MIP_LINEAR
            }
            GraphicsSampleFilter::MinPointMagLinearMipPoint => {
                gs_sample_filter_GS_FILTER_MIN_POINT_MAG_LINEAR_MIP_POINT
            }
            GraphicsSampleFilter::MinPointMagMipLinear => {
                gs_sample_filter_GS_FILTER_MIN_POINT_MAG_MIP_LINEAR
            }
            GraphicsSampleFilter::MinLinearMapMipPoint => {
                gs_sample_filter_GS_FILTER_MIN_LINEAR_MAG_MIP_POINT
            }
            GraphicsSampleFilter::MinLinearMagPointMipLinear => {
                gs_sample_filter_GS_FILTER_MIN_LINEAR_MAG_POINT_MIP_LINEAR
            }
            GraphicsSampleFilter::MinMagLinearMipPoint => {
                gs_sample_filter_GS_FILTER_MIN_MAG_LINEAR_MIP_POINT
            }
        }
    }
}

pub struct GraphicsSamplerInfo {
    info: gs_sampler_info,
}

impl GraphicsSamplerInfo {
    pub fn new() -> Self {
        Self {
            info: gs_sampler_info {
                address_u: GraphicsAddressMode::Clamp.as_raw(),
                address_v: GraphicsAddressMode::Clamp.as_raw(),
                address_w: GraphicsAddressMode::Clamp.as_raw(),
                max_anisotropy: 0,
                border_color: 0,
                filter: GraphicsSampleFilter::Point.as_raw(),
            },
        }
    }
}

impl Default for GraphicsSamplerInfo {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GraphicsSamplerState {
    raw: *mut gs_samplerstate_t,
}

impl From<GraphicsSamplerInfo> for GraphicsSamplerState {
    fn from(info: GraphicsSamplerInfo) -> GraphicsSamplerState {
        unsafe {
            obs_enter_graphics();
            let raw = gs_samplerstate_create(&info.info);
            obs_leave_graphics();

            GraphicsSamplerState { raw }
        }
    }
}

impl Drop for GraphicsSamplerState {
    fn drop(&mut self) {
        unsafe {
            obs_enter_graphics();
            gs_samplerstate_destroy(self.raw);
            obs_leave_graphics();
        }
    }
}

pub struct GraphicsEffectContext {}

impl GraphicsEffectContext {
    pub unsafe fn new() -> Self {
        Self {}
    }
}

impl GraphicsEffectContext {}

pub enum GraphicsColorFormat {
    UNKNOWN,
    A8,
    R8,
    RGBA,
    BGRX,
    BGRA,
    R10G10B10A2,
    RGBA16,
    R16,
    RGBA16F,
    RGBA32F,
    RG16F,
    RG32F,
    R16F,
    R32F,
    DXT1,
    DXT3,
    DXT5,
    R8G8,
}

impl GraphicsColorFormat {
    pub fn as_raw(&self) -> gs_color_format {
        match self {
            GraphicsColorFormat::UNKNOWN => gs_color_format_GS_UNKNOWN,
            GraphicsColorFormat::A8 => gs_color_format_GS_A8,
            GraphicsColorFormat::R8 => gs_color_format_GS_R8,
            GraphicsColorFormat::RGBA => gs_color_format_GS_RGBA,
            GraphicsColorFormat::BGRX => gs_color_format_GS_BGRX,
            GraphicsColorFormat::BGRA => gs_color_format_GS_BGRA,
            GraphicsColorFormat::R10G10B10A2 => gs_color_format_GS_R10G10B10A2,
            GraphicsColorFormat::RGBA16 => gs_color_format_GS_RGBA16,
            GraphicsColorFormat::R16 => gs_color_format_GS_R16,
            GraphicsColorFormat::RGBA16F => gs_color_format_GS_RGBA16F,
            GraphicsColorFormat::RGBA32F => gs_color_format_GS_RGBA32F,
            GraphicsColorFormat::RG16F => gs_color_format_GS_RG16F,
            GraphicsColorFormat::RG32F => gs_color_format_GS_RG32F,
            GraphicsColorFormat::R16F => gs_color_format_GS_R16F,
            GraphicsColorFormat::R32F => gs_color_format_GS_R32F,
            GraphicsColorFormat::DXT1 => gs_color_format_GS_DXT1,
            GraphicsColorFormat::DXT3 => gs_color_format_GS_DXT3,
            GraphicsColorFormat::DXT5 => gs_color_format_GS_DXT5,
            GraphicsColorFormat::R8G8 => gs_color_format_GS_R8G8,
        }
    }
}

pub enum GraphicsAllowDirectRendering {
    NoDirectRendering,
    AllowDirectRendering,
}

impl GraphicsAllowDirectRendering {
    pub fn as_raw(&self) -> obs_allow_direct_render {
        match self {
            GraphicsAllowDirectRendering::NoDirectRendering => {
                obs_allow_direct_render_OBS_NO_DIRECT_RENDERING
            }
            GraphicsAllowDirectRendering::AllowDirectRendering => {
                obs_allow_direct_render_OBS_ALLOW_DIRECT_RENDERING
            }
        }
    }
}

pub struct Vec2 {
    raw: vec2,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Vec2 {
        let mut v = Vec2 {
            raw: vec2::default(),
        };
        v.set(x, y);
        v
    }

    pub fn x(&self) -> f32 {
        unsafe { self.raw.__bindgen_anon_1.__bindgen_anon_1.x }
    }

    pub fn y(&self) -> f32 {
        unsafe { self.raw.__bindgen_anon_1.__bindgen_anon_1.y }
    }

    #[inline]
    pub fn set(&mut self, x: f32, y: f32) {
        self.raw.__bindgen_anon_1.__bindgen_anon_1.x = x;
        self.raw.__bindgen_anon_1.__bindgen_anon_1.y = y;
    }

    pub fn as_ptr(&mut self) -> *mut vec2 {
        &mut self.raw
    }
}

impl Default for Vec2 {
    fn default() -> Self {
        let mut v = Vec2 {
            raw: vec2::default(),
        };
        v.set(0., 0.);
        v
    }
}
