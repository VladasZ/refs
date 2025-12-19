use std::{
    fmt::{Debug, Formatter},
    marker::Unsize,
    ops::{CoerceUnsized, Deref, DerefMut},
    ptr,
    ptr::from_ref,
};

use crate::{AsAny, RawPointer, Weak, ref_counter::RefCounter};

pub(crate) type Stamp = u64;
pub(crate) type Addr = usize;

pub struct Own<T: ?Sized> {
    bx:        Box<T>,
    stamp:     Stamp,
    type_name: &'static str,
}

unsafe impl<T: ?Sized> Send for Own<T> {}
unsafe impl<T: ?Sized> Sync for Own<T> {}

pub(crate) fn stamp() -> Stamp {
    use instant::SystemTime;

    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
        .try_into()
        .unwrap()
}

impl<T: Sized + 'static> Own<T> {
    pub fn new(val: T) -> Self {
        let stamp = stamp();

        let type_name = std::any::type_name::<T>();

        // #[cfg(feature = "stats")]
        // crate::stats::adjust_stat(type_name, 1);

        let val = Box::new(val);
        let address = from_ref::<T>(&val).cast::<u8>() as usize;

        assert_ne!(address, 1, "Invalid address. In cou be a closure or empty type.");

        RefCounter::add(address, stamp);

        Self {
            bx: val,
            stamp,
            type_name,
        }
    }
}

impl<T: ?Sized + AsAny> Own<T> {
    pub fn downcast<U: 'static>(&self) -> Option<Weak<U>> {
        self.weak().downcast()
    }
}

impl<T: ?Sized> Own<T> {
    #[cfg(feature = "checks")]
    fn check() {
        assert!(
            hreads::is_main_thread(),
            "Unsafe Own pointer deref: {}. Thread is not Main. Thread id: {}",
            std::any::type_name::<T>(),
            hreads::current_thread_id()
        );
    }
}

impl<T: ?Sized> Own<T> {
    pub(crate) fn addr(&self) -> usize {
        from_ref::<T>(self.bx.as_ref()).cast::<u8>() as usize
    }
}

impl<T: ?Sized> Drop for Own<T> {
    fn drop(&mut self) {
        // #[cfg(feature = "stats")]
        // crate::stats::adjust_stat(self.type_name, -1);
        RefCounter::remove(self.addr());
    }
}

impl<T: ?Sized> Deref for Own<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.bx.deref()
    }
}

impl<T: ?Sized> DerefMut for Own<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[cfg(feature = "checks")]
        Self::check();
        self.bx.deref_mut()
    }
}

impl<T: ?Sized> Own<T> {
    pub fn weak(&self) -> Weak<T> {
        Weak {
            ptr:       ptr::from_ref(self.bx.deref()).cast_mut(),
            stamp:     self.stamp,
            type_name: self.type_name,
        }
    }

    pub fn raw(&self) -> RawPointer {
        RawPointer::new(self.addr(), self.stamp, self.type_name)
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

    use hreads::set_current_thread_as_main;
    use serial_test::serial;

    use crate::Own;

    #[test]
    fn deref() {
        let num = Own::new(5);
        assert_eq!(num.deref(), &5);
        assert_eq!(num.weak().deref(), &5);
    }

    #[test]
    #[should_panic(expected = "Invalid address. In cou be a closure or empty type.")]
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
        let five_int = 5;

        assert_eq!(five, another_five);
        assert_ne!(five, ten);
        assert_eq!(five, 5);
        assert_ne!(five, 10);
        assert_eq!("5", &format!("{five:?}"));
        assert_eq!(five, five_int);
    }
}
