use std::{
    alloc::{dealloc, Layout},
    borrow::{Borrow, BorrowMut},
    fmt::{Debug, Formatter},
    marker::Unsize,
    ops::{CoerceUnsized, Deref, DerefMut},
    ptr::read,
};

use crate::{ref_deallocators::RefDeallocators, stats::adjust_stat, Address, TotalSize, Weak};

pub struct Own<T: ?Sized> {
    name:       String,
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

        let dealloc_ptr = ptr as *mut u8 as usize;

        if address == 1 {
            panic!("Closure? Empty type?");
        }

        RefDeallocators::add_deallocator(address, move || unsafe {
            let ptr = dealloc_ptr;
            let ptr = ptr as *mut u8 as *mut T;
            read(ptr);
            dealloc(ptr as *mut u8, Layout::new::<T>());
        });

        Self {
            name,
            total_size,
            ptr,
        }
    }
}

impl<T: ?Sized> Own<T> {
    #[cfg(feature = "checks")]
    fn check(&self) {
        if !crate::is_main_thread() {
            panic!(
                "Unsafe Own pointer deref: {}. Thread is not Main. Thread id: {}",
                std::any::type_name::<T>(),
                crate::current_thread_id()
            );
        }
    }
}

impl<T: ?Sized> Own<T> {
    pub fn addr(&self) -> usize {
        self.ptr as *const u8 as usize
    }
}

impl<T: ?Sized> Drop for Own<T> {
    fn drop(&mut self) {
        adjust_stat(&self.name, -1, self.total_size);
        RefDeallocators::remove(self.addr());
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
        #[cfg(feature = "checks")]
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

impl<T: ?Sized> Own<T> {
    pub fn weak(&self) -> Weak<T> {
        Weak { ptr: self.ptr }
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

#[cfg(test)]
mod tests {
    use std::{
        ops::{Deref, DerefMut},
        sync::atomic::{AtomicU64, Ordering},
        thread::spawn,
    };

    use serial_test::serial;

    use crate::{set_current_thread_as_main, Own};

    #[test]
    fn deref() {
        let num = Own::new(5);
        assert_eq!(num.deref(), &5);
        assert_eq!(num.weak().deref(), &5);
    }

    #[test]
    #[should_panic(expected = "Closure? Empty type?")]
    fn own_from_closure() {
        let _ = Own::new(|| {});
    }

    #[test]
    #[serial]
    fn deref_mut() {
        set_current_thread_as_main();
        let mut num = Own::new(5);
        *num = 10;
        assert_eq!(num.deref(), &10);
        assert_eq!(num.weak().deref_mut(), &10);
    }

    #[test]
    #[serial]
    #[should_panic]
    fn deref_async() {
        set_current_thread_as_main();
        let mut num = Own::new(5);
        spawn(move || {
            assert_eq!(num.deref_mut(), &5);
        })
        .join()
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "Defererencing already freed weak pointer: i32")]
    fn deref_freed() {
        let num = Own::new(5);
        let weak = num.weak();
        drop(num);
        dbg!(weak);
    }

    static VAL: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn check_drop() {
        struct ToDrop {
            _a: bool,
        }

        impl Drop for ToDrop {
            fn drop(&mut self) {
                VAL.store(20, Ordering::Relaxed);
            }
        }

        assert_eq!(VAL.load(Ordering::Relaxed), 0);

        let num = Own::new(ToDrop { _a: false });
        let weak = num.weak();
        assert!(!weak.is_null());
        drop(num);
        assert!(weak.is_null());

        assert_eq!(VAL.load(Ordering::Relaxed), 20);
    }

    #[test]
    fn misc() {
        let five = Own::new(5);
        let ten = Own::new(10);
        let another_five = Own::new(5);

        assert_eq!(five, another_five);
        assert_ne!(five, ten);
        assert_eq!(five, 5);
        assert_ne!(five, 10);
        assert_eq!("5", &format!("{five:?}"));
    }
}
