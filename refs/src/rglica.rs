use log::error;
use rtools::address::Address;
use rtools::backtrace;
use rtools::bytes::data_pointer;
use std::{
    fmt::{Debug, Formatter},
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

/// Very unsafe. Basically just `C++` pointer. Do not use.
pub struct Rglica<T: ?Sized> {
    pub ptr: Option<NonNull<T>>,
}

unsafe impl<T: ?Sized> Send for Rglica<T> {}
unsafe impl<T: ?Sized> Sync for Rglica<T> {}

impl<T: ?Sized> Copy for Rglica<T> {}

impl<T: ?Sized> Clone for Rglica<T> {
    fn clone(&self) -> Rglica<T> {
        Self { ptr: self.ptr }
    }
}

impl<T: ?Sized> Rglica<T> {
    pub fn from_ref(rf: &T) -> Rglica<T> {
        let ptr = NonNull::new(rf as *const T as *mut T);
        debug_assert!(ptr.is_some(), "Failed to cast ref to Rglica");
        Self {
            ptr: ptr.unwrap().into(),
        }
    }

    pub fn is_null(&self) -> bool {
        self.ptr.is_none()
    }

    pub fn is_ok(&self) -> bool {
        self.ptr.is_some()
    }

    pub fn invalidate(&mut self) {
        self.ptr = None
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr.unwrap().as_ptr()
    }

    pub fn reset(&mut self) {
        self.ptr = Default::default()
    }

    pub fn get(&mut self) -> Option<&mut T> {
        if self.is_ok() {
            self.deref_mut().into()
        } else {
            None
        }
    }
}

impl<T: ?Sized> Deref for Rglica<T> {
    type Target = T;
    fn deref(&self) -> &T {
        if self.is_null() {
            error!("Null Rglica: {}", std::any::type_name::<T>());
            backtrace();
            panic!("Null Rglica: {}", std::any::type_name::<T>());
        }
        unsafe { self.ptr.unwrap().as_ref() }
    }
}

impl<T: ?Sized> DerefMut for Rglica<T> {
    fn deref_mut(&mut self) -> &mut T {
        if self.is_null() {
            error!("Null Rglica: {}", std::any::type_name::<T>());
            backtrace();
            panic!("Null Rglica: {}", std::any::type_name::<T>());
        }
        unsafe { self.ptr.unwrap().as_mut() }
    }
}

impl<T: ?Sized> Default for Rglica<T> {
    fn default() -> Rglica<T> {
        Self { ptr: None }
    }
}

impl<T: ?Sized> Rglica<T> {
    pub const fn const_default() -> Self {
        Self { ptr: None }
    }
}

impl<T: ?Sized> Debug for Rglica<T> {
    default fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.ptr.fmt(f)
    }
}

impl<T: ?Sized + Debug> Debug for Rglica<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_null() {
            return self.ptr.fmt(f);
        }
        self.deref().fmt(f)
    }
}

impl<T: ?Sized> Address for Rglica<T> {
    fn address(&self) -> usize {
        data_pointer(self.ptr)
    }
}
