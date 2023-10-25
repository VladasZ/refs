use std::{
    sync::atomic::{AtomicU64, Ordering},
    thread::current,
};

static MAIN_THREAD_ID: AtomicU64 = AtomicU64::new(0);

pub fn current_thread_id() -> u64 {
    current().id().as_u64().into()
}

pub fn assert_main_thread() {
    assert!(
        is_main_thread(),
        "This operation can be called only from main thread"
    );
}

pub fn is_main_thread() -> bool {
    current_thread_id() == supposed_main_id()
}

pub fn set_current_thread_as_main() {
    MAIN_THREAD_ID.store(current_thread_id(), Ordering::Relaxed);
}

pub(crate) fn supposed_main_id() -> u64 {
    let id = MAIN_THREAD_ID.load(Ordering::Relaxed);

    if id != 0 {
        id
    } else {
        1
    }
}

#[cfg(test)]
mod test {
    use std::sync::atomic::Ordering;

    use serial_test::serial;

    use crate::utils::{supposed_main_id, MAIN_THREAD_ID};

    #[test]
    #[serial]
    fn test() {
        MAIN_THREAD_ID.store(0, Ordering::Relaxed);
        assert_eq!(supposed_main_id(), 1);
    }
}
