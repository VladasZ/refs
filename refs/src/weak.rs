use std::{
    borrow::{Borrow, BorrowMut},
    fmt::{Debug, Formatter},
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use log::error;

use crate::{is_main_thread, thread_id, Address, RefCounters};

/// Weak reference. Doesn't affect reference counting.
/// It is better to check with `freed()` method before use because it
/// might contain pointer to deallocated object.
pub struct Weak<T: ?Sized> {
    pub(crate) address: usize,
    pub(crate) ptr:     Option<NonNull<T>>,
}

unsafe impl<T: ?Sized> Send for Weak<T> {}
unsafe impl<T: ?Sized> Sync for Weak<T> {}

impl<T: ?Sized> Copy for Weak<T> {}

impl<T: ?Sized> Clone for Weak<T> {
    fn clone(&self) -> Self {
        Self {
            address: self.address,
            ptr:     self.ptr,
        }
    }
}

impl<T: ?Sized> Weak<T> {
    pub const fn const_default() -> Self {
        Self {
            address: 0,
            ptr:     None,
        }
    }

    pub fn from_ref(rf: &T) -> Self {
        let address = rf.address();
        assert!(
            RefCounters::exists(address),
            "Trying to get weak pointer for object which is not managed by reference counter."
        );
        let ptr = NonNull::new(rf as *const T as *mut T);
        assert!(ptr.is_some(), "Failed to get ptr from ref");
        Self { address, ptr }
    }

    pub fn addr(&self) -> usize {
        self.address
    }

    pub fn is_null(&self) -> bool {
        self.address == 0
    }

    pub fn is_ok(&self) -> bool {
        RefCounters::exists(self.address)
    }

    pub fn freed(&self) -> bool {
        self.ptr.is_some() && !RefCounters::exists(self.address)
    }

    pub fn get(&mut self) -> Option<&mut T> {
        if self.is_ok() {
            self.deref_mut().into()
        } else {
            None
        }
    }

    fn check(&self, check_main: bool) {
        if !is_main_thread() && check_main {
            panic!(
                "Unsafe Weak pointer deref: {}. Thread is not Main. Thread id: {}",
                std::any::type_name::<T>(),
                thread_id()
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

        if !RefCounters::exists(self.address) {
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

        RefCounters::add_strong(address, || {});

        Self {
            address,
            ptr: NonNull::new(ptr),
        }
    }
}

impl<T: ?Sized> Deref for Weak<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.check(false);
        unsafe { self.ptr.unwrap().as_ref() }
    }
}

impl<T: ?Sized> DerefMut for Weak<T> {
    fn deref_mut(&mut self) -> &mut T {
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
        Self {
            address: 0,
            ptr:     None,
        }
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

    use crate::{set_current_thread_as_main, Strong, ToWeak, Weak};

    #[derive(Default)]
    struct Sok {
        data: bool,
    }

    impl Sok {
        fn return_true(self: Weak<Self>) -> bool {
            !self.data
        }
    }

    #[test]
    #[serial]
    fn strong_to_weak() {
        set_current_thread_as_main();
        let strong: Strong<Sok> = Strong::new(Sok::default());
        assert!(strong.weak().return_true());
    }

    #[test]
    #[serial]
    fn leak_weak() {
        set_current_thread_as_main();
        let leaked = unsafe { Weak::leak(5) };
        dbg!(leaked.deref());
    }
}
