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
