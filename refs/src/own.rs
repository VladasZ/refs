use std::{
    alloc::{dealloc, Layout},
    borrow::{Borrow, BorrowMut},
    fmt::{Debug, Formatter},
    marker::Unsize,
    ops::{CoerceUnsized, Deref, DerefMut},
    ptr::{read, NonNull},
};

use crate::{
    current_thread_id, is_main_thread, ref_deallocators::RefDeallocators, stats::adjust_stat, Address,
    ToWeak, TotalSize, Weak,
};

pub struct Own<T: ?Sized> {
    name:       String,
    address:    usize,
    total_size: usize,
    ptr:        *mut T,
}

unsafe impl<T: ?Sized> Send for Own<T> {}
unsafe impl<T: ?Sized> Sync for Own<T> {}

impl<T: Sized + 'static> Own<T> {
    pub fn new(val: T) -> Self {
        let total_size = val.total_size();

        let name = std::any::type_name::<T>().to_string();

        adjust_stat(&name, 1, total_size);

        let val = Box::new(val);
        let address = val.deref().address();
        let ptr = Box::leak(val) as *mut T;

        let dealloc_prt = ptr as *mut u8 as usize;

        if address == 1 {
            panic!("Closure? Empty type?");
        }

        RefDeallocators::add_deallocator(address, move || unsafe {
            let ptr = dealloc_prt;
            let ptr = ptr as *mut u8 as *mut T;
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

impl<T: ?Sized> Own<T> {
    pub fn address(&self) -> usize {
        self.address
    }

    fn check(&self) {
        if !is_main_thread() {
            panic!(
                "Unsafe Own pointer deref: {}. Thread is not Main. Thread id: {}",
                std::any::type_name::<T>(),
                current_thread_id()
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
        adjust_stat(&self.name, -1, self.total_size);
        RefDeallocators::remove(self.address);
    }
}

impl<T: ?Sized> Deref for Own<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref().unwrap() }
    }
}

impl<T: ?Sized> DerefMut for Own<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.check();
        unsafe { self.ptr.as_mut().unwrap() }
    }
}

impl<T: ?Sized> Borrow<T> for Own<T> {
    fn borrow(&self) -> &T {
        self.deref()
    }
}

impl<T: ?Sized> BorrowMut<T> for Own<T> {
    fn borrow_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}

impl<T: ?Sized> ToWeak<T> for Own<T> {
    fn weak(&self) -> Weak<T> {
        Weak {
            address: self.address,
            ptr:     NonNull::new(self.ptr),
        }
    }
}

impl<T: Default + Sized + 'static> Default for Own<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: ?Sized + Debug> Debug for Own<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

impl<T: ?Sized + PartialEq> PartialEq for Own<T> {
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(other.deref())
    }
}

impl<T: ?Sized + PartialEq> PartialEq<T> for Own<T> {
    fn eq(&self, other: &T) -> bool {
        self.deref().eq(other)
    }
}

impl<T, U> CoerceUnsized<Own<U>> for Own<T>
where
    T: Unsize<U> + ?Sized,
    U: ?Sized,
{
}
