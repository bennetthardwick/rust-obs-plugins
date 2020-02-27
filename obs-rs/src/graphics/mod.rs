use obs_sys::{
    gs_address_mode, gs_address_mode_GS_ADDRESS_BORDER, gs_address_mode_GS_ADDRESS_CLAMP,
    gs_address_mode_GS_ADDRESS_MIRROR, gs_address_mode_GS_ADDRESS_MIRRORONCE,
    gs_address_mode_GS_ADDRESS_WRAP, gs_effect_create, gs_effect_destroy,
    gs_effect_get_param_by_name, gs_effect_t, gs_eparam_t, gs_sample_filter,
    gs_sample_filter_GS_FILTER_ANISOTROPIC, gs_sample_filter_GS_FILTER_LINEAR,
    gs_sample_filter_GS_FILTER_MIN_LINEAR_MAG_MIP_POINT,
    gs_sample_filter_GS_FILTER_MIN_LINEAR_MAG_POINT_MIP_LINEAR,
    gs_sample_filter_GS_FILTER_MIN_MAG_LINEAR_MIP_POINT,
    gs_sample_filter_GS_FILTER_MIN_MAG_POINT_MIP_LINEAR,
    gs_sample_filter_GS_FILTER_MIN_POINT_MAG_LINEAR_MIP_POINT,
    gs_sample_filter_GS_FILTER_MIN_POINT_MAG_MIP_LINEAR, gs_sample_filter_GS_FILTER_POINT,
    gs_sampler_info, gs_samplerstate_create, gs_samplerstate_destroy, gs_samplerstate_t,
    obs_enter_graphics, obs_leave_graphics,
};

use super::ObsString;

pub struct GraphicsEffect {
    raw: *mut gs_effect_t,
}

impl GraphicsEffect {
    pub unsafe fn from_effect_string(value: ObsString) -> Option<Self> {
        obs_enter_graphics();
        let raw = gs_effect_create(value.as_ptr(), std::ptr::null_mut(), std::ptr::null_mut());
        obs_leave_graphics();

        if !raw.is_null() {
            None
        } else {
            Some(Self { raw })
        }
    }

    pub fn get_effect_by_name(&mut self, name: ObsString) -> Option<GraphicsEffectParam> {
        unsafe {
            let pointer = gs_effect_get_param_by_name(self.raw, name.as_ptr());
            if !pointer.is_null() {
                Some(GraphicsEffectParam::from_raw(pointer))
            } else {
                None
            }
        }
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
