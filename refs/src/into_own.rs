use crate::Own;

impl<T: 'static> From<T> for Own<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod test {
    use std::ops::Deref;

    use serial_test::serial;

    use crate::{set_current_thread_as_main, Own};

    #[test]
    #[serial]
    fn into_own() {
        set_current_thread_as_main();
        let own: Own<_> = 5.into();
        assert_eq!(5, *own.deref());
    }
}
