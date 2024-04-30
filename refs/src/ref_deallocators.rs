use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{own::Addr, Stamp};

type Deallocator = Box<dyn FnOnce() + Send + Sync>;

static DEALLOCATORS: OnceLock<RefDeallocators> = OnceLock::new();

type Map = HashMap<Addr, (Deallocator, Stamp)>;

#[derive(Default)]
pub(crate) struct RefDeallocators {
    deallocators: RwLock<Map>,
}

impl RefDeallocators {
    fn deallocators() -> RwLockReadGuard<'static, Map> {
        DEALLOCATORS.get_or_init(RefDeallocators::default).deallocators.read().unwrap()
    }

    fn deallocators_mut() -> RwLockWriteGuard<'static, Map> {
        DEALLOCATORS.get_or_init(RefDeallocators::default).deallocators.write().unwrap()
    }

    pub(crate) fn stamp_for_address(addr: Addr) -> Option<Stamp> {
        Self::deallocators().get(&addr).map(|a| a.1)
    }

    pub(crate) fn add_deallocator(addr: Addr, stamp: Stamp, dealloc: impl FnOnce() + Send + Sync + 'static) {
        let existing = Self::deallocators_mut().insert(addr, (Box::new(dealloc), stamp));
        if existing.is_some() {
            unreachable!("Adding deallocator of already existing address");
        }
    }

    pub(crate) fn remove(addr: Addr) {
        let deallocator = Self::deallocators_mut().remove(&addr).expect("Removing non existing address").0;
        deallocator();
    }
}
