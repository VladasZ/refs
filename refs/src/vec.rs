use crate::{Own, ToOwn};

pub type OwnVec<T> = Vec<Own<T>>;

pub trait ToOwnVec<T> {
    fn to_own_vec(self) -> OwnVec<T>;
}

impl<T: 'static> ToOwnVec<T> for Vec<T> {
    fn to_own_vec(self) -> OwnVec<T> {
        self.into_iter().map(ToOwn::to_own).collect()
    }
}

#[cfg(test)]
mod test {
    use serial_test::serial;

    use crate::{
        set_current_thread_as_main,
        vec::{OwnVec, ToOwnVec},
    };

    #[test]
    #[serial]
    fn convert() {
        set_current_thread_as_main();
        let vec: Vec<u32> = vec![1, 2, 3, 4, 5];
        let owned_vec: OwnVec<u32> = vec.to_own_vec();

        assert_eq!(&owned_vec, &[1, 2, 3, 4, 5]);
    }

    #[test]
    #[serial]
    fn eq() {
        set_current_thread_as_main();
        let vec: Vec<u32> = vec![1, 2, 3, 4, 5];
        let owned_vec: OwnVec<u32> = vec.clone().to_own_vec();
        let owned_vec2: OwnVec<u32> = vec.clone().to_own_vec();

        assert_eq!(owned_vec, owned_vec2);
        assert_eq!(owned_vec, vec);
    }
}
