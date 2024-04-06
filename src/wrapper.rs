use std::mem::forget;

pub trait PtrWrapperInternal: PtrWrapper {
    /// # Safety
    ///
    /// This function should not be called directly, use `from_raw` and
    /// `from_raw_unchecked` instead.
    unsafe fn new_internal(ptr: *mut Self::Pointer) -> Self;
    /// # Safety
    ///
    /// This function should not be called directly, use `from_raw` and
    /// `from_raw_unchecked` instead.
    unsafe fn get_internal(&self) -> *mut Self::Pointer;
}

pub trait PtrWrapper: Sized {
    type Pointer;

    /// # Safety
    ///
    /// This function called extern C api, and should not be called directly.
    unsafe fn get_ref(ptr: *mut Self::Pointer) -> *mut Self::Pointer;

    /// # Safety
    ///
    /// This function called extern C api, and should not be called directly.
    unsafe fn release(ptr: *mut Self::Pointer);

    /// Wraps the pointer into a **owned** wrapper.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
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
    ///
    /// # Safety
    ///
    /// This function would return a pointer not managed, should only called
    /// when interacting with extern C api.
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
    ///
    /// # Safety
    ///
    /// This function would return a pointer not managed, should only called
    /// when interacting with extern C api.
    unsafe fn as_ptr_mut(&self) -> *mut Self::Pointer {
        self.as_ptr() as *mut _
    }
}

macro_rules! impl_ptr_wrapper {
    (@ptr: $field:ident, $ref:ty, $($tt:tt)*) => {
        impl_ptr_wrapper!(@__impl.trait.internal $ref, $field);
        impl_ptr_wrapper!($ref, $($tt)*);
    };
    ($ref:ty, $ptr:ty, @identity, $release:expr) => {
        impl_ptr_wrapper!(@__impl.trait.wrapper $ref, $ptr, impl_ptr_wrapper!{@__impl.fn.id}, $release);
        // when get_ref is `@identity`, no `Clone implemented`
        impl_ptr_wrapper!(@__impl.trait.drop $ref, $ptr);
    };
    ($ref:ty, $ptr:ty, $get_ref:expr, $release:expr) => {
        impl_ptr_wrapper!(@__impl.trait.wrapper $ref, $ptr, impl_ptr_wrapper!{@__impl.fn.get_ref $get_ref}, $release);
        impl_ptr_wrapper!(@__impl.trait.clone $ref, $ptr);
        impl_ptr_wrapper!(@__impl.trait.drop $ref, $ptr);
    };

    ($ref:ty, $ptr:ty, @addref: $add_ref:expr, $release:expr) => {
        impl_ptr_wrapper!(@__impl.trait.wrapper $ref, $ptr, impl_ptr_wrapper!{@__impl.fn.add_ref $add_ref}, $release);
        impl_ptr_wrapper!(@__impl.trait.clone $ref, $ptr);
        impl_ptr_wrapper!(@__impl.trait.drop $ref, $ptr);
    };
    (@__impl.fn.get_ref $get_ref:expr) => {
        unsafe fn get_ref(ptr: *mut Self::Pointer) -> *mut Self::Pointer {
            unsafe { $get_ref(ptr) }
        }
    };
    (@__impl.fn.add_ref $add_ref:expr) => {
        unsafe fn get_ref(ptr: *mut Self::Pointer) -> *mut Self::Pointer {
            unsafe { $add_ref(ptr); ptr }
        }
    };
    (@__impl.fn.id) => {
        unsafe fn get_ref(ptr: *mut Self::Pointer) -> *mut Self::Pointer {
            ptr
        }
    };
    (@__impl.trait.internal $ref:ty, $field:ident) => {
        impl $crate::wrapper::PtrWrapperInternal for $ref {
            unsafe fn new_internal(ptr: *mut Self::Pointer) -> Self {
                Self { $field: ptr }
            }
            unsafe fn get_internal(&self) -> *mut Self::Pointer {
                self.$field
            }
        }
    };
    (@__impl.trait.wrapper $ref:ty, $ptr:ty, $get_ref:item, $release:expr) => {
        impl PtrWrapper for $ref {
            type Pointer = $ptr;

            unsafe fn from_raw_unchecked(raw: *mut Self::Pointer) -> Option<Self> {
                use $crate::wrapper::PtrWrapperInternal;
                if raw.is_null() {
                    None
                } else {
                    Some(Self::new_internal(raw))
                }
            }

            $get_ref

            unsafe fn release(ptr: *mut Self::Pointer) {
                unsafe { $release(ptr) }
            }

            unsafe fn as_ptr(&self) -> *const Self::Pointer {
                use $crate::wrapper::PtrWrapperInternal;
                self.get_internal()
            }
        }
    };
    (@__impl.trait.clone $ref:ty, $ptr:ty) => {
        impl Clone for $ref {
            fn clone(&self) -> Self {
                Self::from_raw(unsafe { self.as_ptr_mut() }).expect("clone")
            }
        }
    };
    (@__impl.trait.drop $ref:ty, $ptr:ty) => {
        impl Drop for $ref {
            fn drop(&mut self) {
                unsafe { Self::release(self.as_ptr_mut()) }
            }
        }
    };
}
