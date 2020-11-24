use crate::{Error, Result};
use core::convert::TryFrom;
use core::ptr::null_mut;
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
    gs_effect_create, gs_effect_destroy, gs_effect_get_param_by_name, gs_effect_get_param_info,
    gs_effect_param_info, gs_effect_set_next_sampler, gs_effect_set_vec2, gs_effect_t, gs_eparam_t,
    gs_sample_filter, gs_sample_filter_GS_FILTER_ANISOTROPIC, gs_sample_filter_GS_FILTER_LINEAR,
    gs_sample_filter_GS_FILTER_MIN_LINEAR_MAG_MIP_POINT,
    gs_sample_filter_GS_FILTER_MIN_LINEAR_MAG_POINT_MIP_LINEAR,
    gs_sample_filter_GS_FILTER_MIN_MAG_LINEAR_MIP_POINT,
    gs_sample_filter_GS_FILTER_MIN_MAG_POINT_MIP_LINEAR,
    gs_sample_filter_GS_FILTER_MIN_POINT_MAG_LINEAR_MIP_POINT,
    gs_sample_filter_GS_FILTER_MIN_POINT_MAG_MIP_LINEAR, gs_sample_filter_GS_FILTER_POINT,
    gs_sampler_info, gs_samplerstate_create, gs_samplerstate_destroy, gs_samplerstate_t,
    gs_shader_param_type, gs_shader_param_type_GS_SHADER_PARAM_BOOL,
    gs_shader_param_type_GS_SHADER_PARAM_FLOAT, gs_shader_param_type_GS_SHADER_PARAM_INT,
    gs_shader_param_type_GS_SHADER_PARAM_INT2, gs_shader_param_type_GS_SHADER_PARAM_INT3,
    gs_shader_param_type_GS_SHADER_PARAM_INT4, gs_shader_param_type_GS_SHADER_PARAM_MATRIX4X4,
    gs_shader_param_type_GS_SHADER_PARAM_STRING, gs_shader_param_type_GS_SHADER_PARAM_TEXTURE,
    gs_shader_param_type_GS_SHADER_PARAM_UNKNOWN, gs_shader_param_type_GS_SHADER_PARAM_VEC2,
    gs_shader_param_type_GS_SHADER_PARAM_VEC3, gs_shader_param_type_GS_SHADER_PARAM_VEC4,
    gs_texture_create, gs_texture_destroy, gs_texture_get_height, gs_texture_get_width,
    gs_texture_map, gs_texture_set_image, gs_texture_t, gs_texture_unmap, obs_allow_direct_render,
    obs_allow_direct_render_OBS_ALLOW_DIRECT_RENDERING,
    obs_allow_direct_render_OBS_NO_DIRECT_RENDERING, obs_enter_graphics, obs_leave_graphics,
    obs_source_draw, vec2, vec3, vec4, GS_DYNAMIC,
};
use paste::item;
use std::{
    ffi::{CStr, CString},
    ptr,
};
use std::{os::raw::c_int, slice};

use super::string::ObsString;

/// Guard to guarantee that we exit graphics context properly.
/// This does not prevent one from calling APIs that are not supposed to be called outside of the context.
struct GraphicsGuard;

impl GraphicsGuard {
    fn enter() -> Self {
        unsafe {
            obs_enter_graphics();
        }
        Self
    }

    pub fn with_enter<T, F: FnOnce() -> T>(f: F) -> T {
        let _g = Self::enter();
        f()
    }
}

impl Drop for GraphicsGuard {
    fn drop(&mut self) {
        unsafe {
            obs_leave_graphics();
        }
    }
}

#[derive(Clone, Copy)]
pub enum ShaderParamType {
    Unknown,
    Bool,
    Float,
    Int,
    String,
    Vec2,
    Vec3,
    Vec4,
    Int2,
    Int3,
    Int4,
    Mat4,
    Texture,
}

impl ShaderParamType {
    pub fn as_raw(&self) -> gs_shader_param_type {
        match self {
            ShaderParamType::Unknown => gs_shader_param_type_GS_SHADER_PARAM_UNKNOWN,
            ShaderParamType::Bool => gs_shader_param_type_GS_SHADER_PARAM_BOOL,
            ShaderParamType::Float => gs_shader_param_type_GS_SHADER_PARAM_FLOAT,
            ShaderParamType::Int => gs_shader_param_type_GS_SHADER_PARAM_INT,
            ShaderParamType::String => gs_shader_param_type_GS_SHADER_PARAM_STRING,
            ShaderParamType::Vec2 => gs_shader_param_type_GS_SHADER_PARAM_VEC2,
            ShaderParamType::Vec3 => gs_shader_param_type_GS_SHADER_PARAM_VEC3,
            ShaderParamType::Vec4 => gs_shader_param_type_GS_SHADER_PARAM_VEC4,
            ShaderParamType::Int2 => gs_shader_param_type_GS_SHADER_PARAM_INT2,
            ShaderParamType::Int3 => gs_shader_param_type_GS_SHADER_PARAM_INT3,
            ShaderParamType::Int4 => gs_shader_param_type_GS_SHADER_PARAM_INT4,
            ShaderParamType::Mat4 => gs_shader_param_type_GS_SHADER_PARAM_MATRIX4X4,
            ShaderParamType::Texture => gs_shader_param_type_GS_SHADER_PARAM_TEXTURE,
        }
    }

