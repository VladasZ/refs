use crate::Own;

pub trait ToOwn {
    fn to_own(self) -> Own<Self>;
}

impl<T: 'static> ToOwn for T {
    fn to_own(self) -> Own<Self> {
        Own::new(self)
    }
}

#[cfg(test)]
mod test {
    use std::ops::Deref;

    use serial_test::serial;

    use crate::{set_current_thread_as_main, ToOwn};

    #[test]
    #[serial]
    fn to_own() {
        set_current_thread_as_main();
        let own = 5.to_own();
        assert_eq!(5, *own.deref());
    }
}
