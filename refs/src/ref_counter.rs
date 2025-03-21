use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{Stamp, own::Addr};

static COUNTER: OnceLock<RefCounter> = OnceLock::new();

type Map = HashMap<Addr, Stamp>;

#[derive(Default)]
pub(crate) struct RefCounter {
    deallocators: RwLock<Map>,
}

impl RefCounter {
    fn counter() -> RwLockReadGuard<'static, Map> {
        COUNTER.get_or_init(RefCounter::default).deallocators.read().unwrap()
    }

    fn counter_mut() -> RwLockWriteGuard<'static, Map> {
        COUNTER.get_or_init(RefCounter::default).deallocators.write().unwrap()
    }

    pub(crate) fn stamp_for_address(addr: Addr) -> Option<Stamp> {
        Self::counter().get(&addr).copied()
    }

    pub(crate) fn add(addr: Addr, stamp: Stamp) {
        let existing = Self::counter_mut().insert(addr, stamp);
        if existing.is_some() {
            unreachable!("Adding deallocator of already existing address");
        }
    }

    pub(crate) fn remove(addr: Addr) {
        Self::counter_mut().remove(&addr).expect("Removing non existing address");
    }
}
