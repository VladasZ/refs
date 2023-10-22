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

pub fn data_pointer<T>(value: T) -> usize {
    unsafe { *((&value) as *const T as *const usize) }
}

#[cfg(test)]
mod test {
    use crate::Address;

    #[test]
    fn address() {
        let five = 5;
        assert_eq!((&five).address(), (&five) as *const i32 as usize);

        let five = Box::new(5);
        assert_eq!(five.address(), five.as_ref() as *const i32 as usize);

        let five = &mut 5;
        assert_eq!(five.address(), five as *mut i32 as usize);
    }
}
