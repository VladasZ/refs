use crate::Weak;

pub trait ToWeak<T: ?Sized> {
    fn weak(&self) -> Weak<T>;
}

impl<T: ?Sized> ToWeak<T> for &T {
    fn weak(&self) -> Weak<T> {
        Weak::from_ref(self)
    }
}

impl<T: ?Sized> ToWeak<T> for &mut T {
    fn weak(&self) -> Weak<T> {
        Weak::from_ref(self)
    }
}

#[cfg(test)]
mod test {
    use std::ops::{Deref, DerefMut};

    use serial_test::serial;

    use crate::{set_current_thread_as_main, Own, ToWeak};

    #[test]
    #[serial]
    fn test() {
        set_current_thread_as_main();
        let mut five = Own::new(5);

        let five_ref = five.deref();
        let weak = five_ref.weak();
        assert_eq!(*weak.deref(), 5);

        let five_ref = five.deref_mut();
        let weak = five_ref.weak();
        assert_eq!(*weak.deref(), 5);
    }
}
