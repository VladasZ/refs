use derive_more::{Deref, DerefMut};

use crate::Own;

#[derive(Deref, DerefMut, Debug, PartialEq)]
pub struct OwnVec<T>(Vec<Own<T>>);

impl<T> Default for OwnVec<T> {
    fn default() -> Self {
        Self(vec![])
    }
}

impl<T> OwnVec<T> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T: 'static> From<Vec<T>> for OwnVec<T> {
    fn from(value: Vec<T>) -> Self {
        OwnVec::<T>(value.into_iter().map(Into::into).collect())
    }
}

#[cfg(test)]
mod test {
    use hreads::set_current_thread_as_main;
    use serial_test::serial;

    use crate::vec::OwnVec;

    #[test]
    #[serial]
    fn test_own_vec() {
        set_current_thread_as_main();
        let vec: Vec<u32> = vec![1, 2, 3, 4, 5];
        let mut owned_vec: OwnVec<u32> = vec.clone().into();
        let owned_vec2: OwnVec<u32> = vec.clone().into();

        assert_eq!(owned_vec, owned_vec2);

        assert_eq!(owned_vec[3], 4);

        assert_eq!(owned_vec.pop().unwrap(), 5);
        set_current_thread_as_main();
    }
}
