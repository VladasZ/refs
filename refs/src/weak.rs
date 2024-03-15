use core::ptr::from_mut;
use std::{
    fmt::{Debug, Formatter},
    intrinsics::transmute_unchecked,
    marker::Unsize,
    ops::{CoerceUnsized, Deref, DerefMut},
    ptr::{null, null_mut},
};

use crate::{ref_deallocators::RefDeallocators, stamp, weak_from_ref, Address, AsAny};

/// Weak reference. Doesn't affect reference counting.
/// It is better to check with `freed()` method before use because it
/// might contain pointer to deallocated object.
pub struct Weak<T: ?Sized> {
    pub(crate) ptr:   *mut T,
    pub(crate) stamp: u64,
}

unsafe impl<T: ?Sized> Send for Weak<T> {}
unsafe impl<T: ?Sized> Sync for Weak<T> {}

impl<T: ?Sized> Copy for Weak<T> {}

impl<T: ?Sized> Clone for Weak<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Weak<T> {
    pub const fn const_default() -> Self {
        Self {
            ptr:   null_mut(),
            stamp: 0,
        }
    }
}

impl<T: ?Sized> Weak<T> {
    pub fn addr(&self) -> usize {
        self.ptr as *const u8 as usize
    }

    pub fn is_ok(&self) -> bool {
        let Some(stamp) = RefDeallocators::stamp_for_address(self.addr()) else {
            return false;
        };
        if stamp != self.stamp {
            return false;
        };
        true
    }

    pub fn is_null(&self) -> bool {
        !self.is_ok()
    }

    pub fn get(&self) -> Option<&T> {
        if self.is_ok() {
            unsafe { self.deref_unchecked().into() }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.is_ok() {
            unsafe { self.deref_unchecked_mut().into() }
        } else {
            None
        }
    }

    unsafe fn deref_unchecked(&self) -> &T {
        unsafe { self.ptr.as_ref().unwrap_unchecked() }
    }

    unsafe fn deref_unchecked_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut().unwrap_unchecked() }
    }

    #[cfg(feature = "checks")]
    fn check(&self, check_main: bool) {
        use log::error;

        assert!(
            !check_main || crate::is_main_thread(),
            "Unsafe Weak pointer deref: {}. Thread is not Main. Thread id: {}",
            std::any::type_name::<T>(),
            crate::current_thread_id()
        );

        if self.ptr.is_null() {
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

        if self.is_null() {
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
        let ptr = from_mut::<T>(Box::leak(val));

        assert_ne!(address, 1, "Closure? Empty type?");

        let stamp = stamp();

        RefDeallocators::add_deallocator(address, stamp, || {});

        Self { ptr, stamp }
    }
}

impl<T: ?Sized + AsAny> Weak<T> {
    pub fn downcast<U: 'static>(&self) -> Option<Weak<U>> {
        let rf = self.as_any().downcast_ref::<U>()?;
        Some(weak_from_ref(rf))
    }
}

impl<T: ?Sized> Deref for Weak<T> {
    type Target = T;
    fn deref(&self) -> &T {
        #[cfg(feature = "checks")]
        self.check(false);
        unsafe { self.deref_unchecked() }
    }
}

impl<T: ?Sized> DerefMut for Weak<T> {
    fn deref_mut(&mut self) -> &mut T {
        #[cfg(feature = "checks")]
        self.check(true);
        unsafe { self.deref_unchecked_mut() }
    }
}

impl<T: ?Sized> Default for Weak<T> {
    default fn default() -> Self {
        trait Trait {}
        struct Struct;
        impl Trait for Struct {}

        let sized: *const Struct = null();
        let un_sized: *const dyn Trait = sized;

        Self {
            ptr:   unsafe { transmute_unchecked(un_sized) },
            stamp: 0,
        }
    }
}

impl<T> Default for Weak<T> {
    fn default() -> Self {
        Self {
            ptr:   null_mut(),
            stamp: 0,
        }
    }
}

impl<T: ?Sized + Debug> Debug for Weak<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

impl<T, U> CoerceUnsized<Weak<U>> for Weak<T>
where
    T: Unsize<U> + ?Sized,
    U: ?Sized,
{
}

#[cfg(test)]
mod test {
    use std::{
        any::Any,
        ops::{Deref, DerefMut},
        thread::spawn,
    };

    use serial_test::serial;

    use crate::{set_current_thread_as_main, AsAny, Own, Weak};

    #[test]
    #[serial]
    fn leak_weak() {
        set_current_thread_as_main();
        let leaked = unsafe { Weak::leak(5) };
        dbg!(leaked.deref());
    }

    #[test]
    #[should_panic(expected = "Closure? Empty type?")]
    fn leak_weak_closure() {
        let _leaked = unsafe { Weak::leak(|| {}) };
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
        assert_eq!(null.get(), None);

        let five_ref = weak.get_mut().unwrap();

        assert_eq!(five_ref, &5);

        *five_ref = 10;

        assert_eq!(weak.deref(), &10);
    }

    #[test]
    fn default_weak() {
        let weak = Weak::<i32>::default();
        assert!(weak.is_null());

        trait Trait {
            fn _a(&self);
        }
        let weak = Weak::<dyn Trait>::default();
        assert!(weak.is_null());
    }

    #[test]
    fn downcast_weak() {
        trait Tr: AsAny {}
        struct St {
            _a: i32,
        }

        impl Tr for St {}
        impl AsAny for St {
            fn as_any(&self) -> &dyn Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
        }

        let own: Own<dyn Tr> = Own::new(St { _a: 50 });
        let downcasted: Weak<St> = own.downcast().unwrap();

        assert_eq!(downcasted._a, 50);
    }
}
