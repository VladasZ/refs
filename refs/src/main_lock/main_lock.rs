use std::{cell::UnsafeCell, ops::Deref};

use crate::assert_main_thread;

#[derive(Default)]
pub struct MainLock<T: Default> {
    val: UnsafeCell<Option<T>>,
}

unsafe impl<T: Default> Send for MainLock<T> {}
unsafe impl<T: Default> Sync for MainLock<T> {}

impl<T: Default> MainLock<T> {
    pub const fn new() -> Self {
        Self {
            val: UnsafeCell::new(None),
        }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn get_or_init(&self, init: impl Fn() -> T) -> &mut T {
        assert_main_thread();

        let rf = unsafe { self.val.get().as_mut().unwrap() };

        if rf.is_none() {
            *rf = Some(init());
        }

        rf.as_mut().unwrap()
    }

    #[allow(clippy::mut_from_ref)]
    pub fn get_mut(&self) -> &mut T {
        self.get_or_init(|| T::default())
    }

    /// # Safety
    ///
    /// Caller must ensure that this call is performed on main thread
    /// and that walue was already initialized
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get_unchecked(&self) -> &mut T {
        self.val.get().as_mut().unwrap().as_mut().unwrap()
    }
}

impl<T: Default + 'static> Deref for MainLock<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get_mut()
    }
}
