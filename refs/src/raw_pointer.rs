use crate::own::Stamp;

#[derive(Default, Debug, Copy, Clone)]
pub struct RawPointer {
    addr:      usize,
    stamp:     u64,
    type_name: &'static str,
}

impl RawPointer {
    pub(crate) fn addr(&self) -> usize {
        self.addr
    }

    pub(crate) fn stamp(&self) -> u64 {
        self.stamp
    }

    pub(crate) fn type_name(&self) -> &'static str {
        self.type_name
    }

    pub(crate) fn new(addr: usize, stamp: Stamp, type_name: &'static str) -> Self {
        assert_ne!(addr, 0);
        assert_ne!(stamp, 0);
        assert!(!type_name.is_empty());
        Self {
            addr,
            stamp,
            type_name,
        }
    }
}

impl PartialEq for RawPointer {
    fn eq(&self, other: &Self) -> bool {
        self.addr == other.addr && self.stamp == other.stamp
    }
}
