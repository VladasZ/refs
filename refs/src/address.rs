use core::ptr::from_ref;
use std::ops::Deref;

pub trait Address {
    fn address(&self) -> usize;
}

impl<T: ?Sized> Address for Box<T> {
    fn address(&self) -> usize {
        from_ref::<T>(self.deref()).cast::<u8>() as usize
    }
}

impl<T: ?Sized> Address for &T {
    fn address(&self) -> usize {
        from_ref::<T>(*self).cast::<u8>() as usize
    }
}

impl<T: ?Sized> Address for &mut T {
    fn address(&self) -> usize {
        from_ref::<T>(*self).cast::<u8>() as usize
    }
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
