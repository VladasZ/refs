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
    use std::{
        ops::{Deref, DerefMut},
        thread::spawn,
    };

    use serial_test::serial;

    use crate::{set_current_thread_as_main, Own, Weak};

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

    #[test]
    #[should_panic(expected = "Defererencing never initialized weak pointer: i32")]
    fn null_weak() {
        let default = Weak::<i32>::default();
        assert_eq!(default.is_ok(), false);
        let _ = default.deref();
    }

    static WEAK: Weak<bool> = Weak::const_default();

    #[test]
    #[serial]
    fn const_weak_default() {
        set_current_thread_as_main();
        assert!(WEAK.is_null());
    }

    #[test]
    #[should_panic]
    #[serial]
    fn deref_null() {
        set_current_thread_as_main();
        let null = Weak::<u32>::default();
        assert!(null.is_null());
        assert_eq!(null.is_ok(), false);
        dbg!(&null);
    }

    #[test]
    #[serial]
    #[should_panic]
    fn deref_async() {
        set_current_thread_as_main();
        let num = Own::new(5);
        let mut weak = num.weak();
        spawn(move || {
            assert_eq!(weak.deref(), &5);
            assert_eq!(weak.deref_mut(), &5);
        })
        .join()
        .unwrap();
    }

    #[test]
    #[serial]
    fn weak_misc() {
        set_current_thread_as_main();
        let five = Own::new(5);
        let ten = Own::new(10);

        assert_ne!(five, ten);

        let mut weak = five.weak();
        let another_weak = weak.clone();

        assert_eq!(weak.is_null(), false);
        assert_eq!(weak.deref(), another_weak.deref());

        let null = Weak::<i32>::default();

        assert!(null.is_null());
        assert_eq!(null.is_ok(), false);

        let five_ref = weak.get().unwrap();

        assert_eq!(five_ref, &5);

        *five_ref = 10;

        assert_eq!(weak.deref(), &10);
    }
}
