use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Mutex,
};

/// Read the name of the type
type CounterAndDeallocator = (usize, Box<dyn FnOnce()>);

macro_rules! static_default {
    ($type:ident) => {
        static mut _STATIC_DEFAULT: *mut $type = std::ptr::null_mut();
        impl $type {
            pub fn get() -> &'static mut $type {
                unsafe {
                    if _STATIC_DEFAULT.is_null() {
                        _STATIC_DEFAULT = Box::into_raw(Box::default());
                    }
                    _STATIC_DEFAULT.as_mut().unwrap_unchecked()
                }
            }
        }
    };
}

/// Same
#[derive(Default)]
pub(crate) struct RefCounters {
    counters: HashMap<usize, CounterAndDeallocator>,
}
static_default!(RefCounters);

static LOCK: Mutex<()> = Mutex::new(());

impl RefCounters {
    pub(crate) fn exists(addr: usize) -> bool {
        let _lock = LOCK.lock().unwrap();
        Self::get().counters.contains_key(&addr)
    }

    pub(crate) fn strong_count(addr: usize) -> usize {
        let _lock = LOCK.lock().unwrap();
        match Self::get().counters.get(&addr) {
            Some(counter) => counter.0,
            None => 0,
        }
    }

    pub(crate) fn add_strong(addr: usize, dealloc_fn: impl FnOnce() + 'static) {
        let _lock = LOCK.lock().unwrap();
        if let Entry::Vacant(entry) = Self::get().counters.entry(addr) {
            entry.insert((1, Box::new(dealloc_fn)));
        } else {
            Self::get().counters.get_mut(&addr).expect("Failed to increase strong count").0 += 1;
        }
    }

    pub(crate) fn increase_strong(addr: usize) {
        let _lock = LOCK.lock().unwrap();
        Self::get().counters.get_mut(&addr).expect("Failed to increase strong count").0 += 1;
    }

    pub(crate) fn decrease_strong(addr: usize) {
        let _lock = LOCK.lock().unwrap();
        Self::get().counters.get_mut(&addr).expect("Failed to decrease strong count").0 -= 1;
    }

    pub(crate) fn remove(addr: usize) {
        let _lock = LOCK.lock().unwrap();
        let counter = Self::get().counters.remove(&addr).expect("Removing non existing address");

        // Call dealloc
        counter.1()
    }
}
