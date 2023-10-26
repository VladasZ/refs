use std::ptr::NonNull;

use crate::Rglica;

pub trait ToRglica<T: ?Sized> {
    fn to_rglica(&self) -> Rglica<T>;
}

impl<T: ?Sized> ToRglica<T> for Box<T> {
    fn to_rglica(&self) -> Rglica<T> {
        let ptr = NonNull::new((self.as_ref() as *const T).cast_mut());
        debug_assert!(ptr.is_some(), "Failed to make Rglica from Box");
        Rglica {
            ptr: ptr.unwrap().into(),
        }
    }
}

impl<T: ?Sized> ToRglica<T> for &T {
    fn to_rglica(&self) -> Rglica<T> {
        Rglica::from_ref(self)
    }
}

impl<T: ?Sized> ToRglica<T> for &mut T {
    fn to_rglica(&self) -> Rglica<T> {
        Rglica::from_ref(self)
    }
}

#[cfg(test)]
mod test {
    use std::ops::Deref;

    use crate::ToRglica;

    #[test]
    fn test() {
        let five_ref = &5_i32;
        let five = five_ref.to_rglica();
        assert_eq!(*five.deref(), 5);

        let five_mut = &mut 5_i32;
        let five = five_mut.to_rglica();
        assert_eq!(*five.deref(), 5);

        let five_box = Box::new(5);
        let five = five_box.to_rglica();
        assert_eq!(*five.deref(), 5);
    }
}
