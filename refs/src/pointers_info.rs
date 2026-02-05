use std::collections::BTreeMap;

use parking_lot::Mutex;

use crate::{Addr, Stamp};

#[derive(Default, Clone, Debug, Hash, PartialEq)]
pub(crate) struct Allocation {
    pub alloc:   Stamp,
    pub dealloc: Option<Stamp>,
}

#[derive(Default, Clone, Debug, Hash, PartialEq)]
pub(crate) struct PointerInfo {
    pub addr:        Addr,
    pub allocations: Vec<Allocation>,
}

static POINTER_INFO: Mutex<BTreeMap<Addr, PointerInfo>> = Mutex::new(BTreeMap::new());

impl PointerInfo {
    pub fn record_alloc(addr: Addr, alloc: Stamp) {
        let mut info = POINTER_INFO.lock();
        let entry = info.entry(addr).or_default();
        entry.addr = addr;
        entry.allocations.push(Allocation { alloc, dealloc: None });
    }

    pub fn record_dealloc(addr: Addr, dealloc: Stamp) {
        let mut info = POINTER_INFO.lock();
        let entry = info.get_mut(&addr).expect("Recording dealloc for non allocated pointer");
        let alloc = entry
            .allocations
            .last_mut()
            .expect("Recording dealloc for pointer without allocations");
        alloc.dealloc = Some(dealloc);
    }

    pub fn get_info(addr: Addr) -> PointerInfo {
        POINTER_INFO
            .lock()
            .get(&addr)
            .expect("Getting pointer info for unknown pointer")
            .clone()
    }
}
