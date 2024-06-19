#[derive(Debug)]
pub struct Erased;

#[cfg(test)]
mod test {
    use crate::{Own, Weak};

    #[test]
    fn test_erased() {
        let a: Own<i32> = 5.into();
        let weak = a.weak();
        let erased: Weak = weak.erase();
        dbg!(&erased);
    }
}
