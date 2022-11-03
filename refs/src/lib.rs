#![allow(incomplete_features)]
#![feature(specialization)]
#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(const_trait_impl)]
#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_for)]
#![feature(ptr_metadata)]
#![feature(const_default_impls)]
#![feature(thread_id_value)]

pub mod own;
pub(crate) mod ref_counters;
pub mod rglica;
pub mod strong;
pub mod to_rglica;
pub mod to_weak;
pub mod utils;
pub mod weak;

pub use own::*;
pub(crate) use ref_counters::*;
pub use rglica::*;
pub use strong::*;
pub use to_rglica::*;
pub use to_weak::*;
pub use utils::*;
pub use weak::*;

#[cfg(test)]
mod tests {
    use crate::ref_counters::RefCounters;
    use crate::{set_current_thread_as_main, Own, Strong, ToWeak, Weak};
    use serial_test::serial;
    use std::ops::Deref;
    use std::thread::spawn;

    #[test]
    #[serial]
    fn deref() {
        set_current_thread_as_main();
        let num = Own::new(5);
        assert_eq!(num.deref(), &5);
        assert_eq!(num.weak().deref(), &5);
    }

    #[test]
    #[serial]
    fn deref_mut() {
        set_current_thread_as_main();
        let mut num = Own::new(5);
        *num = 10;
        assert_eq!(num.deref(), &10);
        assert_eq!(num.weak().deref(), &10);
    }

    #[test]
    #[should_panic]
    #[serial]
    fn deref_async() {
        set_current_thread_as_main();
        let num = Own::new(5);
        let weak = num.weak();
        spawn(move || {
            assert_eq!(weak.deref(), &5);
        })
        .join()
        .unwrap();
    }

    #[test]
    #[serial]
    fn counters() {
        set_current_thread_as_main();
        let num = Strong::new(5);
        assert_eq!(num.ref_count(), 1);
        let num2 = num.clone();
        assert_eq!(num.ref_count(), 2);
        drop(num2);
        assert_eq!(num.ref_count(), 1);
        let address = num.address();
        drop(num);
        assert_eq!(RefCounters::strong_count(address), 0);
    }

    #[test]
    #[should_panic]
    #[serial]
    fn deref_null() {
        set_current_thread_as_main();
        let null = Weak::<u32>::default();
        dbg!(&null);
    }

    #[test]
    #[should_panic]
    #[serial]
    fn deref_freed() {
        set_current_thread_as_main();
        let num = Strong::new(5);
        let weak = num.weak();
        drop(num);
        dbg!(weak);
    }

    #[test]
    #[serial]
    fn check_freed() {
        set_current_thread_as_main();
        let num = Strong::new(5);
        let weak = num.weak();
        assert!(!weak.freed());
        drop(num);
        assert!(weak.freed());
    }

    #[test]
    #[serial]
    fn from_ref_ok() {
        set_current_thread_as_main();
        let num = Strong::new(5);
        let rf = num.deref();
        let weak = Weak::from_ref(rf);
        assert!(weak.is_ok());
        _ = weak.deref();
    }

    #[test]
    #[should_panic]
    #[serial]
    fn from_ref_fail() {
        set_current_thread_as_main();
        let _weak = Weak::from_ref(&5);
    }
}
