use std::{collections::HashMap, sync::OnceLock};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{Stamp, own::Addr};

static COUNTER: OnceLock<RefCounter> = OnceLock::new();

type Map = HashMap<Addr, Stamp>;

#[derive(Default)]
pub(crate) struct RefCounter {
    deallocators: RwLock<Map>,
}

impl RefCounter {
    fn counter() -> RwLockReadGuard<'static, Map> {
        COUNTER.get_or_init(RefCounter::default).deallocators.read()
    }

    fn counter_mut() -> RwLockWriteGuard<'static, Map> {
        COUNTER.get_or_init(RefCounter::default).deallocators.write()
    }

    pub(crate) fn stamp_for_address(addr: Addr) -> Option<Stamp> {
        Self::counter().get(&addr).copied()
    }

    pub(crate) fn add(addr: Addr) -> Stamp {
        let stamp = stamp();
        let existing = Self::counter_mut().insert(addr, stamp);
        if existing.is_some() {
            unreachable!("Adding deallocator of already existing address");
        }

        #[cfg(feature = "pointers_info")]
        crate::pointers_info::PointerInfo::record_alloc(addr, stamp);

        stamp
    }

    pub(crate) fn remove(addr: Addr) {
        Self::counter_mut().remove(&addr).expect("Removing non existing address");
        #[cfg(feature = "pointers_info")]
        crate::pointers_info::PointerInfo::record_dealloc(addr, stamp());
    }
}

fn stamp() -> Stamp {
    #[cfg(miri)]
    {
        static mut STATIC_START_TIME: Instant =
            unsafe { std::mem::transmute([0u8; std::mem::size_of::<Instant>()]) };

        use std::time::{Instant, UNIX_EPOCH};

        let now = Instant::now();
        let pseudo_duration = now.duration_since(unsafe { STATIC_START_TIME });
        let dur = UNIX_EPOCH.duration_since(UNIX_EPOCH).unwrap() + pseudo_duration;
        dur.as_secs()
    }

    #[cfg(not(miri))]
    {
        use instant::SystemTime;

        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
            .try_into()
            .unwrap()
    }
}
