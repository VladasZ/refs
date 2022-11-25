use crate::stats::adjust_stat;
use crate::ToWeak;
use crate::Weak;
use crate::{is_main_thread, thread_id, RefCounters};
use log::trace;
use rtools::address::Address;
use std::fmt::{Debug, Formatter};
use std::{
    alloc::{dealloc, Layout},
    marker::Unsize,
    ops::{CoerceUnsized, Deref, DerefMut},
    ptr::NonNull,
};

/// Similar to `Strong` but for unique ownership.
pub struct Own<T: ?Sized> {
    address: usize,
    ptr: *mut T,
}

impl<T: Sized + 'static> Own<T> {
    pub fn new(val: T) -> Self {
        let val = Box::new(val);
        let address = val.deref().address();
        let ptr = Box::leak(val) as *mut T;

        trace!(
            "New unique: {}, addr: {}, ptr: {:?}",
            std::any::type_name::<T>(),
            address,
            ptr
        );

        if address == 1 {
            panic!("Closure?");
        }

        adjust_stat::<T>(1);

        RefCounters::add_strong(address, move || unsafe {
            trace!(
                "Deallocating unique: {}, addr: {}, ptr: {:?}",
                std::any::type_name::<T>(),
                address,
                ptr
            );
            dealloc(ptr as *mut u8, Layout::new::<T>());
        });

        Self { address, ptr }
    }
}

impl<T: ?Sized> Own<T> {
    pub fn address(&self) -> usize {
        self.address
    }

    fn check(&self) {
        if !is_main_thread() {
            panic!(
                "Unsafe Own pointer deref: {}. Thread is not Main. Thread id: {}",
                std::any::type_name::<T>(),
                thread_id()
            );
        }
    }
}

impl<T: ?Sized> Own<T> {
    pub fn addr(&self) -> usize {
        self.address
    }
}

impl<T: ?Sized> Drop for Own<T> {
    fn drop(&mut self) {
        adjust_stat::<T>(-1);
        RefCounters::remove(self.address);
    }
}

impl<T: ?Sized> Deref for Own<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.check();
        unsafe { self.ptr.as_ref().unwrap() }
    }
}

impl<T: ?Sized> DerefMut for Own<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.check();
        unsafe { self.ptr.as_mut().unwrap() }
    }
}

impl<T: ?Sized> ToWeak<T> for Own<T> {
    fn weak(&self) -> Weak<T> {
        Weak {
            address: self.address,
            ptr: NonNull::new(self.ptr),
        }
    }
}

impl<T: Default + Sized + 'static> Default for Own<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Debug> Debug for Own<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

impl<T, U> CoerceUnsized<Own<U>> for Own<T>
where
    T: Unsize<U> + ?Sized,
    U: ?Sized,
{
}
