use core::ptr::from_ref;
use std::{
    any::type_name,
    fmt::{Debug, Formatter},
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use log::error;

use crate::Address;

/// Very unsafe. Basically just `C++` pointer. Do not use.
pub struct Rglica<T: ?Sized> {
    pub ptr: Option<NonNull<T>>,
}

unsafe impl<T: ?Sized> Send for Rglica<T> {}
unsafe impl<T: ?Sized> Sync for Rglica<T> {}

impl<T: ?Sized> Copy for Rglica<T> {}

impl<T: ?Sized> Clone for Rglica<T> {
    fn clone(&self) -> Rglica<T> {
        *self
    }
}

impl<T: ?Sized> Rglica<T> {
    pub const fn const_default() -> Self {
        Self { ptr: None }
    }

    pub fn from_ref(rf: &T) -> Rglica<T> {
        let ptr = NonNull::new((from_ref::<T>(rf)).cast_mut());
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

    pub fn as_ptr(&self) -> *mut T {
        self.ptr.unwrap().as_ptr()
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
            // backtrace();
            panic!("Null Rglica: {}", std::any::type_name::<T>());
        }
        unsafe { self.ptr.unwrap().as_ref() }
    }
}

impl<T: ?Sized> DerefMut for Rglica<T> {
    fn deref_mut(&mut self) -> &mut T {
        if self.is_null() {
            error!("Null Rglica: {}", std::any::type_name::<T>());
            // backtrace();
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

impl<T: ?Sized> Debug for Rglica<T> {
    default fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        type_name::<T>().fmt(f)
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
        self.ptr.map_or(0, |ptr| ptr.as_ptr() as *const u8 as usize)
    }
}

#[cfg(test)]
mod test {
    use std::ops::{Deref, DerefMut};

    use crate::{Address, Rglica};

    struct NoDebug;

    #[test]
    #[should_panic(expected = "Null Rglica: i32")]
    fn null_rglica() {
        let null = Rglica::<i32>::default();
        assert!(null.is_null());
        assert_eq!(null.is_ok(), false);
        _ = null.deref();
    }

    #[test]
    #[should_panic(expected = "Null Rglica: i32")]
    fn null_rglica_mut() {
        let mut null = Rglica::<i32>::default();
        assert!(null.is_null());
        assert_eq!(null.is_ok(), false);
        _ = null.deref_mut();
    }

    #[test]
    fn rglica_misc() {
        let five = 5;

        let five = &five;

        let mut val = Rglica::from_ref(five);
        assert_eq!(val.address(), five as *const i32 as usize);

        assert_eq!(val.is_null(), false);
        assert_eq!(val.is_ok(), true);

        assert_eq!("5", &format!("{val:?}"));

        let cloned = val.clone();

        assert_eq!(val.deref(), cloned.deref());

        let get = val.get().unwrap();

        *get = 10;

        assert_eq!(*val.deref(), 10);

        assert_eq!(
            "\"refs::rglica::test::NoDebug\"",
            format!("{:?}", Rglica::from_ref(&NoDebug))
        );

        assert_eq!("None", format!("{:?}", Rglica::<i32>::default()));

        assert_eq!(None, Rglica::<i32>::default().get());
    }
}
