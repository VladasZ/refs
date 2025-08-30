use core::ptr::from_mut;
use std::{
    fmt::{Debug, Formatter},
    hash::{Hash, Hasher},
    intrinsics::transmute_unchecked,
    marker::Unsize,
    ops::{CoerceUnsized, Deref, DerefMut},
    ptr::{null, null_mut},
};

use crate::{Address, AsAny, Erased, Rglica, ToRglica, ref_counter::RefCounter, stamp, weak_from_ref};

const PTR_SIZE: usize = size_of::<usize>();

/// Weak reference. Doesn't affect reference counting.
pub struct Weak<T: ?Sized = Erased> {
    pub(crate) ptr:   *mut T,
    pub(crate) stamp: u64,
    pub type_name:    &'static str,
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
            ptr:       null_mut(),
            stamp:     0,
            type_name: std::any::type_name::<T>(),
        }
    }
}

impl<T: ?Sized> Weak<T> {
    pub fn sized(&self) -> bool {
        let ptr_size = size_of_val(&self.ptr);

        if ptr_size == PTR_SIZE {
            true
        } else if ptr_size == PTR_SIZE * 2 {
            false
        } else {
            unreachable!("Invalid ptr size: {ptr_size}")
        }
    }

    pub fn addr(&self) -> usize {
        if self.sized() {
            self.ptr as *const u8 as usize
        } else {
            // Return only data address
            let ptr_bytes: [usize; 2] = unsafe { transmute_unchecked(self.ptr) };
            ptr_bytes[0]
        }
    }

    pub fn raw(&self) -> (usize, u64, &'static str) {
        (self.addr(), self.stamp, self.type_name)
    }

    pub unsafe fn from_raw(ptr: usize, stamp: u64, type_name: &'static str) -> Self {
        let mut new = Weak::<T> {
            stamp,
            type_name,
            ..Default::default()
        };

        let ptr = if new.sized() {
            unsafe { transmute_unchecked(ptr) }
        } else {
            trait Trait {}
            struct Struct;
            impl Trait for Struct {}

            let sized: *const Struct = null();
            let un_sized: *const dyn Trait = sized;

            let mut ptr_bytes: [usize; 2] = unsafe { transmute_unchecked(un_sized) };
            ptr_bytes[0] = ptr;
            unsafe { transmute_unchecked(ptr_bytes) }
        };

        new.ptr = ptr;

        new
    }

    pub fn was_initialized(&self) -> bool {
        !self.ptr.is_null()
    }

    pub fn is_ok(&self) -> bool {
        if self.ptr.is_null() {
            return false;
        }
        let Some(stamp) = RefCounter::stamp_for_address(self.addr()) else {
            return false;
        };
        if stamp != self.stamp {
            return false;
        }
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

    /// # Safety
    /// Check state before usage
    pub unsafe fn deref_unchecked(&self) -> &T {
        unsafe { self.ptr.as_ref().unwrap_unchecked() }
    }

    /// # Safety
    /// Check state before usage
    pub unsafe fn deref_unchecked_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut().unwrap_unchecked() }
    }

    /// # Safety
    /// unsafe
    pub unsafe fn to_rglica(&self) -> Rglica<T> {
        self.deref().to_rglica()
    }

    #[cfg(feature = "checks")]
    fn check(&self, check_main: bool) {
        use log::error;

        assert!(
            !check_main || crate::is_main_thread(),
            "Unsafe Weak pointer deref: {}. Thread is not Main. Thread id: {}",
            self.type_name,
            crate::current_thread_id()
        );

        if self.ptr.is_null() {
            error!("Defererencing never initialized weak pointer: {}", self.type_name,);
            // backtrace();
            panic!("Defererencing never initialized weak pointer: {}", self.type_name,);
        }

        if self.is_null() {
            error!("Defererencing already freed weak pointer: {}", self.type_name,);
            // backtrace();
            panic!("Defererencing already freed weak pointer: {}", self.type_name,);
        }
    }

    pub fn erase(&self) -> Weak {
        Weak {
            ptr:       self.ptr.cast(),
            stamp:     self.stamp,
            type_name: self.type_name,
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

        assert_ne!(address, 1, "Invalid address. In cou be a closure or empty type.");

        let stamp = stamp();

        RefCounter::add(address, stamp);

        Self {
            ptr,
            stamp,
            type_name: std::any::type_name::<T>(),
        }
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
            ptr:       unsafe { transmute_unchecked(un_sized) },
            stamp:     0,
            type_name: "unsized null",
        }
    }
}

impl<T> Default for Weak<T> {
    fn default() -> Self {
        Self {
            ptr:       null_mut(),
            stamp:     0,
            type_name: std::any::type_name::<T>(),
        }
    }
}

impl<T> Eq for Weak<T> {}

impl<T> PartialEq<Self> for Weak<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr && self.stamp == other.stamp
    }
}

impl<T> Hash for Weak<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
        self.stamp.hash(state);
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
