use crate::is_main_thread;
use std::ops::Deref;
use std::sync::Mutex;

static LOCK: Mutex<Option<Box<dyn Fn() -> bool + Send>>> = Mutex::new(None);

pub struct MainState {}

impl MainState {
    fn locked() -> bool {
        let lock = LOCK.lock().unwrap();

        let Some(check) = lock.deref() else {
            return false;
        };

        check()
    }

    pub fn safe() -> bool {
        is_main_thread() || Self::locked()
    }

    pub fn set_lock_check(check: impl Fn() -> bool + Send + 'static) {
        let mut lock = LOCK.lock().unwrap();
        *lock = Some(Box::new(check));
    }
}
