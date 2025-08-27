#![cfg(test)]

use serial_test::serial;

use crate::{main_lock::MainLock, set_current_thread_as_main};

struct Data {
    a: i32,
}

impl Default for Data {
    fn default() -> Self {
        Self { a: 20 }
    }
}

static DATA: MainLock<Data> = MainLock::new();
static INIT_DATA: MainLock<Data> = MainLock::new();

#[test]
#[serial]
fn test_main_lock() {
    set_current_thread_as_main();
    assert_eq!(DATA.a, 20);
    DATA.get_mut().a = 40;
    assert_eq!(DATA.a, 40);
    assert_eq!(DATA.set(Data { a: 77 }).a, 77);
    assert_eq!(DATA.a, 77);
}

#[test]
#[serial]
fn test_get_or_init() {
    set_current_thread_as_main();

    let data = INIT_DATA.get_or_init(|| Data { a: 44 });

    assert_eq!(data.a, 44);

    assert_eq!(INIT_DATA.a, 44);
}

#[test]
#[should_panic(expected = "This operation can be called only from main thread")]
fn fail_main_lock() {
    _ = DATA.a;
}