    #[allow(non_upper_case_globals)]
    pub fn from_raw(param_type: gs_shader_param_type) -> Self {
        match param_type {
            gs_shader_param_type_GS_SHADER_PARAM_UNKNOWN => ShaderParamType::Unknown,
            gs_shader_param_type_GS_SHADER_PARAM_BOOL => ShaderParamType::Bool,
            gs_shader_param_type_GS_SHADER_PARAM_FLOAT => ShaderParamType::Float,
            gs_shader_param_type_GS_SHADER_PARAM_INT => ShaderParamType::Int,
            gs_shader_param_type_GS_SHADER_PARAM_STRING => ShaderParamType::String,
            gs_shader_param_type_GS_SHADER_PARAM_VEC2 => ShaderParamType::Vec2,
            gs_shader_param_type_GS_SHADER_PARAM_VEC3 => ShaderParamType::Vec3,
            gs_shader_param_type_GS_SHADER_PARAM_VEC4 => ShaderParamType::Vec4,
            gs_shader_param_type_GS_SHADER_PARAM_INT2 => ShaderParamType::Int2,
            gs_shader_param_type_GS_SHADER_PARAM_INT3 => ShaderParamType::Int3,
            gs_shader_param_type_GS_SHADER_PARAM_INT4 => ShaderParamType::Int4,
            gs_shader_param_type_GS_SHADER_PARAM_MATRIX4X4 => ShaderParamType::Mat4,
            gs_shader_param_type_GS_SHADER_PARAM_TEXTURE => ShaderParamType::Texture,
            _ => panic!("Invalid param_type!"),
        }
    }
}

pub struct GraphicsEffect {
    raw: *mut gs_effect_t,
}

impl GraphicsEffect {
    pub fn from_effect_string(value: ObsString, name: ObsString) -> Option<Self> {
        let raw = GraphicsGuard::with_enter(|| unsafe {
            gs_effect_create(value.as_ptr(), name.as_ptr(), std::ptr::null_mut())
        });
        if raw.is_null() {
            None
        } else {
            Some(Self { raw })
        }
    }

    pub fn get_effect_param_by_name<T: TryFrom<GraphicsEffectParam>>(
        &mut self,
        name: ObsString,
    ) -> Option<T> {
        unsafe {
            let pointer = gs_effect_get_param_by_name(self.raw, name.as_ptr());
            if !pointer.is_null() {
                T::try_from(GraphicsEffectParam::from_raw(pointer)).ok()
            } else {
                None
            }
        }
    }

    /// # Safety
    /// Returns a mutable pointer to an effect which if modified could cause UB.
    pub unsafe fn as_ptr(&self) -> *mut gs_effect_t {
        self.raw
    }
}

impl Drop for GraphicsEffect {
    fn drop(&mut self) {
        GraphicsGuard::with_enter(|| unsafe {
            gs_effect_destroy(self.raw);
        });
    }
}

pub enum GraphicsEffectParamConversionError {
    InvalidType,
}

pub struct GraphicsEffectParam {
    raw: *mut gs_eparam_t,
    name: String,
    shader_type: ShaderParamType,
}

