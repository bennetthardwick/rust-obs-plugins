use std::mem::forget;

pub trait PtrWrapper: Sized {
    type Pointer;

    unsafe fn get_ref(ptr: *mut Self::Pointer) -> *mut Self::Pointer;
    unsafe fn release(ptr: *mut Self::Pointer);

    /// Wraps the pointer into a **owned** wrapper.
    ///
    /// # Safety
    ///
    /// Pointer must be valid.
    fn from_raw(raw: *mut Self::Pointer) -> Option<Self> {
        unsafe { Self::from_raw_unchecked(Self::get_ref(raw)) }
    }

    /// Wraps a **owned** pointer into wrapper.
    ///
    /// # Safety
    ///
    /// You have to make sure you owned the pointer
    unsafe fn from_raw_unchecked(raw: *mut Self::Pointer) -> Option<Self>;

    /// Returns the inner pointer.
    unsafe fn as_ptr(&self) -> *const Self::Pointer;

    /// Consumes the wrapper and transfers ownershop to the pointer
    ///
    /// This does **NOT** drop the wrapper internally.
    fn into_raw(self) -> *mut Self::Pointer {
        let raw = unsafe { self.as_ptr_mut() };
        forget(self);
        raw
    }

    /// Returns the inner pointer (mutable version).
    unsafe fn as_ptr_mut(&self) -> *mut Self::Pointer {
        self.as_ptr() as *mut _
    }
}

macro_rules! impl_ptr_wrapper {
    ($ref:ident, $ptr:ty, $get_ref:expr, $release:expr) => {
        impl PtrWrapper for $ref {
            type Pointer = $ptr;

            unsafe fn get_ref(ptr: *mut Self::Pointer) -> *mut Self::Pointer {
                unsafe { $get_ref(ptr) }
            }

            unsafe fn release(ptr: *mut Self::Pointer) {
                unsafe { $release(ptr) }
            }

            unsafe fn from_raw_unchecked(raw: *mut Self::Pointer) -> Option<Self> {
                if raw.is_null() {
                    None
                } else {
                    Some(Self { inner: raw })
                }
            }

            unsafe fn as_ptr(&self) -> *const Self::Pointer {
                self.inner
            }
        }

        impl Clone for $ref {
            fn clone(&self) -> Self {
                Self::from_raw(self.inner).expect("clone")
            }
        }

        impl Drop for $ref {
            fn drop(&mut self) {
                unsafe { Self::release(self.inner) }
            }
        }
    };
}
