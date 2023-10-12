use std::{
    borrow::{Borrow, BorrowMut},
    fmt::{Debug, Formatter},
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::{ref_deallocators::RefDeallocators, Address};

/// Weak reference. Doesn't affect reference counting.
/// It is better to check with `freed()` method before use because it
/// might contain pointer to deallocated object.
pub struct Weak<T: ?Sized> {
    pub(crate) ptr: Option<NonNull<T>>,
}

unsafe impl<T: ?Sized> Send for Weak<T> {}
unsafe impl<T: ?Sized> Sync for Weak<T> {}

impl<T: ?Sized> Copy for Weak<T> {}

impl<T: ?Sized> Clone for Weak<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Weak<T> {
    pub const fn const_default() -> Self {
        Self { ptr: None }
    }

    pub fn from_ref(rf: &T) -> Self {
        let address = rf.address();
        assert!(
            RefDeallocators::exists(address),
            "Trying to get weak pointer for object which is not managed by reference counter."
        );
        let ptr = NonNull::new(rf as *const T as *mut T);
        assert!(ptr.is_some(), "Failed to get ptr from ref");
        Self { ptr }
    }

    pub fn addr(&self) -> usize {
        self.ptr.map(|p| p.as_ptr() as *const u8 as usize).unwrap_or_default()
    }

    pub fn is_null(&self) -> bool {
        self.ptr.is_none()
    }

    pub fn is_ok(&self) -> bool {
        RefDeallocators::exists(self.addr())
    }

    pub fn freed(&self) -> bool {
        self.ptr.is_some() && !RefDeallocators::exists(self.addr())
    }

    pub fn get(&mut self) -> Option<&mut T> {
        if self.is_ok() {
            self.deref_mut().into()
        } else {
            None
        }
    }

    #[cfg(feature = "checks")]
    fn check(&self, check_main: bool) {
        use log::error;

        if check_main && !crate::is_main_thread() {
            panic!(
                "Unsafe Weak pointer deref: {}. Thread is not Main. Thread id: {}",
                std::any::type_name::<T>(),
                crate::current_thread_id()
            );
        }

        if self.ptr.is_none() {
            error!(
                "Defererencing never initialized weak pointer: {}",
                std::any::type_name::<T>()
            );
            // backtrace();
            panic!(
                "Defererencing never initialized weak pointer: {}",
                std::any::type_name::<T>()
            );
        }

        if !RefDeallocators::exists(self.addr()) {
            error!(
                "Defererencing already freed weak pointer: {}",
                std::any::type_name::<T>()
            );
            // backtrace();
            panic!(
                "Defererencing already freed weak pointer: {}",
                std::any::type_name::<T>()
            );
        }
    }
}

impl<T> Weak<T> {
    /// # Safety
    ///
    /// Create `Weak` without `Own` and leak memory.
    /// Use only for test purposes.
    pub unsafe fn leak(val: T) -> Self {
        let val = Box::new(val);
        let address = val.deref().address();
        let ptr = Box::leak(val) as *mut T;

        if address == 1 {
            panic!("Closure? Empty type?");
        }

        RefDeallocators::add_deallocator(address, || {});

        Self {
            ptr: NonNull::new(ptr),
        }
    }
}

impl<T: ?Sized> Deref for Weak<T> {
    type Target = T;
    fn deref(&self) -> &T {
        #[cfg(feature = "checks")]
        self.check(false);
        unsafe { self.ptr.unwrap().as_ref() }
    }
}

impl<T: ?Sized> DerefMut for Weak<T> {
    fn deref_mut(&mut self) -> &mut T {
        #[cfg(feature = "checks")]
        self.check(true);
        unsafe { self.ptr.unwrap().as_mut() }
    }
}

impl<T: ?Sized> Borrow<T> for Weak<T> {
    fn borrow(&self) -> &T {
        self.deref()
    }
}

impl<T: ?Sized> BorrowMut<T> for Weak<T> {
    fn borrow_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}

impl<T: ?Sized> Default for Weak<T> {
    fn default() -> Self {
        Self { ptr: None }
    }
}

impl<T: ?Sized + Debug> Debug for Weak<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

// TODO: Coerce
// impl<T, U> CoerceUnsized<Weak<U>> for Weak<T>
//     where
//         T: Unsize<U> + ?Sized,
//         U: ?Sized,
// {
// }

#[cfg(test)]
mod test {
    use std::ops::Deref;

    use serial_test::serial;

    use crate::{set_current_thread_as_main, Own, ToWeak, Weak};

    #[derive(Default)]
    struct Sok {}

    #[test]
    #[serial]
    fn leak_weak() {
        set_current_thread_as_main();
        let leaked = unsafe { Weak::leak(5) };
        dbg!(leaked.deref());
    }

    #[test]
    #[serial]
    fn addr() {
        let own = Own::new(5);
        let weak = own.weak();
        assert_eq!(own.addr(), weak.addr());
    }
}
