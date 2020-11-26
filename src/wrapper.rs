use std::mem::forget;

pub trait PtrWrapper: Sized {
    type Pointer;
    /// Wraps the pointer into a **owned** wrapper.
    unsafe fn from_raw(raw: *mut Self::Pointer) -> Self;

    /// Returns the inner pointer.
    fn as_ptr(&self) -> *const Self::Pointer;

    /// Wraps the pointer into a **borrowed** wrapper.
    unsafe fn from_ptr_mut<'a>(raw: *mut Self::Pointer) -> &'a mut Self {
        let mut s = Self::from_raw(raw);
        let r = &mut s as *mut Self;
        forget(s);
        r.as_mut().unwrap()
    }

    /// Wraps the pointer into a **borrowed** wrapper.
    unsafe fn from_ptr<'a>(raw: *const Self::Pointer) -> &'a Self {
        let s = Self::from_raw(raw as *mut _);
        let r = &s as *const Self;
        forget(s);
        r.as_ref().unwrap()
    }

    /// Consumes the wrapper and transfers ownershop to the pointer
    ///
    /// This does **NOT** drop the wrapper internally.
    fn into_raw(mut self) -> *mut Self::Pointer {
        let raw = self.as_ptr_mut();
        forget(self);
        raw
    }

    /// Returns the inner pointer (mutable version).
    fn as_ptr_mut(&mut self) -> *mut Self::Pointer {
        self.as_ptr() as *mut _
    }
}
