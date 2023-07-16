use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard, OnceLock},
};

type Deallocator = Box<dyn FnOnce() + Send>;

static DEALLOCATORS: OnceLock<RefDeallocators> = OnceLock::new();

#[derive(Default)]
pub(crate) struct RefDeallocators {
    deallocators: Mutex<HashMap<usize, Deallocator>>,
}

impl RefDeallocators {
    fn deallocators() -> MutexGuard<'static, HashMap<usize, Deallocator>> {
        DEALLOCATORS.get_or_init(RefDeallocators::default).deallocators.lock().unwrap()
    }

    pub(crate) fn exists(addr: usize) -> bool {
        Self::deallocators().contains_key(&addr)
    }

    pub(crate) fn add_deallocator(addr: usize, dealloc: impl FnOnce() + Send + 'static) {
        let existing = Self::deallocators().insert(addr, Box::new(dealloc));
        if existing.is_some() {
            panic!("Adding deallocator of already existing address");
        }
    }

    pub(crate) fn remove(addr: usize) {
        let deallocator = Self::deallocators().remove(&addr).expect("Removing non existing address");
        deallocator()
    }
}
