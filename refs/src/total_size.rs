use std::mem::size_of;

pub trait TotalSize {
    fn total_size(&self) -> usize;
}

impl<T> TotalSize for T {
    default fn total_size(&self) -> usize {
        size_of::<T>()
    }
}

#[cfg(test)]
mod tests {
    use crate::TotalSize;
    use std::mem::size_of;
    use std::ops::Deref;

    #[derive(Default)]
    struct WithHeapData {
        _int: i32,
        data: Box<u8>,
    }

    impl TotalSize for WithHeapData {
        fn total_size(&self) -> usize {
            size_of::<Self>() + self.data.deref().total_size()
        }
    }

    #[test]
    fn total_size() {
        assert_eq!(size_of::<i32>(), 5.total_size());
        assert_eq!(size_of::<String>(), String::new().total_size());

        let data = WithHeapData::default();

        assert_eq!(
            data.total_size(),
            size_of::<WithHeapData>() + size_of::<u8>()
        );
    }
}
