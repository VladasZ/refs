use core::ptr::from_ref;

use crate::{Address, Weak, ref_counter::RefCounter};

pub fn weak_from_ref<T: ?Sized>(rf: &T) -> Weak<T> {
    let address = rf.address();

    let Some(stamp) = RefCounter::stamp_for_address(address) else {
        panic!("Trying to get weak pointer for object which is not managed by reference counter.")
    };

    let ptr = from_ref::<T>(rf).cast_mut();
    assert!(!ptr.is_null(), "Failed to get ptr from ref");
    Weak {
        ptr,
        stamp,
        type_name: std::any::type_name::<T>(),
    }
}

#[cfg(test)]
mod test {
    use std::ops::Deref;

    use crate::{Own, weak_from_ref};

    #[test]
    #[should_panic(
        expected = "Trying to get weak pointer for object which is not managed by reference counter."
    )]
    fn test() {
        let five = Own::new(5);
        let five = five.deref();

        let weak = weak_from_ref(five);

        assert_eq!(*weak.deref(), 5);

        let five = 5;
        let five = &five;

        let _ = weak_from_ref(five);
    }
}
