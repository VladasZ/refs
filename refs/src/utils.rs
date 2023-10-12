use std::{
    sync::atomic::{AtomicU64, Ordering},
    thread::current,
};

static MAIN_THREAD_ID: AtomicU64 = AtomicU64::new(0);

pub fn current_thread_id() -> u64 {
    current().id().as_u64().into()
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
