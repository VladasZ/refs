use std::{
    alloc::{dealloc, Layout},
    borrow::{Borrow, BorrowMut},
    fmt::{Debug, Formatter},
    marker::Unsize,
    ops::{CoerceUnsized, Deref, DerefMut},
    ptr::{read, NonNull},
};

use crate::{ref_deallocators::RefDeallocators, stats::adjust_stat, Address, ToWeak, TotalSize, Weak};

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

impl<T: ?Sized> ToWeak<T> for Own<T> {
    fn weak(&self) -> Weak<T> {
        Weak {
            ptr: NonNull::new(self.ptr),
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

#[cfg(test)]
mod tests {
    use std::{
        ops::{Deref, DerefMut},
        thread::spawn,
    };

    use serial_test::serial;

    use crate::{set_current_thread_as_main, Own, ToWeak, Weak};

    #[test]
    fn deref() {
        let num = Own::new(5);
        assert_eq!(num.deref(), &5);
        assert_eq!(num.weak().deref(), &5);
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

    #[test]
    fn check_freed() {
        let num = Own::new(5);
        let weak = num.weak();
        assert!(!weak.freed());
        drop(num);
        assert!(weak.freed());
    }

    #[test]
    fn from_ref_ok() {
        let num = Own::new(5);
        let rf = num.deref();
        let weak = Weak::from_ref(rf);
        assert!(weak.is_ok());
        _ = weak.deref();
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
