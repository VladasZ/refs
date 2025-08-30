#![cfg(test)]

use std::ops::Deref;

use serial_test::serial;
use wasm_bindgen_test::wasm_bindgen_test;

use crate::{Own, Weak, set_current_thread_as_main};

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[serial]
#[wasm_bindgen_test(unsupported = test)]
fn weak_misc() {
    set_current_thread_as_main();
    let five = Own::new(5);
    let ten = Own::new(10);

    assert_ne!(five, ten);

    let mut weak = five.weak();
    let another_weak = weak.clone();

    assert_eq!(weak.is_null(), false);
    assert_eq!(weak.deref(), another_weak.deref());

    let null = Weak::<i32>::default();

    assert!(null.is_null());
    assert_eq!(null.is_ok(), false);
    assert_eq!(null.get(), None);

    let five_ref = weak.get_mut().unwrap();

    assert_eq!(five_ref, &5);

    *five_ref = 10;

    assert_eq!(weak.deref(), &10);

    assert!(!weak.is_null());
    assert_eq!(weak.is_ok(), true);
    assert_eq!(weak.get(), Some(10).as_ref());

    drop(five);

    assert!(weak.is_null());
    assert_eq!(weak.is_ok(), false);
    assert_eq!(weak.get(), None);
}
