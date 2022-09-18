use std::mem::forget;

pub trait PtrWrapper: Sized {
    type Pointer;
    /// Wraps the pointer into a **owned** wrapper.
    ///
    /// # Safety
    ///
    /// Pointer must be valid.
    unsafe fn from_raw(raw: *mut Self::Pointer) -> Self;

    /// Returns the inner pointer.
    fn as_ptr(&self) -> *const Self::Pointer;

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
