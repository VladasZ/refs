use std::{
    cell::{RefCell, RefMut},
    ops::Deref,
    ptr::null_mut,
};

use crate::assert_main_thread;

pub struct MainLock<T: Default> {
    ptr: RefCell<*mut T>,
}

unsafe impl<T: Default> Send for MainLock<T> {}
unsafe impl<T: Default> Sync for MainLock<T> {}

impl<T: Default> MainLock<T> {
    pub const fn new() -> Self {
        Self {
            ptr: RefCell::new(null_mut()),
        }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn get_mut(&self) -> &mut T {
        unsafe { self.get_ptr().as_mut().unwrap() }
    }

    unsafe fn get_ptr(&self) -> RefMut<*mut T> {
        assert_main_thread();
        let mut ptr = self.ptr.borrow_mut();
        if ptr.is_null() {
            *ptr = Box::into_raw(Box::default());
        }
        ptr
    }
}

impl<T: Default + 'static> Deref for MainLock<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.get_ptr().as_ref().unwrap() }
    }
}