impl GraphicsEffectParam {
    /// # Safety
    /// Creates a GraphicsEffectParam from a mutable reference. This data could be modified
    /// somewhere else so this is UB.
    pub unsafe fn from_raw(raw: *mut gs_eparam_t) -> Self {
        let mut info = gs_effect_param_info::default();
        gs_effect_get_param_info(raw, &mut info);

        let shader_type = ShaderParamType::from_raw(info.type_);
        let name = CString::from(CStr::from_ptr(info.name))
            .into_string()
            .unwrap_or(String::from("{unknown-param-name}"));

        Self {
            raw,
            shader_type,
            name,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

macro_rules! impl_graphics_effects {
    ($($t:ident)*) => {
        $(
            item! {
                pub struct [<GraphicsEffect $t Param>] {
                    effect: GraphicsEffectParam,
                }

                impl TryFrom<GraphicsEffectParam> for [<GraphicsEffect $t Param>] {
                    type Error = Error;

                    fn try_from(effect: GraphicsEffectParam) -> Result<Self> {
                        match effect.shader_type {
                            ShaderParamType::[<$t>] => Ok([<GraphicsEffect $t Param>] { effect }),
                            _ => Err(Error),
                        }
                    }
                }
            }
        )*
    };
}

impl_graphics_effects! {
    Vec2
    Texture
}

impl GraphicsEffectVec2Param {
    pub fn set_vec2(&mut self, _context: &GraphicsEffectContext, value: &Vec2) {
        unsafe {
            gs_effect_set_vec2(self.effect.raw, &value.raw);
        }
    }
}

impl GraphicsEffectTextureParam {
    pub fn set_next_sampler(
        &mut self,
        _context: &GraphicsEffectContext,
        value: &mut GraphicsSamplerState,
    ) {
        unsafe {
            gs_effect_set_next_sampler(self.effect.raw, value.raw);
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

    pub fn with_address_u(mut self, mode: GraphicsAddressMode) -> Self {
        self.info.address_u = mode.as_raw();
        self
    }

    pub fn with_address_v(mut self, mode: GraphicsAddressMode) -> Self {
        self.info.address_v = mode.as_raw();
        self
    }

    pub fn with_address_w(mut self, mode: GraphicsAddressMode) -> Self {
        self.info.address_w = mode.as_raw();
        self
    }

    pub fn with_filter(mut self, mode: GraphicsSampleFilter) -> Self {
        self.info.filter = mode.as_raw();
        self
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
        let raw = GraphicsGuard::with_enter(|| unsafe { gs_samplerstate_create(&info.info) });
        GraphicsSamplerState { raw }
    }
}

impl Drop for GraphicsSamplerState {
    fn drop(&mut self) {
        GraphicsGuard::with_enter(|| unsafe {
            gs_samplerstate_destroy(self.raw);
        });
    }
}

pub struct GraphicsEffectContext {}

impl GraphicsEffectContext {
    /// # Safety
    /// GraphicsEffectContext has methods that should only be used in certain situations.
    /// Constructing it at the wrong time could cause UB.
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

macro_rules! vector_impls {
    ($($rust_name: ident, $name:ident => $($component:ident)*,)*) => (
        $(
        #[derive(Clone)]
        pub struct $rust_name {
            raw: $name,
        }

        impl $rust_name {
            pub fn new($( $component: f32, )*) -> Self {
                let mut v = Self {
                    raw: $name::default(),
                };
                v.set($($component,)*);
                v
            }

            #[inline]
            pub fn zero(&mut self) {
                $(
                    self.raw.__bindgen_anon_1.__bindgen_anon_1.$component = 0.;
                )*
            }

            #[inline]
            pub fn copy(&mut self, input: &$rust_name) {
                self.set($(input.$component(),)*);
            }

            #[inline]
            pub fn add(&mut self, input: &$rust_name) {
                self.set($(self.$component() + input.$component(),)*);
            }

            #[inline]
            pub fn sub(&mut self, input: &$rust_name) {
                self.set($(self.$component() - input.$component(),)*);
            }

            #[inline]
            pub fn mul(&mut self, input: &$rust_name) {
                self.set($(self.$component() * input.$component(),)*);
            }

            #[inline]
            pub fn div(&mut self, input: &$rust_name) {
                self.set($(self.$component() / input.$component(),)*);
            }

            #[inline]
            pub fn addf(&mut self, input: f32) {
                self.set($(self.$component() + input,)*);
            }

            #[inline]
            pub fn subf(&mut self, input: f32) {
                self.set($(self.$component() - input,)*);
            }

            #[inline]
            pub fn mulf(&mut self, input: f32) {
                self.set($(self.$component() * input,)*);
            }

            #[inline]
            pub fn divf(&mut self, input: f32) {
                self.set($(self.$component() / input,)*);
            }

            #[inline]
            pub fn neg(&mut self) {
                self.set($(-self.$component(),)*);
            }

            #[inline]
            pub fn dot(&mut self, input: &$rust_name) -> f32 {
                $(
                    self.$component() * input.$component() +
                )* 0.
            }

            #[inline]
            pub fn len(&mut self) -> f32 {
                ($( self.$component() * self.$component() + )* 0.).sqrt()
            }

            #[inline]
            pub fn set(&mut self, $( $component: f32, )*) {
                $(
                    self.raw.__bindgen_anon_1.__bindgen_anon_1.$component = $component;
                )*
            }

            #[inline]
            pub fn min(&mut self, input: &$rust_name) {
                self.set($(self.$component().min(input.$component()),)*);
            }

            #[inline]
            pub fn max(&mut self, input: &$rust_name) {
                self.set($(self.$component().max(input.$component()),)*);
            }

            #[inline]
            pub fn minf(&mut self, input: f32) {
                self.set($(self.$component().min(input),)*);
            }

            #[inline]
            pub fn maxf(&mut self, input: f32) {
                self.set($(self.$component().max(input),)*);
            }

            #[inline]
            pub fn abs(&mut self) {
                self.set($(self.$component().abs(),)*);
            }

            #[inline]
            pub fn ceil(&mut self) {
                self.set($(self.$component().ceil(),)*);
            }

            #[inline]
            pub fn floor(&mut self) {
                self.set($(self.$component().floor(),)*);
            }

            #[inline]
            pub fn close(&mut self, input: &$rust_name, epsilon: f32) -> bool {
                $(
                    (self.$component() - input.$component()).abs() > epsilon &&
                )* true
            }

            $(
                item! {
                    #[inline]
                    pub fn [<$component>](&self) -> f32 {
                        unsafe {
                            self.raw.__bindgen_anon_1.__bindgen_anon_1.$component
                        }
                    }
                }
            )*

            pub unsafe fn as_ptr(&mut self) -> *mut $name {
                &mut self.raw
            }
        }

        impl Default for $rust_name {
            fn default() -> Self {
                $(
                    let $component = 0.;
                )*
                Self::new($( $component, )*)
            }
        }
        )*
    );
}

vector_impls! {
    Vec2, vec2 => x y,
    Vec3, vec3 => x y z,
    Vec4, vec4 => x y z w,
}

/// Wrapper around [`gs_texture_t`](https://obsproject.com/docs/reference-libobs-graphics-graphics.html#c.gs_texture_t)
pub struct GraphicsTexture {
    raw: *mut gs_texture_t,
}

impl GraphicsTexture {
    pub fn new(width: u32, height: u32, format: GraphicsColorFormat) -> Self {
        let raw = GraphicsGuard::with_enter(|| unsafe {
            gs_texture_create(width, height, format.as_raw(), 1, null_mut(), GS_DYNAMIC)
        });
        Self { raw }
    }

    #[inline]
    pub fn height(&self) -> u32 {
        GraphicsGuard::with_enter(|| unsafe { gs_texture_get_height(self.raw) })
    }

    #[inline]
    pub fn width(&self) -> u32 {
        GraphicsGuard::with_enter(|| unsafe { gs_texture_get_width(self.raw) })
    }

    pub fn set_image(&mut self, data: &[u8], linesize: u32, invert: bool) {
        GraphicsGuard::with_enter(|| unsafe {
            gs_texture_set_image(self.raw, data.as_ptr(), linesize, invert);
        });
    }

    pub fn draw(&self, x: c_int, y: c_int, cx: u32, cy: u32, flip: bool) {
        unsafe {
            obs_source_draw(self.raw, x, y, cx, cy, flip);
        }
    }

    #[inline]
    pub fn map(&mut self) -> Result<MappedTexture> {
        MappedTexture::new(self)
    }

    pub unsafe fn as_ptr(&self) -> *mut gs_texture_t {
        self.raw
    }
}

impl Drop for GraphicsTexture {
    fn drop(&mut self) {
        GraphicsGuard::with_enter(|| unsafe {
            gs_texture_destroy(self.raw);
        });
    }
}

/// Represents a mapped texture blob from [`GraphicsTexture`].
pub struct MappedTexture<'tex> {
    tex: &'tex mut GraphicsTexture,
    ptr: *mut u8,
    len: usize,
}

impl<'tex> MappedTexture<'tex> {
    fn new(tex: &'tex mut GraphicsTexture) -> Result<Self> {
        let mut ptr: *mut u8 = ptr::null_mut();
        let mut linesize = 0u32;
        let map_result = GraphicsGuard::with_enter(|| unsafe {
            gs_texture_map(tex.as_ptr(), &mut ptr, &mut linesize)
        });
        if !map_result {
            return Err(Error);
        }
        let len = (linesize * tex.height()) as usize;
        Ok(Self { tex, ptr, len })
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.tex.width()
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.tex.height()
    }
}

impl std::ops::Deref for MappedTexture<'_> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
    }
}

impl std::ops::DerefMut for MappedTexture<'_> {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
    }
}

impl std::fmt::Debug for MappedTexture<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&**self, f)
    }
}

impl Drop for MappedTexture<'_> {
    fn drop(&mut self) {
        GraphicsGuard::with_enter(|| unsafe {
            gs_texture_unmap(self.tex.as_ptr());
        });
    }
}
