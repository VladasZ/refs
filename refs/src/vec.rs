use crate::{Own, ToOwn};

pub type OwnVec<T> = Vec<Own<T>>;

trait ToOwnVec<T> {
    fn to_own(self) -> OwnVec<T>;
}

impl<T: 'static> ToOwnVec<T> for Vec<T> {
    fn to_own(self) -> OwnVec<T> {
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
    fn test() {
        set_current_thread_as_main();
        let vec: Vec<u32> = vec![1, 2, 3, 4, 5];
        let owned_vec: OwnVec<u32> = vec.to_own();

        dbg!(owned_vec);
    }
}
