use obs_sys::{
    obs_display_add_draw_callback, obs_display_destroy, obs_display_enabled,
    obs_display_remove_draw_callback, obs_display_resize, obs_display_t, obs_render_main_texture,
};

pub struct DisplayRef {
    inner: *mut obs_display_t,
}

impl_ptr_wrapper!(@ptr: inner, DisplayRef, obs_display_t, @identity, obs_display_destroy);

impl DisplayRef {
    pub fn enabled(&self) -> bool {
        unsafe { obs_display_enabled(self.inner) }
    }

    pub fn set_size(&self, cx: u32, cy: u32) {
        unsafe { obs_display_resize(self.inner, cx, cy) }
    }

    pub fn add_draw_callback<'a, S: DrawCallback>(&'a self, callback: S) -> DrawCallbackId<'a, S> {
        let data = Box::into_raw(Box::new(callback));
        let callback = draw_callback::<S>;
        unsafe {
            obs_display_add_draw_callback(self.inner, Some(callback), data as *mut std::ffi::c_void)
        }
        DrawCallbackId::new(data, callback as *const _, self.inner)
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
    pub fn new(
        data: *mut S,
        callback: *const std::ffi::c_void,
        display: *mut obs_display_t,
    ) -> Self {
        DrawCallbackId {
            data,
            callback,
            display,
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
