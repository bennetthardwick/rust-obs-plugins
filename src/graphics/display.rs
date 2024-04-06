use obs_sys::{
    obs_display_add_draw_callback, obs_display_destroy, obs_display_enabled,
    obs_display_remove_draw_callback, obs_display_resize, obs_display_set_background_color,
    obs_display_set_enabled, obs_display_size, obs_display_t, obs_render_main_texture,
};

use super::GraphicsColorFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

fn srgb_nonlinear_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
fn srgb_linear_to_nonlinear(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}
impl Color {
    pub const BLACK: Color = Color::new(0, 0, 0, 255);
    pub const WHITE: Color = Color::new(255, 255, 255, 255);
    pub const RED: Color = Color::new(255, 0, 0, 255);
    pub const GREEN: Color = Color::new(0, 255, 0, 255);
    pub const BLUE: Color = Color::new(0, 0, 255, 255);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    pub fn as_format(self, format: GraphicsColorFormat) -> u32 {
        match format {
            GraphicsColorFormat::RGBA => self.as_rgba(),
            GraphicsColorFormat::BGRA => self.as_bgra(),
            _ => unimplemented!("unsupported color format"),
        }
    }

    pub fn as_rgba(self) -> u32 {
        u32::from_ne_bytes([self.r, self.g, self.b, self.a])
    }

    pub fn as_bgra(self) -> u32 {
        u32::from_ne_bytes([self.b, self.g, self.r, self.a])
    }

    /// gs_float3_srgb_nonlinear_to_linear
    pub fn srgb_nonlinear_to_linear(self) -> Self {
        let r = srgb_nonlinear_to_linear(self.r as f32 / 255.0);
        let g = srgb_nonlinear_to_linear(self.g as f32 / 255.0);
        let b = srgb_nonlinear_to_linear(self.b as f32 / 255.0);
        Color {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
            a: self.a,
        }
    }

    pub fn srgb_linear_to_nonlinear(self) -> Self {
        let r = srgb_linear_to_nonlinear(self.r as f32 / 255.0);
        let g = srgb_linear_to_nonlinear(self.g as f32 / 255.0);
        let b = srgb_linear_to_nonlinear(self.b as f32 / 255.0);
        Color {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
            a: self.a,
        }
    }
}

/// A reference to a display, inner pointer is not managed by reference count.
/// So no `Clone` is implemented. you might want to use `Arc<DisplayRef>` if you need to clone it.
pub struct DisplayRef {
    inner: *mut obs_display_t,
}

impl_ptr_wrapper!(@ptr: inner, DisplayRef, obs_display_t, @identity, obs_display_destroy);

impl DisplayRef {
    pub fn enabled(&self) -> bool {
        unsafe { obs_display_enabled(self.inner) }
    }

    pub fn set_enabled(&self, enabled: bool) {
        unsafe { obs_display_set_enabled(self.inner, enabled) }
    }

    pub fn size(&self) -> (u32, u32) {
        let mut cx = 0;
        let mut cy = 0;
        unsafe { obs_display_size(self.inner, &mut cx, &mut cy) }
        (cx, cy)
    }

    pub fn set_size(&self, cx: u32, cy: u32) {
        unsafe { obs_display_resize(self.inner, cx, cy) }
    }

    pub fn set_background_color(&self, color: Color) {
        unsafe { obs_display_set_background_color(self.inner, color.as_rgba()) }
    }

    pub fn add_draw_callback<S: DrawCallback>(&self, callback: S) -> DrawCallbackId<'_, S> {
        let data = Box::into_raw(Box::new(callback));
        // force the pointer to be a function pointer, since it is not garantueed to be the same pointer
        // for different instance of generic functions
        // see https://users.rust-lang.org/t/generic-functions-and-their-pointer-uniquness/36989
        // clippy: #[deny(clippy::fn_address_comparisons)]
        let callback: unsafe extern "C" fn(*mut std::ffi::c_void, u32, u32) = draw_callback::<S>;
        unsafe {
            obs_display_add_draw_callback(self.inner, Some(callback), data as *mut std::ffi::c_void)
        }
        DrawCallbackId::new(data, callback as *const _, self)
    }

    pub fn remove_draw_callback<S: DrawCallback>(&self, data: DrawCallbackId<S>) -> S {
        data.take(self)
    }
}

pub struct RenderMainTexture;

impl DrawCallback for RenderMainTexture {
    fn draw(&self, _cx: u32, _cy: u32) {
        unsafe { obs_render_main_texture() }
    }
}

pub trait DrawCallback {
    fn draw(&self, cx: u32, cy: u32);
}

/// # Safety
/// This function is called by OBS, and it is guaranteed that the pointer is valid.
pub unsafe extern "C" fn draw_callback<S: DrawCallback>(
    data: *mut std::ffi::c_void,
    cx: u32,
    cy: u32,
) {
    let callback = &*(data as *const S);
    callback.draw(cx, cy);
}

pub struct DrawCallbackId<'a, S> {
    data: *mut S,
    callback: *const std::ffi::c_void,
    display: *mut obs_display_t,
    _marker: std::marker::PhantomData<&'a S>,
}

impl<'a, S> DrawCallbackId<'a, S> {
    pub fn new(data: *mut S, callback: *const std::ffi::c_void, display: &'a DisplayRef) -> Self {
        DrawCallbackId {
            data,
            callback,
            display: display.inner,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn take(self, display: &DisplayRef) -> S {
        assert_eq!(self.display, display.inner);
        let ptr = self.data;
        unsafe {
            obs_display_add_draw_callback(
                self.display,
                Some(std::mem::transmute(self.callback)),
                ptr as *mut std::ffi::c_void,
            )
        }
        std::mem::forget(self);
        unsafe { *Box::from_raw(ptr) }
    }

    /// Forget the callback and keep it alive forever.
    /// As long as the underlying display is alive, the callback will be called.
    /// If the display is destroyed, the callback will be also dropped.
    pub fn forever(self) {
        std::mem::forget(self);
    }
}

impl<'a, S> Drop for DrawCallbackId<'a, S> {
    fn drop(&mut self) {
        unsafe {
            // we don't check validity of the display here
            obs_display_remove_draw_callback(
                self.display,
                Some(std::mem::transmute(self.callback)),
                self.data as *mut std::ffi::c_void,
            );
            drop(Box::from_raw(self.data));
        }
    }
}
