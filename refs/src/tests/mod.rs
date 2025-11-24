#![cfg(test)]

use std::{
    any::Any,
    collections::HashMap,
    ops::{Deref, DerefMut},
    thread::spawn,
};

use pretty_assertions::assert_eq;
use serial_test::serial;
use wasm_bindgen_test::wasm_bindgen_test;

use crate::{AsAny, Own, Weak, set_current_thread_as_main};

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

#[serial]
#[wasm_bindgen_test(unsupported = test)]
fn leak_weak() {
    set_current_thread_as_main();
    let leaked = unsafe { Weak::leak(5) };
    dbg!(leaked.deref());
}

#[wasm_bindgen_test(unsupported = test)]
#[should_panic(expected = "Invalid address. In cou be a closure or empty type.")]
fn leak_weak_closure() {
    let _leaked = unsafe { Weak::leak(|| {}) };
}

#[serial]
#[wasm_bindgen_test(unsupported = test)]
fn addr() {
    let own = Own::new(5);
    let weak = own.weak();
    assert_eq!(own.addr(), weak.addr());
}

#[wasm_bindgen_test(unsupported = test)]
#[should_panic(expected = "Defererencing never initialized weak pointer: i32")]
fn null_weak_panic() {
    let default = Weak::<i32>::default();
    assert_eq!(default.is_ok(), false);
    let _ = default.deref();
}

#[wasm_bindgen_test(unsupported = test)]
#[should_panic(expected = "Defererencing already freed weak pointer: i32")]
fn freed_unsized_weak_panic() {
    let own = Own::new(5);
    let weak: Weak<dyn Any> = own.weak();
    drop(own);

    assert_eq!(weak.type_name, "i32");
    assert_eq!(weak.is_ok(), false);
    let _ = weak.deref();
}

static WEAK: Weak<bool> = Weak::const_default();

#[serial]
#[wasm_bindgen_test(unsupported = test)]
fn const_weak_default() {
    set_current_thread_as_main();
    assert!(WEAK.is_null());
}

#[serial]
#[should_panic]
#[wasm_bindgen_test(unsupported = test)]
fn deref_null() {
    set_current_thread_as_main();
    let null = Weak::<u32>::default();
    assert!(null.is_null());
    assert_eq!(null.is_ok(), false);
    dbg!(&null);
}

#[serial]
#[should_panic]
#[wasm_bindgen_test(unsupported = test)]
fn deref_async() {
    set_current_thread_as_main();
    let num = Own::new(5);
    let mut weak = num.weak();
    spawn(move || {
        assert_eq!(weak.deref(), &5);
        assert_eq!(weak.deref_mut(), &5);
    })
    .join()
    .unwrap();
}

#[wasm_bindgen_test(unsupported = test)]
fn default_weak() {
    let weak = Weak::<i32>::default();
    assert!(weak.is_null());

    trait Trait {
        fn _a(&self);
    }
    let weak = Weak::<dyn Trait>::default();
    assert!(weak.is_null());
}

#[wasm_bindgen_test(unsupported = test)]
fn downcast_weak() {
    trait Tr: AsAny {}
    struct St {
        _a: i32,
    }

    impl Tr for St {}
    impl AsAny for St {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    let own: Own<dyn Tr> = Own::new(St { _a: 50 });
    let downcasted: Weak<St> = own.downcast().unwrap();

    assert_eq!(downcasted._a, 50);
}

#[wasm_bindgen_test(unsupported = test)]
fn weak_map_key() {
    struct NonHash {
        _a: u8,
    }
    let own = Own::new(NonHash { _a: 0 });
    let weak = own.weak();

    let mut map: HashMap<Weak<NonHash>, u32> = HashMap::new();
    map.entry(weak).or_insert(5);
    assert_eq!(map.get(&weak).unwrap(), &5);
}

#[wasm_bindgen_test(unsupported = test)]
fn was_initialized() {
    let a = Weak::<i32>::default();
    let b = Own::new(5);
    let b = b.weak();

    assert!(!a.was_initialized());
    assert!(b.was_initialized());
}

#[wasm_bindgen_test(unsupported = test)]
fn raw_and_dump() {
    let a = Own::new(5);
    let a = a.weak();
    let erased = a.erase();

    let raw = a.raw();

    let from_raw: Weak<i32> = unsafe { Weak::from_raw(raw) };
    let from_raw_unsized: Weak<dyn Any> = unsafe { Weak::from_raw(raw) };

    assert_eq!(a.raw(), from_raw.raw());
    assert_eq!(a.raw(), from_raw_unsized.raw());
    assert_eq!(a.raw(), erased.raw());
}
