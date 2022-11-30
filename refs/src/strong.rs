use crate::stats::adjust_stat;
use crate::{is_main_thread, thread_id, Weak};
use crate::{Address, RefCounters};
use crate::{ToWeak, TotalSize};
use log::trace;
use std::ptr::read;
use std::{
    alloc::{dealloc, Layout},
    marker::Unsize,
    ops::{CoerceUnsized, Deref, DerefMut},
    ptr::NonNull,
};

/// Strong reference. Takes part in reference counting.
/// When `Strong` ref counter reaches 0 object gets deallocated.
/// All `Weak` refs become invalid.
pub struct Strong<T: ?Sized> {
    name: String,
    address: usize,
    total_size: usize,
    ptr: *mut T,
}

unsafe impl<T: ?Sized> Send for Strong<T> {}
unsafe impl<T: ?Sized> Sync for Strong<T> {}

impl<T: Sized + 'static> Strong<T> {
    pub fn new(val: T) -> Self {
        let total_size = val.total_size();

        let name = std::any::type_name::<T>().to_string();

        adjust_stat::<T>(&name, 1, total_size);

        let val = Box::new(val);
        let address = val.deref().address();
        let ptr = Box::leak(val) as *mut T;

        trace!("New strong: {name}, addr: {address}, ptr: {:?}", ptr);

        if address == 1 {
            panic!("Closure?");
        }

        RefCounters::add_strong(address, move || unsafe {
            trace!(
                "Deallocating strong: {}, addr: {}, ptr: {:?}",
                std::any::type_name::<T>(),
                address,
                ptr
            );
            read(ptr);
            dealloc(ptr as *mut u8, Layout::new::<T>());
        });

        Self {
            name,
            address,
            total_size,
            ptr,
        }
    }
}

impl<T: ?Sized> Strong<T> {
    pub fn address(&self) -> usize {
        self.address
    }

    pub fn ref_count(&self) -> usize {
        RefCounters::strong_count(self.address)
    }

    fn check(&self) {
        if !is_main_thread() {
            panic!(
                "Unsafe Strong pointer deref: {}. Thread is not Main. Thread id: {}",
                std::any::type_name::<T>(),
                thread_id()
            );
        }
    }
}

impl<T: ?Sized> Deref for Strong<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.check();
        unsafe { self.ptr.as_ref().unwrap() }
    }
}

impl<T: ?Sized> DerefMut for Strong<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.check();
        unsafe { self.ptr.as_mut().unwrap() }
    }
}

impl<T: ?Sized> Clone for Strong<T> {
    fn clone(&self) -> Self {
        RefCounters::increase_strong(self.address);
        Self {
            name: self.name.clone(),
            address: self.address,
            total_size: self.total_size,
            ptr: self.ptr,
        }
    }
}

impl<T: ?Sized> Drop for Strong<T> {
    fn drop(&mut self) {
        RefCounters::decrease_strong(self.address);
        if RefCounters::strong_count(self.address) == 0 {
            adjust_stat::<T>(&self.name, -1, self.total_size);
            RefCounters::remove(self.address);
        }
    }
}

impl<T: ?Sized> ToWeak<T> for Strong<T> {
    fn weak(&self) -> Weak<T> {
        Weak {
            address: self.address,
            ptr: NonNull::new(self.ptr),
        }
    }
}

impl<T: Default + Sized + 'static> Default for Strong<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T, U> CoerceUnsized<Strong<U>> for Strong<T>
where
    T: Unsize<U> + ?Sized,
    U: ?Sized,
{
}
