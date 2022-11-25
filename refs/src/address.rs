use std::ops::Deref;

pub trait Address {
    fn address(&self) -> usize;
}

impl<T: ?Sized> Address for Box<T> {
    fn address(&self) -> usize {
        data_pointer(self.deref())
    }
}

impl<T: ?Sized> Address for &T {
    fn address(&self) -> usize {
        data_pointer(*self)
    }
}

impl<T: ?Sized> Address for &mut T {
    fn address(&self) -> usize {
        data_pointer(*self as *const T)
    }
}

pub(crate) fn data_pointer<T>(value: T) -> usize {
    unsafe { *((&value) as *const T as *const usize) }
}
