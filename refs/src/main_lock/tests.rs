#![cfg(test)]

use serial_test::serial;

use crate::{set_current_thread_as_main, MainLock};

struct Data {
    a: i32,
}

impl Default for Data {
    fn default() -> Self {
        Self { a: 20 }
    }
}

static DATA: MainLock<Data> = MainLock::const_new();

#[test]
#[serial]
fn test_main_lock() {
    set_current_thread_as_main();
    assert_eq!(DATA.a, 20);
    DATA.get_mut().a = 40;
    assert_eq!(DATA.a, 40);
}

#[test]
#[should_panic(expected = "This operation can be called only from main thread")]
fn fail_main_lock() {
    _ = DATA.a;
}
